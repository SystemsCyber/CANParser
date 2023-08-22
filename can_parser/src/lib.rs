mod can_message;
mod error;
mod j1939_spec;
#[macro_use]
mod utils;
mod csv_serializer;
mod json_serializer;
mod specification;
#[cfg(feature = "sqlite")]
mod sqlite_serializer;
pub use can_message::{parse_id, parse_j1939_data, CANMessage, CANID};
use csv_serializer::to_csv;
pub use error::CANParserError;
use json_serializer::to_json;
pub use specification::{Metadata, SpecPGN, SpecSPN, Specification, FilteredSpec};
#[cfg(feature = "sqlite")]
use sqlite_serializer::to_sqlite;

use crate::j1939_spec::J1939Spec;

use regex::Regex;
#[cfg(feature = "sqlite")]
use rusqlite::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
#[cfg(feature = "debug")]
use std::mem;
use std::sync::{Arc, Mutex, RwLock};
#[cfg(feature = "wasm")]
use web_sys::console;

impl IntoIterator for FilteredSpec {
    type Item = (String, HashMap<u16, SpecPGN>);
    type IntoIter = std::collections::hash_map::IntoIter<String, HashMap<u16, SpecPGN>>;

    fn into_iter(self) -> Self::IntoIter {
        let mut map = HashMap::new();
        map.insert("j1939".to_string(), (*self.j1939.read().unwrap()).clone());
        map.into_iter()
    }
}

pub const LOG_TYPE_BINARY: &'static str = "binary";
pub const LOG_TYPE_TEXT: &'static str = "text";

pub const ERROR_IGNORE: &'static str = "ignore";
pub const ERROR_WARN: &'static str = "warn";

pub const SPEC_TYPE_J1939: &'static str = "j1939";
pub const SPEC_TYPE_CAN: &'static str = "can";
pub const SPEC_TYPE_UDS: &'static str = "uds";
pub const SPEC_TYPE_TRANSPORT: &'static str = "transport";

/// A struct representing the specifications for various protocols used in CAN communication.
struct Specs {
    pub j1939: Option<J1939Spec>,
    // pub can: Option<J1939Spec>,
    // pub uds: Option<J1939Spec>,
    // pub transport: Option<J1939Spec>,
}

impl Default for Specs {
    fn default() -> Self {
        Self {
            j1939: None,
            // can: None,
            // uds: None,
            // transport: None,
        }
    }
}

/// A struct representing a CAN parser.
pub struct CANParser {
    /// A regular expression used to match lines in the input file.
    line_regex: Arc<Regex>,
    /// An optional reference to a `Specs` struct.
    specs: Option<Arc<Specs>>,
    /// A string representing the error handling mode.
    error_handling: String,
    /// A thread-safe reference to a `FileFlags` struct.
    pub flags: Arc<RwLock<FileFlags>>,
    /// A thread-safe reference to a `FilteredSpec` struct.
    pub filtered_spec: Arc<FilteredSpec>,
    /// A vector of `CANMessage` structs.
    pub messages: Vec<CANMessage>,
}

/// Represents the flags for different types of protocol that might found during parsing by the CAN parser.
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct FileFlags {
    pub j1939: bool,
    pub canfd: bool,
    pub transport_protocol: bool,
    pub uds: bool,
    pub ethernet: bool,
}

impl Default for FileFlags {
    fn default() -> Self {
        Self {
            j1939: false,
            canfd: false,
            transport_protocol: false,
            uds: false,
            ethernet: false,
        }
    }
}

