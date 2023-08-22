use std::fmt::{Display, Formatter};
use csv::{IntoInnerError, Writer};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

use crate::specification::SpecError;


/// Represents the possible errors that can occur during parsing of a CAN message.
#[derive(Debug)]
pub enum CANParserError {
    /// A fatal error occurred during parsing.
    ParserError(String),
    /// A warning occurred during parsing.
    ParserWarning(Vec<String>),
    /// An error occurred during serialization.
    SerializationError(serde_json::Error),
    /// An error occurred during input/output operations.
    IOError(std::io::Error),
    /// An error occurred during CSV parsing.
    CsvError(csv::Error),
    /// An error occurred during SQLite operations.
    #[cfg(feature = "sqlite")]
    SqliteError(rusqlite::Error),
    /// An error occurred due to an invalid specification.
    SpecError(SpecError),
    /// An error occurred during JSON parsing.
    JsonError(String),
    /// An error occurred during CSV writing.
    WriterError(IntoInnerError<Writer<Vec<u8>>>),
    /// An error occurred during regular expression parsing.
    RegexError(regex::Error),
}

impl std::error::Error for CANParserError {}

impl Display for CANParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CANParserError::ParserError(s) => write!(f, "Parser Error: {}", s),
            CANParserError::ParserWarning(v) => {
                writeln!(f, "Parser Warning:")?;
                for s in v {
                    writeln!(f, "{}", s)?;
                }
                Ok(())
            },
            CANParserError::SerializationError(e) => write!(f, "Serialization Error: {}", e),
            CANParserError::IOError(e) => write!(f, "IO Error: {}", e),
            CANParserError::CsvError(e) => write!(f, "CSV Error: {}", e),
            #[cfg(feature = "sqlite")]
            CANParserError::SqliteError(e) => write!(f, "SQLite Error: {}", e),
            CANParserError::SpecError(s) => write!(f, "Specification Error: {}", s),
            CANParserError::JsonError(s) => write!(f, "JSON Error: {}", s),
            CANParserError::WriterError(e) => write!(f, "Writer Error: {}", e),
            CANParserError::RegexError(e) => write!(f, "Regex Error: {}", e),
        }
    }
}

#[cfg(feature = "wasm")]
impl Into<JsValue> for CANParserError {
    fn into(self) -> JsValue {
        match self {
            CANParserError::ParserError(s) => s.into(),
            CANParserError::ParserWarning(v) => {
                v.into_iter().map(|s| s.to_string()).collect::<Vec<String>>().join("\n").into()
            },
            CANParserError::SerializationError(e) => e.to_string().into(),
            CANParserError::IOError(e) => e.to_string().into(),
            CANParserError::CsvError(e) => e.to_string().into(),
            #[cfg(feature = "sqlite")]
            CANParserError::SqliteError(e) => e.to_string().into(),
            CANParserError::SpecError(s) => s.to_string().into(),
            CANParserError::JsonError(s) => s.into(),
            CANParserError::WriterError(e) => e.to_string().into(),
            CANParserError::RegexError(e) => e.to_string().into(),
        }
    }
}

impl From<serde_json::Error> for CANParserError {
    fn from(err: serde_json::Error) -> CANParserError {
        CANParserError::SerializationError(err)
    }
}

impl From<std::io::Error> for CANParserError {
    fn from(err: std::io::Error) -> CANParserError {
        CANParserError::IOError(err)
    }
}

impl From<csv::Error> for CANParserError {
    fn from(err: csv::Error) -> CANParserError {
        CANParserError::CsvError(err)
    }
}

#[cfg(feature = "sqlite")]
impl From<rusqlite::Error> for CANParserError {
    fn from(err: rusqlite::Error) -> CANParserError {
        CANParserError::SqliteError(err)
    }
}

impl From<IntoInnerError<Writer<Vec<u8>>>> for CANParserError {
    fn from(err: IntoInnerError<Writer<Vec<u8>>>) -> CANParserError {
        CANParserError::WriterError(err)
    }
}

impl From<regex::Error> for CANParserError {
    fn from(err: regex::Error) -> CANParserError {
        CANParserError::RegexError(err)
    }
}

impl From<SpecError> for CANParserError {
    fn from(err: SpecError) -> CANParserError {
        CANParserError::SpecError(err)
    }
}