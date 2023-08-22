extern crate can_parser;

use can_parser::{CANParser, FileFlags, FilteredSpec};
use serde_wasm_bindgen::{from_value, to_value};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use wasm_bindgen::prelude::*;
// pub use wasm_bindgen_rayon::init_thread_pool;

/// This module contains the implementation of the `CANParserWasm` struct, which is a WebAssembly-compatible
/// wrapper around the `CANParser` struct. It provides methods for parsing lines of text, converting the parsed
/// data to JSON, and accessing and modifying various properties of the `CANParser` instance.
#[wasm_bindgen]
pub struct CANParserWasm {
    inner: CANParser,
}

/// This is a custom TypeScript section that exports constants used in the CANParser WebAssembly module.
#[wasm_bindgen(typescript_custom_section)]
const _TS_APPEND_FILE_TYPE: &'static str = r#"
export const ERROR_IGNORE = "ignore";
export const ERROR_WARN = "warn";
export const LOG_TYPE_TEXT = "text";
export const LOG_TYPE_BINARY = "binary";
export const SPEC_TYPE_CAN = "can";
export const SPEC_TYPE_J1939 = "j1939";
export const SPEC_TYPE_UDS = "uds";
export const SPEC_TYPE_TRANSPORT = "transport";
"#;

#[wasm_bindgen]
impl CANParserWasm {
    /// Constructs a new instance of `CANParserWasm` with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `error_handling` - A string that specifies the error handling mode of the parser.
    /// * `line_regex` - An optional string that specifies the regular expression used to match lines in the input.
    /// * `specs_annexes` - A `JsValue` that contains a JSON object with the specifications and annexes used by the parser.
    ///
    /// # Errors
    ///
    /// Returns a `JsError` if the creation of the `CANParser` fails.
    #[wasm_bindgen(constructor)]
    pub fn new(
        error_handling: String,
        line_regex: Option<String>,
        specs_annexes: JsValue,
    ) -> Result<CANParserWasm, JsError> {
        console_error_panic_hook::set_once();
        let specs_annexes: Option<HashMap<String, String>> =
            from_value(specs_annexes).expect("Failed to parse specs_annexes");
        let inner = CANParser::new(error_handling, line_regex, specs_annexes).map_err(|e| {
            let msg = format!("Failed to create CANParser: {}", e);
            JsError::new(&msg)
        })?;
        Ok(CANParserWasm { inner })
    }

    /// Parses a vector of strings representing lines of CAN data.
    ///
    /// # Arguments
    ///
    /// * `lines` - A `JsValue` containing a vector of strings representing lines of CAN data.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the parsing was successful, otherwise returns an error as a `JsValue`.
    pub fn parse_lines(&mut self, lines: JsValue) -> Result<(), JsValue> {
        let lines: Vec<String> = from_value(lines)?;
        let result = self.inner.parse_lines(&lines);
        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(err.into()),
        }
    }

    /// Converts the CAN message data to JSON format and writes it to a file.
    ///
    /// # Arguments
    ///
    /// * `file_path` - An optional file path to write the JSON data to. If not provided, the JSON data will be returned as a string.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the conversion and writing was successful, otherwise returns an error as a `JsValue`.
    pub fn to_json(&mut self, file_path: Option<String>) -> Result<(), JsValue> {
        let result = self.inner.to_json(file_path);
        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(err.into()),
        }
    }

    /// Converts the CAN message data to CSV format and writes it to a file.
    /// 
    /// # Arguments
    /// 
    /// * `file_path` - An optional file path to write the CSV data to. If not provided, the CSV data will be returned as a string.
    /// 
    /// # Returns
    /// 
    /// Returns `Ok(())` if the conversion and writing was successful, otherwise returns an error as a `JsValue`.
    pub fn to_csv(&mut self, file_path: Option<String>) -> Result<(), JsValue> {
        let result = self.inner.to_csv(file_path);
        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(err.into()),
        }
    }

    // Getters and setters
    /// Returns a `Result` containing a `JsValue` representation of the `messages` field of the inner `CANParser` struct.
    #[wasm_bindgen(getter)]
    pub fn messages(&self) -> Result<JsValue, serde_wasm_bindgen::Error> {
        to_value(&self.inner.messages)
    }

    /// Clears all messages from the CAN parser's message buffer.
    pub fn clear_messages(&mut self) {
        self.inner.messages.clear();
    }

    /// Returns the filtered specification as a `JsValue`.
    #[wasm_bindgen(getter)]
    pub fn filtered_spec(&self) -> Result<JsValue, serde_wasm_bindgen::Error> {
        to_value(&(*self.inner.filtered_spec))
    }

    /// Clears the filtered specification of the CAN parser.
    pub fn clear_filtered_spec(&mut self) {
        self.inner.filtered_spec = Arc::new(FilteredSpec::default());
    }

    /// Returns the flags of the CAN parser.
    #[wasm_bindgen(getter)]
    pub fn flags(&self) -> Result<JsValue, serde_wasm_bindgen::Error> {
        to_value(&(*self.inner.flags.read().unwrap()))
    }

    /// Clears the flags of the CAN parser.
    pub fn clear_flags(&mut self) {
        self.inner.flags = Arc::new(RwLock::new(FileFlags::default()));
    }
}
