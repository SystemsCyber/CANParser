use crate::can_message::CANID;
use crate::specification::{
    determine_path_and_file_type, Annex, FileType, Metadata, SpecError, SpecPGN, SpecSPN,
    Specification,
};
#[cfg(feature = "xlsx")]
use calamine::{open_workbook, DataType, Reader, Xlsx};
use can_dbc::DBC;
use serde_json::{Map, Value};
use std::fs::{read_to_string, File};
use std::io::{BufReader, Read};

/// A struct representing the J1939 specification, which includes an annex.
pub struct J1939Spec {
    pub annex: Annex,
}

impl Default for J1939Spec {
    fn default() -> Self {
        Self {
            annex: Annex::Json(Map::new()),
        }
    }
}

const PGN_DB_KEY: &str = "J1939PGNdb";
const SPN_DB_KEY: &str = "J1939SPNdb";
#[cfg(feature = "xlsx")]
const SPG_SHEET_NAME: &str = "SPs & PGs";

impl Specification for J1939Spec {
    /// Creates a new `J1939Spec` instance from a specification string or file path.
    ///
    /// # Arguments
    ///
    /// * `spec` - A string slice or file path containing the specification data.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `J1939Spec` instance if successful, or a `SpecError` if an error occurred.
    ///
    /// # Examples
    ///
    /// ```
    /// use can_parser::j1939_spec::J1939Spec;
    ///
    /// let spec_path = "/path/to/spec.json";
    /// let spec = J1939Spec::new(&spec_path);
    ///
    /// assert!(spec.is_ok());
    /// ```
    fn new(spec: &String) -> Result<Self, SpecError> {
        let (is_path, file_type) = determine_path_and_file_type(spec)?;
        match file_type {
            FileType::Json => {
                let json_str;
                if is_path {
                    json_str = read_to_string(&spec).map_err(|e| {
                        SpecError(format!("Error reading specification file: {}", e))
                    })?;
                } else {
                    json_str = spec.clone();
                }
                let json: Value = serde_json::from_str(&json_str)
                    .map_err(|e| SpecError(format!("Error parsing specification file: {}", e)))?;
                return Ok(Self {
                    annex: Annex::Json(json.as_object().unwrap().clone()),
                });
            }
            FileType::Xlsx => {
                #[cfg(feature = "xlsx")]
                {
                    if !is_path {
                        return Err(SpecError(
                            "Xlsx can only be read from file not from string.".to_string(),
                        ));
                    }
                    let mut workbook: Xlsx<_> = open_workbook(spec).map_err(|e| {
                        SpecError(format!("Could not open Xlsx Digital Annex: {}", e))
                    })?;
                    let range = workbook
                        .worksheet_range(SPG_SHEET_NAME)
                        .ok_or_else(|| {
                            SpecError(format!(
                                "Could not find {} sheet in Xlsx Digital Annex",
                                SPG_SHEET_NAME
                            ))
                        })?
                        .unwrap();
                    return Ok(J1939Spec {
                        annex: Annex::Xlsx(range),
                    });
                }
                #[cfg(not(feature = "xlsx"))]
                {
                    return Err(SpecError(
                        "Xlsx support is not enabled. Please enable the Xlsx feature.".to_string(),
                    ));
                }
            }
            FileType::Dbc => {
                let mut dbc_slice = vec![];
                if is_path {
                    let dbc_file = File::open(spec).map_err(|e| {
                        SpecError(format!("Could not open DBC Digital Annex: {}", e))
                    })?;
                    let mut dbc_reader = BufReader::new(dbc_file);
                    dbc_reader.read_to_end(&mut dbc_slice).map_err(|e| {
                        SpecError(format!("Could not read DBC Digital Annex: {}", e))
                    })?;
                } else {
                    dbc_slice = spec.as_bytes().to_vec();
                }
                let dbc_annex = DBC::from_slice(&dbc_slice)
                    .map_err(|_| SpecError("Could not parse DBC Digital Annex.".to_string()))?;
                return Ok(J1939Spec {
                    annex: Annex::Dbc(dbc_annex),
                });
            }
        }
    }

