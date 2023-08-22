use crate::specification::SpecSPN;
use crate::utils::process_string;
#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::PyDict;
use regex::Match;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

// const CAN_EFF_FLAG: u32 = 0x80000000;
pub const CAN_SFF_MASK: u32 = 0x000007FF;
pub const CAN_RTR_FLAG: u32 = 0x40000000;
pub const CAN_ERR_FLAG: u32 = 0x20000000;
pub const PRIORITY_MASK: u32 = 0x1C000000;
pub const PRIORITY_SHIFT: u32 = 26;
pub const PDU_FORMAT_MASK: u32 = 0x3FF0000;
pub const PDU_FORMAT_SHIFT: u16 = 16;
pub const PDU_SPECIFIC_MASK: u32 = 0xFF00;
pub const PDU_SPECIFIC_SHIFT: u16 = 8;

/// Represents the flags associated with a CAN message.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CANMessageFlags {
    /// Indicates whether the message has an extended identifier.
    pub ext: bool,
    /// Indicates whether the message is an error frame.
    pub err: bool,
    /// Indicates whether the message is a remote transmission request.
    pub rtr: bool,
}

/// A struct representing a CAN message's data.
#[derive(Debug, Clone)]
pub struct CANData {
    /// The length of the data in bytes.
    pub len: u8,
    /// The data bytes.
    pub data: [u8; 64],
    /// A HashMap containing the Signal Parameter Names (SPNs) and their corresponding values.
    pub spns: HashMap<u16, f32>,
}

impl Serialize for CANData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("CANData", 3)?;
        state.serialize_field("len", &self.len)?;
        //if array is all zeros, return empty string
        if self.data.iter().all(|&x| x == 0) {
            state.serialize_field("data", &"")?;
        } else {
            state.serialize_field("data", &hex::encode_upper(&self.data[..self.len as usize]))?;
        }
        let mut map = self.spns.clone();
        for (_, v) in map.iter_mut() {
            *v = (*v * 1000.0).round() / 1000.0;
        }
        state.serialize_field("spns", &map)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for CANData {
    fn deserialize<D>(deserializer: D) -> Result<CANData, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        use serde::de::MapAccess;
        use serde::de::Visitor;
        use std::fmt;

        struct CANDataVisitor;

        impl<'de> Visitor<'de> for CANDataVisitor {
            type Value = CANData;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct CANData")
            }

            fn visit_map<V>(self, mut map: V) -> Result<CANData, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut len = None;
                let mut data = None;
                let mut spns = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        "len" => {
                            if len.is_some() {
                                return Err(Error::duplicate_field("len"));
                            }
                            len = Some(map.next_value()?);
                        }
                        "data" => {
                            if data.is_some() {
                                return Err(Error::duplicate_field("data"));
                            }
                            let processed = process_string(map.next_value()?, 64);

                            let array: [u8; 64] = processed
                                .as_bytes()
                                .chunks(2)
                                .map(|b| {
                                    u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap()
                                })
                                .collect::<Vec<u8>>()
                                .try_into()
                                .unwrap();
                            data = Some(array);
                        }
                        "spns" => {
                            if spns.is_some() {
                                return Err(Error::duplicate_field("spns"));
                            }
                            spns = Some(map.next_value()?);
                        }
                        _ => {
                            return Err(Error::unknown_field(key, &["len", "data", "spns"]));
                        }
                    }
                }
                let len = len.ok_or_else(|| Error::missing_field("len"))?;
                let data = data.ok_or_else(|| Error::missing_field("data"))?;
                let spns = spns.ok_or_else(|| Error::missing_field("spns"))?;
                Ok(CANData { len, data, spns })
            }
        }

        const FIELDS: &[&str] = &["len", "data", "spns"];
        deserializer.deserialize_struct("CANData", FIELDS, CANDataVisitor)
    }
}

#[cfg(feature = "python")]
impl FromPyObject<'_> for CANData {
    fn extract(ob: &'_ PyAny) -> PyResult<Self> {
        let dict = ob.downcast::<PyDict>()?;
        Ok(CANData {
            len: dict.get_item("len").unwrap().extract()?,
            // data: smallvec_from_py(dict.get_item("data").unwrap())?.try_into().unwrap(),
            data: dict.get_item("data").unwrap().extract()?,
            spns: dict.get_item("spns").unwrap().extract()?,
        })
    }
}

/// A struct representing a CAN message ID.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CANID {
    /// The message ID.
    #[serde(
        serialize_with = "serialize_id",
        deserialize_with = "deserialize_id::<__D, 8>"
    )]
    pub id: u32,
    /// The message priority.
    pub pri: u8,
    /// The message destination address.
    pub da: u8,
    /// The message source address.
    pub sa: u8,
    /// The message PGN.
    pub pgn: u16,
    /// The message flags.
    pub flags: CANMessageFlags,
}

/// Serializes a CAN message ID to a string in hexadecimal format.
///
/// # Arguments
///
/// * `id` - A reference to the CAN message ID to be serialized.
/// * `serializer` - The serializer to use for serializing the ID.
///
/// # Returns
///
/// Returns a `Result` containing the serialized ID if successful, or an error if serialization fails.
///
/// # Examples
///
/// ```
/// use serde::Serializer;
/// use can_parser::can_message::serialize_id;
///
/// let id = 0x12345678;
/// let mut serializer = serde_json::Serializer::new(Vec::new());
/// let result = serialize_id(&id, &mut serializer);
/// assert_eq!(result.unwrap(), "\"12345678\"");
/// ```
fn serialize_id<S>(id: &u32, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("{:08X}", id))
}

