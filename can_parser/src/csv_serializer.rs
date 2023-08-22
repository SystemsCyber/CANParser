use crate::can_message::CANMessage;
use crate::error::CANParserError;
use crate::specification::{FilteredSpec, SpecPGN};
use csv::Writer;
use std::collections::HashMap;
use std::path::Path;

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
pub fn to_csv(
    output_path: Option<String>,
    filtered_spec: &FilteredSpec,
    messages: &Vec<CANMessage>,
) -> Result<Option<String>, CANParserError> {
    let mut csv_collection = HashMap::new();

    for (key, value) in (*filtered_spec).clone() {
        csv_collection.insert(key.to_string(), serialize_to_csv(&value)?);
    }

    csv_collection.insert(
        "messages".to_string(),
        serialize_messages_to_csv(&messages)?,
    );

    if let Some(output_path) = output_path {
        save_to_files(&csv_collection, &output_path)?;
        Ok(None)
    } else {
        let csv_string = combine_csv_strings(csv_collection)?;
        Ok(Some(csv_string))
    }
}

/// Serializes a HashMap of SpecPGN to CSV format.
///
/// # Arguments
///
/// * `data` - A reference to a HashMap of SpecPGN.
///
/// # Returns
///
/// A Result containing a vector of bytes representing the CSV content, or a CANParserError if an error occurs.
fn serialize_to_csv(data: &HashMap<u16, SpecPGN>) -> Result<Vec<u8>, CANParserError> {
    let mut wtr = Writer::from_writer(vec![]);
    let mut first = true;
    for (key, value) in data {
        let value_json = serde_json::to_value(value)?;
        let obj = value_json.as_object().ok_or_else(|| {
            CANParserError::ParserError("Failed to convert JSON to object".to_string())
        })?;
        if first {
            let mut headers = vec![];
            headers.push("id".to_string());
            for key in obj.keys() {
                headers.push(key.clone());
            }
            wtr.write_record(&headers)?;
            first = false;
        }
        let mut values = vec![];
        values.push(key.to_string());
        for value in obj.values() {
            values.push(value.to_string());
        }
        wtr.write_record(&values)?;
    }
    Ok(wtr.into_inner()?)
}

/// Serializes a vector of CAN messages to CSV format.
///
/// # Arguments
///
/// * `messages` - A vector of `CANMessage` structs to be serialized.
///
/// # Returns
///
/// Returns a `Result` containing a vector of bytes representing the CSV content if successful, or a `CANParserError` if an error occurs.
///
/// # Example
///
/// ```
/// use can_parser::CANMessage;
///
/// let messages = vec![
///     CANMessage {
///         id: 123,
///         data: vec![1, 2, 3, 4],
///         length: 4,
///         timestamp: 1234567890,
///     },
///     CANMessage {
///         id: 456,
///         data: vec![5, 6, 7, 8],
///         length: 4,
///         timestamp: 1234567900,
///     },
/// ];
///
/// let csv_content = can_parser::serialize_messages_to_csv(&messages).unwrap();
/// ```
fn serialize_messages_to_csv(messages: &Vec<CANMessage>) -> Result<Vec<u8>, CANParserError> {
    let mut wtr = Writer::from_writer(vec![]);
    let mut first = true;
    for message in messages {
        let value_json = serde_json::to_value(message)?;
        let obj = value_json.as_object().ok_or_else(|| {
            CANParserError::ParserError("Failed to convert to object".to_string())
        })?;
        if first {
            wtr.write_record(obj.keys())?;
            first = false;
        }
        wtr.write_record(obj.values().map(|v| v.to_string()))?;
    }
    Ok(wtr.into_inner()?)
}

/// Saves the given CSV data to files in the specified output directory.
///
/// # Arguments
///
/// * `csv_collection` - A HashMap containing the CSV data to be saved, with the key being the file name and the value being the CSV data.
/// * `output_path` - A string slice representing the path to the output directory where the CSV files will be saved.
///
/// # Errors
///
/// Returns a CANParserError if there was an error writing to any of the files.
///
/// # Example
///
/// ```
/// use std::collections::HashMap;
/// use can_parser::CANParserError;
///
/// let mut csv_collection = HashMap::new();
/// csv_collection.insert(String::from("file1"), vec![1, 2, 3]);
/// csv_collection.insert(String::from("file2"), vec![4, 5, 6]);
///
/// let output_path = "/home/user/output";
///
/// match save_to_files(&csv_collection, output_path) {
///     Ok(_) => println!("CSV files saved successfully!"),
///     Err(e) => println!("Error saving CSV files: {}", e),
/// }
/// ```
fn save_to_files(
    csv_collection: &HashMap<String, Vec<u8>>,
    output_path: &str,
) -> Result<(), CANParserError> {
    let output_path = Path::new(output_path);
    for (key, value) in csv_collection {
        let mod_output_path = output_path.with_file_name(&format!(
            "{}_{}.{}",
            output_path.file_stem().unwrap().to_str().unwrap(),
            key,
            output_path.extension().unwrap().to_str().unwrap()
        ));
        std::fs::write(&mod_output_path, value)?;
    }
    Ok(())
}

/// Combines a collection of CSV strings into a single string, with each CSV string
/// separated by a newline character. The resulting string is returned as a `Result`.
///
/// # Arguments
///
/// * `csv_collection` - A `HashMap` containing the CSV strings to be combined.
///
/// # Returns
///
/// Returns a `Result` containing the combined CSV strings as a `String` if successful,
/// or a `CANParserError` if an error occurs.
fn combine_csv_strings(
    mut csv_collection: HashMap<String, Vec<u8>>,
) -> Result<String, CANParserError> {
    let mut csv_string = String::new();
    for (key, value) in csv_collection.drain() {
        csv_string.push_str(&format!("{}:\n", key));
        csv_string.push_str(
            &String::from_utf8(value).map_err(|e| {
                CANParserError::ParserError(format!("Failed to convert to string: {}", e))
            })?,
        );
        csv_string.push_str("\n");
    }
    Ok(csv_string)
}