    /// Retrieves metadata for a given CAN ID by parsing the J1939 specification.
    ///
    /// # Arguments
    ///
    /// * `id` - A reference to a `CANID` struct representing the CAN ID to retrieve metadata for.
    ///
    /// # Returns
    ///
    /// * `Ok(Metadata)` - A `Metadata` enum variant containing the parsed J1939 metadata.
    /// * `Err(SpecError)` - A `SpecError` struct containing an error message if parsing fails.
    fn get_id_metadata(&self, id: &CANID) -> Result<Metadata, SpecError> {
        let mut aux_info = Metadata::J1939(SpecPGN::default());
        if let Metadata::J1939(ref mut spec_pgn) = aux_info {
            match &self.annex {
                #[cfg(feature = "xlsx")]
                Annex::Xlsx(a) => {
                    let mut got_pgn_info = false;
                    for i in a.rows() {
                        // Check if pgn is equal to any cell in the row
                        if let Some(cell) = i.get(4) {
                            if let Some(read_pgn) = cell.get_float() {
                                if i64::from(id.pgn) == (read_pgn as i64) {
                                    self.parse_row_for_pgn_info_xlsx(
                                        &mut got_pgn_info,
                                        spec_pgn,
                                        i,
                                    );
                                    spec_pgn.spns.insert(
                                        i.get(19).unwrap().get_int().unwrap_or_default() as u16,
                                        self.parse_j1939_spns_xlsx(i),
                                    );
                                    continue;
                                }
                                if got_pgn_info {
                                    break;
                                }
                            }
                        }
                    }
                }
                Annex::Json(a) => {
                    let pgn_annex = a.get(PGN_DB_KEY).ok_or_else(|| {
                        SpecError("Could not find PGN database in JSON Digital Annex".to_string())
                    })?;
                    let pgn_annex = pgn_annex.as_object().ok_or_else(|| {
                        SpecError("Could not find PGN database in JSON Digital Annex".to_string())
                    })?;
                    if let Some(pgn_data) = pgn_annex.get(id.pgn.to_string().as_str()) {
                        self.parse_row_for_pgn_info_json(pgn_data, spec_pgn);
                        let spns = pgn_data.get("SPNs").unwrap().as_array().unwrap();
                        let spn_annex: &Value = a.get(SPN_DB_KEY).ok_or_else(|| {
                            SpecError(
                                "Could not find SPN database in JSON Digital Annex".to_string(),
                            )
                        })?;
                        let spn_annex = spn_annex.as_object().ok_or_else(|| {
                            SpecError(
                                "Could not find SPN database in JSON Digital Annex".to_string(),
                            )
                        })?;
                        if let Some(spn_start_bit) = pgn_data.get("SPNStartBits") {
                            self.parse_j1939_spns_json(
                                &spn_annex,
                                spec_pgn,
                                spns,
                                spn_start_bit.as_array().unwrap(),
                            );
                        } else {
                            self.parse_j1939_spns_json(&spn_annex, spec_pgn, spns, &Vec::new());
                        }
                    }
                }
                _ => {
                    return Err(SpecError("Annex type not supported".to_string()));
                }
            }
        }
        return Ok(aux_info);
    }
}

impl J1939Spec {
    /// Parses a row of PGN information in JSON format and updates the `SpecPGN` struct with the parsed information.
    ///
    /// # Arguments
    ///
    /// * `pgn_data` - A reference to a `Value` object containing the PGN information in JSON format.
    /// * `aux_info` - A mutable reference to a `SpecPGN` object that will be updated with the parsed information.
    fn parse_row_for_pgn_info_json(&self, pgn_data: &Value, aux_info: &mut SpecPGN) {
        self.string_to_slice(
            pgn_data.get("Name").unwrap().to_owned().to_string(),
            &mut aux_info.label,
            32,
        );
        self.string_to_slice(
            pgn_data.get("Label").unwrap().to_owned().to_string(),
            &mut aux_info.acronym,
            10,
        );
        aux_info.length = pgn_data
            .get("PGNLength")
            .unwrap()
            .as_str()
            .unwrap()
            .parse()
            .unwrap_or_default();
        self.string_to_slice(
            pgn_data.get("Rate").unwrap().to_owned().to_string(),
            &mut aux_info.transmission_rate,
            50,
        );
    }

