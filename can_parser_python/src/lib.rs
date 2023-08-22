extern crate can_parser;

use can_parser::{
    CANMessage, CANParser, FileFlags, FilteredSpec, ERROR_IGNORE, ERROR_WARN, LOG_TYPE_BINARY,
    LOG_TYPE_TEXT, SPEC_TYPE_CAN, SPEC_TYPE_J1939, SPEC_TYPE_TRANSPORT, SPEC_TYPE_UDS,
};
use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::Python;
use std::collections::HashMap;
use std::sync::Arc;

/// This function is a PyO3 entry point that initializes the CANParserPython module.
/// It adds the `CANParserPython` class and several constants to the module.
#[pymodule]
fn can_parser_python(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<CANParserPython>()?;
    m.add("SPEC_TYPE_CAN", SPEC_TYPE_CAN)?;
    m.add("SPEC_TYPE_J1939", SPEC_TYPE_J1939)?;
    m.add("SPEC_TYPE_UDS", SPEC_TYPE_UDS)?;
    m.add("SPEC_TYPE_TRANSPORT", SPEC_TYPE_TRANSPORT)?;
    m.add("LOG_TYPE_TEXT", LOG_TYPE_TEXT)?;
    m.add("LOG_TYPE_BINARY", LOG_TYPE_BINARY)?;
    m.add("ERROR_IGNORE", ERROR_IGNORE)?;
    m.add("ERROR_WARN", ERROR_WARN)?;
    Ok(())
}

/// A Python wrapper for the CANParser struct.
#[pyclass]
pub struct CANParserPython {
    inner: CANParser,
}

#[pymethods]
impl CANParserPython {
    /// Creates a new instance of `CANParserPython`.
    ///
    /// # Arguments
    ///
    /// * `error_handling` - A string representing the error handling mode.
    /// * `line_regex` - An optional string representing the line regex.
    /// * `specs_annexes` - An optional hashmap containing the specs annexes.
    ///
    /// # Errors
    ///
    /// Returns a `PyValueError` if there is an error creating the `CANParser` instance.
    ///
    /// # Returns
    ///
    /// Returns a `PyResult` containing the new `CANParserPython` instance.
    #[new]
    #[pyo3(signature=(error_handling=ERROR_WARN.to_string(), line_regex=None, specs_annexes=None))]
    pub fn new(
        error_handling: String,
        line_regex: Option<String>,
        specs_annexes: Option<HashMap<String, String>>,
    ) -> PyResult<Self> {
        let inner = CANParser::new(error_handling, line_regex, specs_annexes)
            .map_err(|e| exceptions::PyValueError::new_err(format!("{}", e)))?;
        Ok(CANParserPython { inner })
    }

    /// Parses a file given its file path.
    ///
    /// # Arguments
    ///
    /// * `file_path` - A string slice that holds the path to the file to be parsed.
    ///
    /// # Returns
    ///
    /// Returns a `PyResult` that contains either `Ok(())` if the file was parsed successfully or
    /// `Err(exceptions::PyValueError)` if there was an error while parsing the file.
    pub fn parse_file(&mut self, file_path: &str) -> PyResult<()> {
        let result = self.inner.parse_file(file_path);
        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(exceptions::PyValueError::new_err(format!("{}", err))),
        }
    }

    /// Parses a vector of strings and returns a PyResult.
    ///
    /// # Arguments
    ///
    /// * `self` - A mutable reference to the CANParser instance.
    /// * `lines` - A vector of strings to be parsed.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the parsing is successful.
    /// * `Err(exceptions::PyValueError)` if there is an error during parsing.
    pub fn parse_lines(&mut self, lines: Vec<String>) -> PyResult<()> {
        let result = self.inner.parse_lines(&lines);
        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(exceptions::PyValueError::new_err(format!("{}", err))),
        }
    }

    /// Converts the CAN data to a JSON string. If the `file_path` argument is provided, the JSON string is written to the file at the specified path. Otherwise, the JSON string is returned as a `String`.
    ///
    /// # Arguments
    ///
    /// * `file_path` - An optional `String` representing the path to the file where the JSON string should be written.
    ///
    /// # Returns
    ///
    /// A `PyResult<String>` containing either the JSON string or an error message if the conversion fails.
    pub fn to_json(&mut self, file_path: Option<String>) -> PyResult<String> {
        let result = self.inner.to_json(file_path);
        match result {
            Ok(result) => Ok(result.unwrap_or("".to_string())),
            Err(err) => Err(exceptions::PyValueError::new_err(format!("{}", err))),
        }
    }

    /// Converts the CAN data to a CSV string. If the `file_path` argument is provided, the CSV string is written to the file at the specified path. Otherwise, the CSV string is returned as a `String`.
    /// 
    /// # Arguments
    /// 
    /// * `file_path` - An optional `String` representing the path to the file where the CSV string should be written.
    /// 
    /// # Returns
    /// 
    /// A `PyResult<String>` containing either the CSV string or an error message if the conversion fails.
    pub fn to_csv(&mut self, file_path: Option<String>) -> PyResult<String> {
        let result = self.inner.to_csv(file_path);
        match result {
            Ok(result) => Ok(result.unwrap_or("".to_string())),
            Err(err) => Err(exceptions::PyValueError::new_err(format!("{}", err))),
        }
    }

    /// Converts the CAN data to a SQLite database. If the `file_path` argument is provided, the database is written to the file at the specified path. Otherwise, the database is returned as a `Vec<u8>`.
    /// 
    /// # Arguments
    /// 
    /// * `file_path` - An optional `String` representing the path to the file where the database should be written.
    /// 
    /// # Returns
    /// 
    /// A `PyResult<Vec<u8>>` containing either the database or an error message if the conversion fails.
    pub fn to_sqlite(&mut self, file_path: String) -> PyResult<()> {
        let result = self.inner.to_sqlite(file_path);
        match result {
            Ok(result) => Ok(result),
            Err(err) => Err(exceptions::PyValueError::new_err(format!("{}", err))),
        }
    }

    // Getters and setters
    /// Returns a copy of the list of CAN messages.
    #[getter]
    pub fn get_messages(&self) -> PyResult<Vec<CANMessage>> {
        Ok(self.inner.messages.clone())
    }

    /// Clears all messages from the CAN parser.
    pub fn clear_messages(&mut self) -> PyResult<()> {
        self.inner.messages.clear();
        Ok(())
    }

    /// Returns a copy of the `FilteredSpec` struct that contains the current filter settings.
    #[getter]
    pub fn get_filtered_spec(&self) -> PyResult<FilteredSpec> {
        Ok((*self.inner.filtered_spec).clone())
    }

    /// Clears the filtered specification.
    pub fn clear_filtered_spec(&mut self) -> PyResult<()> {
        self.inner.filtered_spec = Arc::new(FilteredSpec::default());
        Ok(())
    }

    /// Returns the file flags.
    #[getter]
    pub fn get_flags(&self) -> PyResult<FileFlags> {
        Ok((*self.inner.flags.read().unwrap()).clone())
    }
}