impl CANParser {
    /// Creates a new `CANParser` instance with the given error handling, line regex, and specs annexes.
    ///
    /// # Arguments
    ///
    /// * `error_handling` - A `String` that specifies the error handling method to use. Valid values are "warn", and "ignore".
    /// * `line_regex` - An optional `String` that specifies the regular expression to use for parsing lines. If `None`, its assumed the file is binary.
    /// * `specs_annexes` - An optional `HashMap<String, String>` that specifies the specification annexes to use for parsing messages.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `CANParser` instance if successful, or a `CANParserError` if an error occurred.
    pub fn new(
        error_handling: String,
        line_regex: Option<String>,
        specs_annexes: Option<HashMap<String, String>>,
    ) -> Result<Self, CANParserError> {
        #[cfg(feature = "wasm")]
        console::log_1(
            &format!("CANParser::new({:?})", line_regex.clone().unwrap())
                .as_str()
                .into(),
        );
        let line_regex = line_regex.map(|s| Regex::new(&s)).transpose()?;

        let known_keys = vec![
            SPEC_TYPE_J1939,
            SPEC_TYPE_CAN,
            SPEC_TYPE_UDS,
            SPEC_TYPE_TRANSPORT,
        ]; // ... add other known keys as necessary

        let specs = if let Some(annexes) = specs_annexes {
            // Check for any unknown keys
            for key in annexes.keys() {
                if !known_keys.contains(&key.to_ascii_lowercase().as_str()) {
                    return Err(CANParserError::ParserError(format!(
                        "Unknown spec key: {}",
                        key
                    )));
                }
            }

            Some(Arc::new(Specs {
                j1939: Self::fetch_spec::<J1939Spec>(&annexes, SPEC_TYPE_J1939)?,
                // can: None,       // Self::fetch_spec::<CANSpec>(&annexes, CAN)?,
                // uds: None,       // Self::fetch_spec::<UDSSpec>(&annexes, UDS)?,
                // transport: None, // Self::fetch_spec::<TransportSpec>(&annexes, TRANSPORT)?,
            }))
        } else {
            None
        };

        Ok(Self {
            line_regex: Arc::new(line_regex.unwrap_or_else(|| Regex::new("").unwrap())),
            specs,
            error_handling: error_handling.to_ascii_lowercase(),
            flags: Arc::new(RwLock::new(FileFlags::default())),
            filtered_spec: Arc::new(FilteredSpec::default()),
            messages: Vec::with_capacity(0),
        })
    }

