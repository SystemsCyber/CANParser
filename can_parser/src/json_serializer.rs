use crate::can_message::CANMessage;
use crate::error::CANParserError;
use crate::specification::FilteredSpec;
use serde_json::{Map, Value};
use std::fs::File;
use std::io::BufWriter;

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
pub fn to_json(
    output_path: Option<String>,
    filtered_spec: &FilteredSpec,
    messages: &Vec<CANMessage>,
) -> Result<Option<String>, CANParserError> {
    let mut json = Map::new();

    json.insert(
        "spec".to_string(),
        serde_json::to_value((filtered_spec).clone())?,
    );
    json.insert(
        "results".to_string(),
        serde_json::to_value(messages.as_slice())?,
    );

    if let Some(output_path) = output_path {
        write_json_to_file(json, output_path)
    } else {
        serde_json::to_string_pretty(&json)
            .map_err(|e| CANParserError::JsonError(e.to_string()))
            .map(|s| Some(s))
    }
}

/// Writes a JSON object to a file at the specified output path.
///
/// # Arguments
///
/// * `self` - A reference to the CANParser object.
/// * `json` - A `serde_json::Map` representing the JSON object to write to file.
/// * `output_path` - A `String` representing the path to the output file.
///
/// # Returns
///
/// Returns a `Result` containing `None` if the write operation was successful, or a `CANParserError`
/// if an error occurred while creating or writing to the file.
fn write_json_to_file(
    json: Map<String, Value>,
    output_path: String,
) -> Result<Option<String>, CANParserError> {
    let file = File::create(output_path.clone())?;
    let mut json_writer = BufWriter::new(file);

    serde_json::to_writer(&mut json_writer, &json)?;

    Ok(None)
}
