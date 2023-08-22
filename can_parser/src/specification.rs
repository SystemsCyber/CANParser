use crate::can_message::CANID;
use crate::utils::process_string;
#[cfg(feature = "xlsx")]
use calamine::{DataType, Range};
use can_dbc::DBC;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, RwLock};

// Helpers for serialization
/// Serializes a u8 array to a string using the given serializer.
/// If the array is all zeros, an empty string is returned.
/// Otherwise, the array is converted to a string using UTF-8 encoding and any trailing whitespace is trimmed.
///
/// # Arguments
///
/// * `array` - The u8 array to serialize.
/// * `serializer` - The serializer to use for serialization.
///
/// # Returns
///
/// Returns a Result containing the serialized string if successful, or an error if serialization fails.
fn serialize_u8_array<S, const N: usize>(array: &[u8; N], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if array.iter().all(|&x| x == 0) {
        return serializer.serialize_str("");
    }
    let str = std::str::from_utf8(array).map_err(serde::ser::Error::custom)?;
    serializer.serialize_str(str.trim())
}

// Helpers for deserialization
/// Deserialize a string into a fixed-size array of u8 values.
///
/// # Arguments
///
/// * `deserializer` - The deserializer to use.
///
/// # Type Parameters
///
/// * `N` - The size of the array.
///
/// # Returns
///
/// Returns a Result containing the deserialized array of u8 values.
///
/// # Errors
///
/// Returns an error if the deserialization fails.
fn deserialize_u8_array<'de, D, const N: usize>(deserializer: D) -> Result<[u8; N], D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let processed = process_string(&s, N);

    let mut array = [0u8; N];
    let bytes = processed.as_bytes();
    array[..bytes.len()].copy_from_slice(bytes);
    Ok(array)
}

/// Struct representing a Signal Parameter Name (SPN) specification.
#[derive(Clone, Serialize, Deserialize)]
pub struct SpecSPN {
    /// Label of the SPN.
    #[serde(
        serialize_with = "serialize_u8_array",
        deserialize_with = "deserialize_u8_array"
    )]
    pub label: [u8; 32],
    /// Description of the SPN.
    pub description: String,
    /// Units of the SPN.
    #[serde(
        serialize_with = "serialize_u8_array",
        deserialize_with = "deserialize_u8_array"
    )]
    pub units: [u8; 10],
    /// Length of the SPN in bits.
    pub length: u8,
    /// Resolution of the SPN.
    pub resolution: f32,
    /// Offset of the SPN.
    pub offset: f32,
    /// Maximum value of the SPN.
    pub max: f32,
    /// Starting bit of the SPN.
    pub start_bit: u8,
    /// Type of the SPN.
    #[serde(
        serialize_with = "serialize_u8_array",
        deserialize_with = "deserialize_u8_array"
    )]
    pub spn_type: [u8; 8],
}

impl Default for SpecSPN {
    fn default() -> Self {
        Self {
            label: [0; 32],
            description: "".to_string(),
            units: [0; 10],
            length: 0,
            resolution: 0.0,
            offset: 0.0,
            max: 0.0,
            start_bit: 0,
            spn_type: [0; 8],
        }
    }
}

/// A struct representing a Parameter Group Number (PGN) specification.
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[derive(Clone, Serialize, Deserialize)]
pub struct SpecPGN {
    /// The label of the PGN.
    #[serde(
        serialize_with = "serialize_u8_array",
        deserialize_with = "deserialize_u8_array"
    )]
    pub label: [u8; 32],
    /// The acronym of the PGN.
    #[serde(
        serialize_with = "serialize_u8_array",
        deserialize_with = "deserialize_u8_array"
    )]
    pub acronym: [u8; 10],
    /// The description of the PGN.
    pub description: String,
    /// The PDU format of the PGN.
    pub pdu_format: u8,
    /// The PDU specific of the PGN.
    pub pdu_specific: u8,
    /// The priority of the PGN.
    pub priority: u8,
    /// The length of the PGN.
    pub length: u8,
    /// The transmission rate of the PGN.
    #[serde(
        serialize_with = "serialize_u8_array",
        deserialize_with = "deserialize_u8_array"
    )]
    pub transmission_rate: [u8; 50],
    /// A HashMap containing the SPNs (Suspect Parameter Numbers) of the PGN.
    pub spns: HashMap<u16, SpecSPN>,
}

impl Default for SpecPGN {
    fn default() -> Self {
        Self {
            label: [0; 32],
            acronym: [0; 10],
            description: "".to_string(),
            pdu_format: 0,
            pdu_specific: 0,
            priority: 0,
            length: 0,
            transmission_rate: [0; 50],
            spns: HashMap::new(),
        }
    }
}

pub enum Annex {
    Json(Map<String, Value>),
    #[cfg(feature = "xlsx")]
    Xlsx(Range<DataType>),
    Dbc(DBC),
}

pub enum Metadata {
    J1939(SpecPGN),
    CAN(u32),
    UDS(u32),
    Transport(u32),
}

impl Default for Metadata {
    fn default() -> Self {
        Self::CAN(0)
    }
}

#[derive(Debug)]
pub struct SpecError(pub String);