    /// Parses a row for PGN information from an xlsx file.
    ///
    /// # Arguments
    ///
    /// * `got_pgn_info` - A mutable reference to a boolean indicating whether PGN information has been obtained.
    /// * `aux_info` - A mutable reference to a `SpecPGN` struct containing auxiliary PGN information.
    /// * `i` - A slice of `DataType` containing the row data to be parsed.
    #[cfg(feature = "xlsx")]
    fn parse_row_for_pgn_info_xlsx(
        &self,
        got_pgn_info: &mut bool,
        aux_info: &mut SpecPGN,
        i: &[DataType],
    ) {
        if !got_pgn_info.to_owned() {
            self.string_to_slice(
                i.get(5).unwrap().get_string().unwrap().to_owned(),
                &mut aux_info.label,
                32,
            );
            self.string_to_slice(
                i.get(6).unwrap().get_string().unwrap().to_owned(),
                &mut aux_info.acronym,
                10,
            );
            aux_info.description = i.get(7).unwrap().get_string().unwrap_or_default().to_owned();
            aux_info.pdu_format = i.get(10).unwrap().get_int().unwrap_or_default() as u8;
            aux_info.pdu_specific = i.get(11).unwrap().get_int().unwrap_or_default() as u8;
            aux_info.priority = i.get(15).unwrap().get_int().unwrap_or_default() as u8;
            aux_info.length = i.get(14).unwrap().get_int().unwrap_or_default() as u8;
            self.string_to_slice(
                i.get(13).unwrap().get_string().unwrap_or_default().to_owned(),
                &mut aux_info.transmission_rate,
                50,
            );
            *got_pgn_info = true;
        }
    }

    /// Parses a list of J1939 SPNs in JSON format and updates the provided `SpecPGN` object with the parsed information.
    ///
    /// # Arguments
    ///
    /// * `annex` - A reference to a `Map<String, Value>` object containing the annex information for the SPNs.
    /// * `aux_info` - A mutable reference to a `SpecPGN` object that will be updated with the parsed SPN information.
    /// * `spns` - A reference to a `Vec<Value>` object containing the SPNs to be parsed.
    /// * `spn_start_bit` - A reference to a `Vec<Value>` object containing the start bit information for each SPN.
    fn parse_j1939_spns_json(
        &self,
        annex: &Map<String, Value>,
        aux_info: &mut SpecPGN,
        spns: &Vec<Value>,
        spn_start_bit: &Vec<Value>,
    ) {
        for (spn, start_bit) in spns.iter().zip(spn_start_bit.iter()) {
            let spn_name = spn.to_string();
            let start;
            if let Some(start_t) = start_bit.as_array() {
                start = start_t.get(0).unwrap().as_i64().unwrap_or_default();
            } else {
                start = start_bit.as_i64().unwrap_or_default();
            }
            let spn_t;
            if let Some(spn_tt) = annex.get(&spn_name) {
                spn_t = spn_tt.as_object().unwrap_or_else(|| {
                    panic!("SPN {} not object in J1939SPNdb", spn_name.as_str())
                });
            } else {
                continue;
            }
            let mut spn = SpecSPN {
                label: [0u8; 32],
                description: "".to_string(),
                units: [0u8; 10],
                length: spn_t.get("SPNLength").unwrap().as_i64().unwrap_or_default() as u8,
                resolution: spn_t
                    .get("Resolution")
                    .unwrap()
                    .as_f64()
                    .unwrap_or_default() as f32,
                offset: spn_t.get("Offset").unwrap().as_f64().unwrap_or_default() as f32,
                max: spn_t
                    .get("OperationalHigh")
                    .unwrap()
                    .as_f64()
                    .unwrap_or_default() as f32,
                start_bit: 0,
                spn_type: [0; 8],
            };
            self.string_to_slice(
                spn_t
                    .get("Name")
                    .unwrap()
                    .as_str()
                    .unwrap_or_default()
                    .to_owned(),
                &mut spn.label,
                32,
            );
            self.string_to_slice(
                spn_t
                    .get("Units")
                    .unwrap()
                    .as_str()
                    .unwrap_or_default()
                    .to_owned(),
                &mut spn.units,
                10,
            );
            if start >= 0 {
                spn.start_bit = start as u8;
            }
            aux_info
                .spns
                .insert(spn_name.parse::<u16>().unwrap_or_default(), spn);
        }
    }