    /// Fetches a specification from the given `annexes` hashmap using the specified `key`.
    ///
    /// # Arguments
    ///
    /// * `annexes` - A hashmap containing the annexes to search for the specification.
    /// * `key` - The key to use to search for the specification in the `annexes` hashmap.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing an `Option` of the fetched specification or a `CANParserError` if an error occurred.
    fn fetch_spec<T: 'static + Specification>(
        annexes: &HashMap<String, String>,
        key: &str,
    ) -> Result<Option<T>, CANParserError> {
        if let Some(annex) = annexes.get(key) {
            Ok(Some(T::new(&annex)?))
        } else {
            Ok(None)
        }
    }

    /// Parses a file containing CAN messages and returns a vector of parsed messages.
    ///
    /// # Arguments
    ///
    /// * `self` - A mutable reference to the CANParser instance.
    /// * `file_path` - A string slice that holds the path to the file to be parsed.
    ///
    /// # Errors
    ///
    /// Returns a `CANParserError` if the file cannot be opened or if there are parsing errors.
    ///
    /// # Example
    ///
    /// ```
    /// use can_parser::CANParser;
    ///
    /// let mut parser = CANParser::new();
    /// parser.parse_file("path/to/file.log").unwrap();
    /// ```
    pub fn parse_file(&mut self, file_path: &str) -> Result<(), CANParserError> {
        #[cfg(feature = "debug")]
        let start_time = Self::current_time();
        let file = File::open(file_path)?;

        let reader = BufReader::new(file);
        let errors = Arc::new(Mutex::new(vec![]));

        let parse_file_line = |line_result: Result<String, std::io::Error>| -> Option<_> {
            match line_result {
                Ok(line) => Some(line),
                Err(e) => {
                    Self::handle_parsing_error(
                        &self.error_handling,
                        &errors,
                        e.to_string(),
                        &String::new(),
                    );
                    None
                }
            }
        };

        // Function to parse a line
        let parse_can_message = |line: String| -> Option<_> {
            match Self::parse_line_inner(&self.specs, &self.line_regex, &line, &self.filtered_spec)
            {
                Ok(message) => Some(message),
                Err(e) => {
                    Self::handle_parsing_error(&self.error_handling, &errors, e, &line);
                    None
                }
            }
        };

        // Core Logic
        self.messages = if cfg!(feature = "parallel") {
            #[cfg(feature = "parallel")]
            {
                use rayon::prelude::*;
                reader
                    .lines()
                    .filter_map(parse_file_line)
                    .par_bridge()
                    .filter_map(parse_can_message)
                    .collect()
            }
            #[cfg(not(feature = "parallel"))]
            {
                panic!("Parallel feature not enabled");
            }
        } else {
            reader
                .lines()
                .filter_map(parse_file_line)
                .filter_map(parse_can_message)
                .collect()
        };

        // Debugging Logic
        #[cfg(feature = "debug")]
        Self::debug_log(&self.messages, Self::current_time() - start_time);

        // Error Check
        let errors = errors.lock().unwrap();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(CANParserError::ParserWarning(errors.clone()))
        }
    }

    /// Parses a vector of CAN messages from a vector of strings.
    ///
    /// # Arguments
    ///
    /// * `self` - A mutable reference to the CANParser instance.
    /// * `lines` - A vector of strings containing the CAN messages to be parsed.
    ///
    /// # Errors
    ///
    /// Returns a `CANParserError` if there are any errors encountered during parsing.
    ///
    /// # Example
    ///
    /// ```
    /// use can_parser::CANParser;
    ///
    /// let mut parser = CANParser::new();
    /// let lines = vec![
    ///     "can0 123#1122334455667788".to_string(),
    ///     "can0 456#1122334455667788".to_string(),
    /// ];
    /// let result = parser.parse_lines(&lines);
    /// assert!(result.is_ok());
    /// ```
    pub fn parse_lines(&mut self, lines: &Vec<String>) -> Result<(), CANParserError> {
        #[cfg(feature = "debug")]
        let start_time = Self::current_time();
        let errors = Arc::new(Mutex::new(vec![]));

        // Function to parse a line
        let parse_can_message = |line: &String| -> Option<_> {
            match Self::parse_line_inner(&self.specs, &self.line_regex, &line, &self.filtered_spec)
            {
                Ok(message) => Some(message),
                Err(e) => {
                    Self::handle_parsing_error(&self.error_handling, &errors, e, &line);
                    None
                }
            }
        };

        // Core Logic
        self.messages = if cfg!(feature = "parallel") {
            #[cfg(feature = "parallel")]
            {
                use rayon::prelude::*;
                lines.par_iter().filter_map(parse_can_message).collect()
            }
            #[cfg(not(feature = "parallel"))]
            {
                panic!("Parallel feature not enabled");
            }
        } else {
            lines.iter().filter_map(parse_can_message).collect()
        };
        // Debugging Logic
        #[cfg(feature = "debug")]
        Self::debug_log(&self.messages, Self::current_time() - start_time);

        // Error Check
        let errors = errors.lock().unwrap();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(CANParserError::ParserWarning(errors.clone()))
        }
    }

    /// Returns the current time in milliseconds as a floating-point number.
    ///
    /// If the `debug` feature is enabled, this function will return the current time
    /// using either the `web_sys::Performance` API (if the `wasm` feature is also enabled),
    /// or the `std::time::SystemTime` API (if the `wasm` feature is not enabled).
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "debug")]
    /// # mod tests {
    /// #     use super::*;
    /// #     
    /// #     #[test]
    /// #     fn test_current_time() {
    /// #         let time = current_time();
    /// #         assert!(time > 0.0);
    /// #     }
    /// # }
    /// ```
    #[cfg(feature = "debug")]
    fn current_time() -> f64 {
        #[cfg(feature = "wasm")]
        {
            use wasm_bindgen::JsCast;
            let performance = js_sys::Reflect::get(&js_sys::global(), &"performance".into())
                .expect("failed to get performance from global object")
                .unchecked_into::<web_sys::Performance>();
            performance.now()
        }
        #[cfg(not(feature = "wasm"))]
        {
            let instant = std::time::SystemTime::now();
            let since_the_epoch = instant
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Time went backwards or other time error occurred");
            since_the_epoch.as_secs() as f64 * 1000.0 + since_the_epoch.subsec_millis() as f64
        }
    }

    /// Prints debug information about the parsed CAN messages, including the file parsing duration,
    /// total lines parsed, average time per line, and average message size. The output is printed to
    /// the console if the `wasm` feature is enabled, otherwise it is printed to the standard output.
    #[cfg(feature = "debug")]
    fn debug_log(messages: &Vec<CANMessage>, elapsed_time: f64) {
        let num_messages = messages.len();
        let mut avg_time_per_message = 0;
        if num_messages > 0 {
            avg_time_per_message = ((elapsed_time * 1000000.0) as u64) / (num_messages as u64);
        }
        let avg_message_size =
            messages.iter().map(mem::size_of_val).sum::<usize>() as f32 / num_messages as f32;
        #[cfg(feature = "wasm")]
        {
            console::log_1(&"Debug Information:".into());
            console::log_1(&"------------------".into());
            console::log_1(
                &format!(
                    "File Parsing Duration: {:.2} seconds",
                    elapsed_time / 1000.0
                )
                .into(),
            );
            console::log_1(&format!("Total Lines Parsed: {}", num_messages).into());
            console::log_1(
                &format!("Average Time per Line: {:.2} ns", avg_time_per_message).into(),
            );
            console::log_1(&format!("Average Message Size: {:.2} bytes", avg_message_size).into());
            console::log_1(&"------------------".into());
        }

        #[cfg(not(feature = "wasm"))]
        {
            println!("Debug Information:");
            println!("------------------");
            println!(
                "File Parsing Duration: {:.2} seconds",
                elapsed_time / 1000.0
            );
            println!("Total Lines Parsed: {}", num_messages);
            println!("Average Time per Line: {:.2} ns", avg_time_per_message);
            println!("Average Message Size: {:.2} bytes", avg_message_size);
            println!("------------------");
        }
    }

    /// Parses a single line of CAN data and returns a `CANMessage` if successful.
    ///
    /// # Arguments
    ///
    /// * `self` - A reference to the `CANParser` instance.
    /// * `line` - A `String` containing the line of CAN data to parse.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `CANMessage` if parsing was successful, or a `CANParserError` if an error occurred.
    ///
    pub fn parse_line(&self, line: String) -> Result<CANMessage, CANParserError> {
        match Self::parse_line_inner(&self.specs, &self.line_regex, &line, &self.filtered_spec) {
            Ok(message) => Ok(message),
            Err(e) => Err(CANParserError::ParserError(format!(
                "Failed to parse line: {}",
                e
            ))),
        }
    }

    /// Handles parsing errors based on the error handling option specified.
    ///
    /// # Arguments
    ///
    /// * `error_handling` - A reference to a string containing the error handling option.
    /// * `errors` - A reference to an Arc wrapped Mutex containing a vector of error strings.
    /// * `error` - A string containing the error message.
    /// * `line` - A reference to a string containing the line where the error occurred.
    fn handle_parsing_error(
        error_handling: &String,
        errors: &Arc<Mutex<Vec<String>>>,
        error: String,
        line: &String,
    ) {
        match error_handling.as_str() {
            ERROR_IGNORE => {}
            ERROR_WARN => {
                errors.lock().unwrap().push(format!("{}: {}", line, error));
            }
            _ => {
                panic!("Unknown error handling option: {}", error_handling);
            }
        }
    }

    /// Parses a single line of CAN data and returns a `CANMessage` struct containing the parsed data.
    ///
    /// # Arguments
    ///
    /// * `annex` - An optional `Arc` reference to a `Specs` struct containing additional specifications.
    /// * `line_regex` - An `Arc` reference to a `Regex` struct used to match the line of data.
    /// * `line` - The line of data to be parsed.
    /// * `spec` - An `Arc` reference to a `FilteredSpec` struct containing the specifications for the parsed data.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `CANMessage` struct if the parsing was successful, or a `String` error message if parsing failed.
    fn parse_line_inner(
        annex: &Option<Arc<Specs>>,
        line_regex: &Arc<Regex>,
        line: &str,
        spec: &Arc<FilteredSpec>,
    ) -> Result<CANMessage, String> {
        let captures = line_regex
            .captures(line)
            .ok_or_else(|| "No captures found".to_string())?;

        let mut msg = CANMessage::default();

        if let Some(timestamp) = captures.name("timestamp") {
            msg.ts = timestamp
                .as_str()
                .parse::<f64>()
                .map_err(|_| "Failed to parse timestamp".to_string())?;
        }

        if let Some(data) = captures.name("data") {
            let data = data.as_str();
            let length = data.len().min(64);
            for i in (0..length).step_by(2) {
                msg.data.data[i / 2] = u8::from_str_radix(&data[i..i + 2], 16)
                    .map_err(|_| "Failed to parse data".to_string())?;
            }
            msg.data.len = (length / 2) as u8;
        }
        if let (Some(id), Some(ref a)) = (captures.name("id"), annex) {
            parse_id(id, &mut msg.id);
            if msg.id.flags.ext {
                let mut hit = false;
                if let Some(cache_result) = spec.j1939.read().unwrap().get(&msg.id.pgn) {
                    parse_j1939_data(&mut msg.data, &cache_result.spns);
                    hit = true;
                }
                if !hit {
                    if let Some(ref j1939) = a.j1939 {
                        match j1939.get_id_metadata(&msg.id).map_err(|e| {
                            format!("Failed to get metadata for PGN {}: {}", msg.id.pgn, e)
                        })? {
                            Metadata::J1939(aux) => {
                                // Insert aux_info using a write lock.
                                spec.j1939.write().unwrap().insert(msg.id.pgn, aux.clone());
                                parse_j1939_data(&mut msg.data, &aux.spns);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        Ok(msg)
    }

    /// Converts the CANParser object to a JSON string.
    ///
    /// # Arguments
    ///
    /// * `output_path` - An optional string representing the output file path. If provided, the JSON
    /// string will be written to the file at the specified path. If not provided, the JSON string will
    /// be returned as a Result.
    ///
    /// # Returns
    ///
    /// Returns a Result containing an optional string. If `output_path` is provided, the Result will
    /// contain `None`. If `output_path` is not provided, the Result will contain the JSON string.
    ///
    /// # Errors
    ///
    /// Returns a CANParserError if there is an error serializing the CANParser object to JSON or writing
    /// the JSON string to a file.
    pub fn to_json(&self, output_path: Option<String>) -> Result<Option<String>, CANParserError> {
        to_json(output_path, &self.filtered_spec, &self.messages)
    }

    /// Converts the filtered CAN specification and messages to a CSV format.
    ///
    /// # Arguments
    ///
    /// * `output_path` - An optional string representing the path to save the CSV file.
    ///
    /// # Returns
    ///
    /// * `Ok(None)` - If the `output_path` is provided and the CSV file is successfully saved.
    /// * `Ok(Some(csv_string))` - If the `output_path` is not provided and the CSV data is successfully combined into a single string.
    /// * `Err(CANParserError)` - If there is an error during the serialization or saving process.
    pub fn to_csv(&self, output_path: Option<String>) -> Result<Option<String>, CANParserError> {
        to_csv(output_path, &self.filtered_spec, &self.messages)
    }

    /// Writes the parsed CAN data to an SQLite database at the specified output path.
    ///
    /// # Arguments
    ///
    /// * `output_path` - A `String` representing the path to the output SQLite database.
    ///
    /// # Errors
    ///
    /// Returns a `CANParserError::Fatal` variant if the database fails to open.
    ///
    /// # Examples
    ///
    /// ```
    /// use can_parser::CANParser;
    ///
    /// let parser = CANParser::new();
    /// parser.parse("path/to/can/data.log").unwrap();
    /// parser.to_sqlite("path/to/output.db").unwrap();
    /// ```
    #[cfg(feature = "sqlite")]
    pub fn to_sqlite(&self, output_path: String) -> Result<(), CANParserError> {
        to_sqlite(output_path, &self.filtered_spec, &self.messages)
    }
}