impl std::fmt::Display for SpecError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "SpecError: {}", self.0)
    }
}

impl std::error::Error for SpecError {}

pub enum FileType {
    Json,
    Xlsx,
    Dbc,
}

/// A trait for defining a CAN specification.
pub trait Specification {
    /// Creates a new instance of the specification.
    ///
    /// # Arguments
    ///
    /// * `spec` - A string slice that contains the specification.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new instance of the specification or a `SpecError` if the
    /// specification is invalid.
    fn new(spec: &String) -> Result<Self, SpecError>
    where
        Self: Sized;

    /// Gets the metadata for a given CAN ID.
    ///
    /// # Arguments
    ///
    /// * `id` - A reference to the CAN ID.
    ///
    /// # Returns
    ///
    /// A `Result` containing the metadata for the given CAN ID or a `SpecError` if the ID is
    /// not found in the specification.
    fn get_id_metadata(&self, id: &CANID) -> Result<Metadata, SpecError>;

    // fn get_session_metadata(&self, session: &Vec<CANMessage>) -> Result<Metadata, SpecError>;
}

/// Determines the path and file type of a given string.
///
/// # Arguments
///
/// * `s` - A string slice that represents the path or contents of a file.
///
/// # Returns
///
/// A `Result` containing a tuple of a boolean and a `FileType` enum variant, or a `SpecError` if an error occurs.
///
/// The boolean indicates whether the input string is a file path or file contents.
///
/// The `FileType` enum variant represents the type of file, which can be one of the following:
///
/// * `Json` - A JSON file.
/// * `Xlsx` - An Excel file.
/// * `Dbc` - A CAN database file.
///
/// # Examples
///
/// ```
/// use can_parser::specification::{determine_path_and_file_type, FileType};
///
/// let file_path = "/home/user/data.json";
/// let file_contents = "{\"name\": \"John\", \"age\": 30, \"city\": \"New York\"}";
///
/// let result = determine_path_and_file_type(file_path);
/// assert_eq!(result.unwrap(), (true, FileType::Json));
///
/// let result = determine_path_and_file_type(file_contents);
/// assert_eq!(result.unwrap(), (false, FileType::Json));
/// ```
pub fn determine_path_and_file_type(s: &str) -> Result<(bool, FileType), SpecError> {
    let mut is_path = false;
    if s.len() <= 256 {
        let file_path_regex = Regex::new(
            r"^(?-u:(?-u:[\w \.-]|[\\/]){0,2}:?[\\/]/?)(?-u:[\w \.-]+[\\/])*(?-u:[\w \.-])*$",
        )
        .map_err(|e| SpecError(e.to_string()))?;
        is_path = file_path_regex.is_match(s);
    }
    if is_path {
        if Path::new(s).is_file() {
            let ext = Path::new(s).extension().and_then(std::ffi::OsStr::to_str);

            match ext {
                Some("json") => Ok((true, FileType::Json)),
                Some("xlsx") => Ok((true, FileType::Xlsx)),
                Some("dbc") => Ok((true, FileType::Dbc)),
                None => {
                    let mut file = File::open(s).map_err(|e| SpecError(e.to_string()))?;
                    let mut buffer = [0; 5];
                    file.read(&mut buffer)
                        .map_err(|e| SpecError(e.to_string()))?;
                    let contents = String::from_utf8_lossy(&buffer);
                    let file_type = determine_file_type_from_contents(&contents)?;
                    Ok((true, file_type))
                }
                _ => Err(SpecError("Unsupported specification file type".to_string())),
            }
        } else {
            Err(SpecError("Specification file does not exist".to_string()))
        }
    } else {
        let file_type = determine_file_type_from_contents(s)?;
        Ok((false, file_type))
    }
}

/// Determines the file type of a specification file based on its contents.
///
/// # Arguments
///
/// * `contents` - A string slice that contains the contents of the specification file.
///
/// # Returns
///
/// Returns a `Result` enum that contains the `FileType` if the file type is supported, or a `SpecError` if the file type is not supported.
///
/// # Examples
///
/// ```
/// use can_parser::specification::{determine_file_type_from_contents, FileType};
///
/// let contents = "{ \"name\": \"example\", \"version\": 1 }";
/// let file_type = determine_file_type_from_contents(contents).unwrap();
/// assert_eq!(file_type, FileType::Json);
/// ```
pub fn determine_file_type_from_contents(contents: &str) -> Result<FileType, SpecError> {
    if contents.starts_with("{") {
        Ok(FileType::Json)
    } else if contents.starts_with("PK") {
        Ok(FileType::Xlsx)
    } else if contents.starts_with("VERSION") {
        Ok(FileType::Dbc)
    } else {
        Err(SpecError("Unsupported specification file type".to_string()))
    }
}


/// A struct representing a filtered specification, containing a mapping of J1939 PGNs to their corresponding `SpecPGN`.
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[derive(Clone, Serialize, Deserialize)]
pub struct FilteredSpec {
    pub j1939: Arc<RwLock<HashMap<u16, SpecPGN>>>,
}

impl Default for FilteredSpec {
    fn default() -> Self {
        Self {
            j1939: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}