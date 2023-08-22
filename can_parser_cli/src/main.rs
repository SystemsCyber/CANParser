use clap::{arg, builder::PossibleValue, Arg, ArgGroup, Command};
use std::collections::HashMap;
use std::fs;
use std::io::Read;

use can_parser::{CANParser, CANParserError, ERROR_WARN, LOG_TYPE_BINARY, LOG_TYPE_TEXT};

fn cli() -> Command {
    Command::new("can_parser_cli")
        .version("1.0")
        .author("Jake Jepson")
        .about("Parses CAN log files or a single CAN message.")
        .arg_required_else_help(true)
        .group(
            ArgGroup::new("input")
                .args(&["file", "message"])
                .required(true)
                .multiple(false),
        )
        .next_help_heading(Some("Input Options"))
        .args([
            arg!(-f --file <FILE_PATH> "CAN log file to parse"),
            arg!(-m --message <MSG> "Single CAN message to parse"),
        ])
        .group(
            ArgGroup::new("parsing")
                .args(&["template", "custom_regex"])
                .required(true)
                .multiple(false),
        )
        .next_help_heading(Some("Parsing Options"))
        .args([
            arg!(-t --template <TEMPLATE> "Regex template for parsing")
                .value_parser([PossibleValue::new("candump").help("candump format")]),
            arg!(-r --custom_regex <REGEX> "Custom regex expression"),
        ])
        .group(
            ArgGroup::new("specification")
                .args(&["specs", "specs_types"])
                .required(false)
                .multiple(true),
        )
        .next_help_heading(Some("Specification Options"))
        .args([
            Arg::new("specs")
                .short('s')
                .long("specs")
                .value_name("PATH")
                .help("Comma separated list of specification files")
                .value_delimiter(','),
            Arg::new("specs_types")
                .short('S')
                .long("specs_types")
                .value_name("TYPE")
                .help("Comma separated list of specification types")
                .value_delimiter(','),
        ])
        .group(
            ArgGroup::new("output options")
                .args(&["output", "force"])
                .required(false)
                .multiple(true),
        )
        .next_help_heading(Some("Output Options"))
        .args([
            arg!(-o --output <PATH> "File path to write the results"),
            Arg::new("force")
                .long("force")
                .help("Forcefully overwrite the output file if it exists")
                .action(clap::ArgAction::SetTrue),
            arg!(-'F' --format <FORMAT> "Output format").value_parser([
                PossibleValue::new("json").help("JSON format"),
                PossibleValue::new("csv").help("CSV format"),
                PossibleValue::new("sqlite").help("SQLite format"),
            ]),
        ])
}

fn detect_file_type(path: &str) -> Result<String, String> {
    // Read the first few bytes
    let mut file = fs::File::open(path).map_err(|e| format!("Error: {}", e))?;
    let mut buffer = [0; 5];
    file.read(&mut buffer)
        .map_err(|e| format!("Error: {}", e))?;
    let content = String::from_utf8_lossy(&buffer);

    if content.chars().all(|c| c.is_ascii() && !c.is_control()) {
        return Ok(LOG_TYPE_TEXT.to_string());
    }
    match path.split('.').last().unwrap_or_default() {
        "txt" | "log" => Ok(LOG_TYPE_TEXT.to_string()),
        _ => Ok(LOG_TYPE_BINARY.to_string()),
    }
}

fn write_results(matches: &clap::ArgMatches, parser: &mut CANParser) -> Result<(), String> {
    match matches.get_one::<String>("output") {
        Some(output) => {
            if !matches.get_flag("force") {
                let path = std::path::Path::new(&output);
                if path.exists() {
                    return Err(format!(
                        "Output file {} already exists. Use --force to overwrite",
                        output
                    ));
                }
            }
            match matches.get_one::<String>("format") {
                Some(format) => match format.as_str() {
                    "json" => {
                        parser
                            .to_json(Some(output.clone()))
                            .map_err(|e| format!("Error: {}", e))?;
                    }
                    "csv" => {
                        parser
                            .to_csv(Some(output.clone()))
                            .map_err(|e| format!("Error: {}", e))?;
                    }
                    "sqlite" => {
                        parser
                            .to_sqlite(output.clone())
                            .map_err(|e| format!("Error: {}", e))?;
                    }
                    _ => return Err("Invalid format".to_string()),
                },
                None => {
                    parser
                        .to_json(Some(output.clone()))
                        .map_err(|e| format!("Error: {}", e))?;
                }
            }
        }
        None => println!(
            "{}",
            parser
                .to_json(None)
                .map_err(|e| format!("Error: {}", e))?
                .unwrap_or_else(|| "No results".to_string())
        ),
    }
    Ok(())
}

fn parse_input(matches: &clap::ArgMatches) -> Result<(), String> {
    let _file_type = match matches.get_one::<String>("file") {
        Some(path) => detect_file_type(path)?,
        None => LOG_TYPE_TEXT.to_string(),
    };
    let line_regex = match matches.get_one::<String>("template") {
        Some(template) => match template.as_str() {
            "candump" => {
                r"^\((?P<timestamp>[0-9]+\.[0-9]+)\).*?(?P<id>[0-9A-F]{3,8})#(?P<data>[0-9A-F]+)"
            }
            _ => return Err("Invalid template".to_string()),
        },
        None => match matches.get_one::<String>("custom_regex") {
            Some(regex) => regex,
            None => return Err("No regex template or custom regex provided".to_string()),
        },
    };
    let specs_map: Option<HashMap<String, String>> = match matches.get_many::<String>("specs_types")
    {
        Some(specs_types) => {
            let specs_types: Vec<String> = specs_types.cloned().collect();

            if let Some(specs) = matches.get_many::<String>("specs") {
                let specs: Vec<String> = specs.cloned().collect();
                if specs.len() != specs_types.len() {
                    return Err(
                        "The number of specification files and types must be equal".to_string()
                    );
                }
                Some(specs_types.into_iter().zip(specs).collect())
            } else {
                return Err("No specification files provided".to_string());
            }
        }
        None => None,
    };
    let mut parser = CANParser::new(
        ERROR_WARN.to_string(),
        Some(line_regex.to_string()),
        specs_map,
    )
    .map_err(|e| format!("Error: {}", e))?;

    let results;
    if let Some(path) = matches.get_one::<String>("file") {
        results = parser.parse_file(path);
    } else if let Some(message) = matches.get_one::<String>("message") {
        let messages = vec![message.clone()];
        results = parser.parse_lines(&messages);
    } else {
        return Err("No input provided".to_string());
    }
    match results {
        Ok(_) => write_results(matches, &mut parser),
        Err(e) => match e {
            CANParserError::ParserError(e) => {
                return Err(e);
            }
            CANParserError::ParserWarning(e) => {
                eprintln!(
                    "The parser threw some warning(s) while parsing the file: {:?}",
                    e
                );
                write_results(matches, &mut parser)
            }
            _ => Err(format!("Error: {}", e)),
        },
    }
}

fn main() {
    let matches = cli().get_matches();
    if let Err(e) = parse_input(&matches) {
        eprintln!("😓 Oops! An error occurred: {}", e);
    }
}