/// Deserialize a CAN message ID from a string representation in hexadecimal format.
///
/// # Arguments
///
/// * `deserializer` - The deserializer to use for deserializing the ID.
///
/// # Returns
///
/// The deserialized ID as a `u32`.
///
/// # Errors
///
/// Returns an error if the deserialization fails.
fn deserialize_id<'de, D, const N: usize>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    use serde::de::Visitor;
    use std::fmt;

    struct CANIDVisitor;

    impl<'de> Visitor<'de> for CANIDVisitor {
        type Value = u32;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("struct CANID")
        }

        fn visit_str<E>(self, value: &str) -> Result<u32, E>
        where
            E: Error,
        {
            u32::from_str_radix(value, 16).map_err(Error::custom)
        }
    }

    deserializer.deserialize_str(CANIDVisitor)
}

/// A struct representing a CAN message.
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CANMessage {
    /// Timestamp of the message.
    pub ts: f64,
    /// ID of the message.
    pub id: CANID,
    /// Data of the message.
    pub data: CANData,
}

impl Default for CANMessage {
    fn default() -> Self {
        Self {
            ts: 0.0,
            id: CANID {
                id: 0,
                pri: 0,
                da: 0,
                sa: 0,
                pgn: 0,
                flags: CANMessageFlags {
                    ext: false,
                    err: false,
                    rtr: false,
                },
            },
            data: CANData {
                len: 0,
                data: [0; 64],
                spns: HashMap::with_capacity(0),
            },
        }
    }
}

/// Parses the given `Match` into a `CANID` struct.
///
/// # Arguments
///
/// * `id` - A `Match` object representing the ID to be parsed.
/// * `can_id` - A mutable reference to a `CANID` struct where the parsed ID will be stored.
pub fn parse_id(id: Match, can_id: &mut CANID) {
    // let extended = (id & CAN_EFF_FLAG) == CAN_EFF_FLAG; // Not working
    can_id.id = u32::from_str_radix(id.as_str(), 16).unwrap();
    can_id.flags.ext = if can_id.id > CAN_SFF_MASK {
        true
    } else {
        false
    };
    can_id.flags.err = (can_id.id & CAN_ERR_FLAG) == CAN_ERR_FLAG;
    can_id.flags.rtr = (can_id.id & CAN_RTR_FLAG) == CAN_RTR_FLAG;
    if can_id.flags.ext {
        parse_j1939_id(can_id);
    }
}

/// Parses a J1939 CAN ID and updates the fields of the given `CANID` struct accordingly.
pub fn parse_j1939_id(can_id: &mut CANID) {
    // TODO: Confirm that this bit shift works correctly.
    can_id.pri = ((can_id.id & PRIORITY_MASK) >> PRIORITY_SHIFT) as u8;
    let pdu_fmt = ((can_id.id & PDU_FORMAT_MASK) >> PDU_FORMAT_SHIFT) as u16;
    let pdu_spec = ((can_id.id & PDU_SPECIFIC_MASK) >> PDU_SPECIFIC_SHIFT) as u8;
    can_id.sa = (can_id.id & 0xFF) as u8;
    can_id.da = 255;
    // can_id.pdu_type = 1;
    if pdu_fmt >= 240 {
        // can_id.pdu_type = 2;
        can_id.pgn = (pdu_fmt << 8) + (pdu_spec) as u16;
    } else {
        can_id.pgn = pdu_fmt << 8;
        can_id.da = pdu_spec;
    }
}

/// Parses J1939 data from a CAN message and populates the given `CANData` struct with the parsed SPNs.
///
/// # Arguments
///
/// * `data` - A mutable reference to a `CANData` struct to populate with the parsed SPNs.
/// * `spn_info` - A reference to a `HashMap` containing information about the SPNs to parse.
pub fn parse_j1939_data(data: &mut CANData, spn_info: &HashMap<u16, SpecSPN>) {
    data.spns.reserve(spn_info.len());
    for (spn, spec) in spn_info {
        data.spns.insert(
            *spn,
            parse_j1939_data_inner(
                &mut data.data,
                spec.start_bit,
                spec.length,
                spec.resolution,
                spec.offset,
                spec.max,
            ),
        );
    }
}

/// Parses J1939 data from a slice of bytes.
///
/// # Arguments
///
/// * `data` - A mutable reference to a slice of bytes containing the data to be parsed.
/// * `start_bit` - The starting bit position of the data to be parsed.
/// * `length` - The length of the data to be parsed in bits.
/// * `scaling` - The scaling factor to be applied to the parsed value.
/// * `offset` - The offset to be applied to the parsed value.
/// * `max` - The maximum value that the parsed value can have.
///
/// # Returns
///
/// The parsed value as a `f32`.
fn parse_j1939_data_inner(
    data: &mut [u8],
    start_bit: u8,
    length: u8,
    scaling: f32,
    offset: f32,
    max: f32,
) -> f32 {
    let mut value = 0.0;
    let len = if ((data.len() - 1) as u8) < length {
        (data.len() - 1) as u8
    } else {
        length
    };
    for i in 0..len {
        let byte = (start_bit + i) / 8;
        let bit = (start_bit + i) % 8;
        let mut bit_value = 0;
        if byte <= len {
            bit_value = (data.get(byte as usize).unwrap() >> bit) & 1;
        }
        value += (bit_value as f32) * 2.0f32.powf(i as f32);
    }
    value = value * scaling + offset;
    if value > max {
        value -= max;
    }
    return value;
}