    /// Parses J1939 SPNs from an XLSX file.
    #[cfg(feature = "xlsx")]
    fn parse_j1939_spns_xlsx(&self, i: &[DataType]) -> SpecSPN {
        let mut spn = SpecSPN {
            label: [0u8; 32],
            description: i
                .get(21)
                .unwrap()
                .get_string()
                .unwrap_or_default()
                .to_owned(),
            units: [0u8; 10],
            length: i.get(35).unwrap().get_int().unwrap_or_default() as u8,
            resolution: i.get(32).unwrap().get_float().unwrap_or_default() as f32,
            offset: i.get(33).unwrap().get_float().unwrap_or_default() as f32,
            max: i.get(34).unwrap().get_float().unwrap_or_default() as f32,
            start_bit: 0,
            spn_type: [0u8; 8],
        };
        self.string_to_slice(
            i.get(20)
                .unwrap()
                .get_string()
                .unwrap_or_default()
                .to_owned(),
            &mut spn.label,
            32,
        );
        self.string_to_slice(
            i.get(27)
                .unwrap()
                .get_string()
                .unwrap_or_default()
                .to_owned(),
            &mut spn.units,
            10,
        );
        self.string_to_slice(
            i.get(30)
                .unwrap()
                .get_string()
                .unwrap_or_default()
                .to_owned(),
            &mut spn.spn_type,
            8,
        );
        let start_bit = i.get(18).unwrap().get_float().unwrap_or_default();
        if start_bit != 0.0 {
            spn.start_bit = self.start_bit_to_offset(start_bit);
        }
        return spn;
    }

    /// Calculates the bit offset of a given start bit.
    ///
    /// # Arguments
    ///
    /// * `start_bit` - The start bit of the signal.
    ///
    /// # Returns
    ///
    /// The bit offset of the given start bit.
    #[cfg(feature = "xlsx")]
    fn start_bit_to_offset(&self, start_bit: f64) -> u8 {
        let byte_offset = (start_bit.trunc() as u8) - 1;
        let bit_offset = ((start_bit.fract() * 8.0).round() as u8) - 1;
        byte_offset * 8 + bit_offset
    }

    /// Converts a given string to a byte slice of specified length, with additional modifications if necessary.
    ///
    /// # Arguments
    ///
    /// * `input` - A string to be converted to a byte slice.
    /// * `output` - A mutable byte slice to store the converted string.
    /// * `len` - The length of the byte slice.
    ///
    /// # Panics
    ///
    /// This function will panic if the length of the byte slice is less than the specified length.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut output = [0; 8];
    /// let input = String::from("Hello, World!");
    /// let len = 8;
    /// string_to_slice(&input, &mut output, len);
    /// assert_eq!(output, [72, 101, 108, 108, 111, 44, 32, 0]);
    /// ```
    fn string_to_slice(&self, input: String, output: &mut [u8], len: usize) {
        assert!(
            len <= output.len(),
            "Length must be less than or equal to slice size"
        );
        let mut input_chars = input.as_bytes().to_vec();
        if input == input.to_uppercase() {
            // If all uppercase, just truncate and copy to output.
            input_chars.truncate(len);
        } else {
            // If not all uppercase, first remove spaces.
            input_chars.retain(|&c| c != b' ');
            if input_chars.len() > len {
                // If still too long, remove vowels.
                input_chars.retain(|&c| {
                    !matches!(
                        c,
                        b'a' | b'e' | b'i' | b'o' | b'u' | b'A' | b'E' | b'I' | b'O' | b'U'
                    )
                });
                if input_chars.len() > len {
                    // If still too long, truncate.
                    input_chars.truncate(len);
                }
            }
        }
        // Ensure remaining items in the array are spaces
        for i in &mut output[input_chars.len()..len] {
            *i = b' ';
        }
        output[..input_chars.len()].copy_from_slice(&input_chars);
    }
}
