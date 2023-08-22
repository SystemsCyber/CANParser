use crate::can_message::{CANMessage, CANID};
use crate::error::CANParserError;
use crate::specification::{SpecPGN, SpecSPN, FilteredSpec};
use rusqlite::{params, Connection, DatabaseName};
use std::path::Path;

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
pub fn to_sqlite(
    output_path: String,
    filtered_spec: &FilteredSpec,
    messages: &Vec<CANMessage>,
) -> Result<(), CANParserError> {
    let conn = Connection::open_in_memory()?;
    create_sqlite_tables(&conn)?;
    for (pgn, pgn_data) in filtered_spec.j1939.read().unwrap().clone().into_iter() {
        insert_spec_pgn(&conn, pgn, &pgn_data)?;
        for (spn, spn_data) in pgn_data.spns {
            insert_spec_spn(&conn, pgn, spn, &spn_data)?;
        }
    }
    for message in messages.clone() {
        insert_canid(&conn, &message.id)?;
        insert_message(&conn, &message)?;
    }
    conn.backup(DatabaseName::Main, Path::new(&output_path), None)?;
    Ok(())
}

/// Creates SQLite tables for storing CAN messages, PGNs, SPNs, and CAN IDs.
///
/// # Arguments
///
/// * `conn` - A reference to a SQLite `Connection` object.
///
/// # Errors
///
/// This function returns a `CANParserError` if any of the SQLite table creation queries fail.
///
/// # Example
///
/// ```
/// use can_parser::CANParser;
///
/// let parser = CANParser::new();
/// let conn = parser.connect_to_database().unwrap();
/// parser.create_sqlite_tables(&conn).unwrap();
/// ```
pub fn create_sqlite_tables(conn: &Connection) -> Result<(), CANParserError> {
    // create table for specPGN
    conn.execute(
        "CREATE TABLE IF NOT EXISTS SpecPGNs (
                id INTEGER PRIMARY KEY,
                label TEXT,
                acronym TEXT,
                description TEXT,
                pdu_format INTEGER,
                pdu_specific INTEGER,
                priority INTEGER,
                length INTEGER,
                transmission_rate TEXT
            )",
        [],
    )?;
    // create table for specSPN
    conn.execute(
        "CREATE TABLE IF NOT EXISTS SpecSPNs (
                id INTEGER PRIMARY KEY,
                pgn REFERENCES SpecPGNs(id),
                label TEXT,
                description TEXT,
                units TEXT,
                length INTEGER,
                resolution REAL,
                offset REAL,
                maximum REAL,
                start_bit INTEGER,
                spn_type TEXT
            )",
        [],
    )?;
    // create table for CANID
    conn.execute(
        "CREATE TABLE IF NOT EXISTS CANIDs (
                id INTEGER PRIMARY KEY,
                pgn REFERENCES SpecPGNs(id),
                priority INTEGER,
                destination_address INTEGER,
                source_address INTEGER,
                extended INTEGER,
                error INTEGER,
                rtr INTEGER
            )",
        [],
    )?;
    // create table for messages
    conn.execute(
        "CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp REAL,
                can_id REFERENCES CANIDs(id),
                length INTEGER,
                data BLOB,
                spn_values TEXT
            )",
        [],
    )?;
    Ok(())
}

/// Inserts a new SpecPGN into the database.
///
/// # Arguments
///
/// * `conn` - A reference to a SQLite `Connection` object.
/// * `pgn` - The PGN number to insert.
/// * `pgn_data` - A reference to a `SpecPGN` struct containing the data to insert.
///
/// # Returns
///
/// Returns `Ok(())` if the insertion was successful, otherwise returns a `CANParserError`.
pub fn insert_spec_pgn(conn: &Connection, pgn: u16, pgn_data: &SpecPGN) -> Result<(), CANParserError> {
    conn.execute(
        "INSERT OR IGNORE INTO SpecPGNs (
                id,
                label,
                acronym,
                description,
                pdu_format,
                pdu_specific,
                priority,
                length,
                transmission_rate
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            pgn,
            pgn_data.label,
            pgn_data.acronym,
            pgn_data.description,
            pgn_data.pdu_format,
            pgn_data.pdu_specific,
            pgn_data.priority,
            pgn_data.length,
            pgn_data.transmission_rate
        ],
    )?;
    Ok(())
}

/// Inserts a new SpecSPN into the database.
///
/// # Arguments
///
/// * `conn` - A reference to a SQLite `Connection` object.
/// * `pgn` - The PGN (Parameter Group Number) of the SpecSPN.
/// * `spn` - The SPN (Suspect Parameter Number) of the SpecSPN.
/// * `spn_data` - A reference to a `SpecSPN` struct containing the data to be inserted.
///
/// # Errors
///
/// Returns a `CANParserError` if the insertion fails.
///
/// # Example
///
/// ```
/// use can_parser::{CANParserError, SpecSPN};
/// use rusqlite::Connection;
///
/// let conn = Connection::open_in_memory().unwrap();
/// let spn_data = SpecSPN {
///     label: "Engine Speed".to_string(),
///     description: "Engine Speed".to_string(),
///     units: "RPM".to_string(),
///     length: 2,
///     resolution: 0.125,
///     offset: 0.0,
///     max: 8031.875,
///     start_bit: 0,
///     spn_type: "Measured".to_string(),
/// };
///
/// let result = insert_spec_spn(&conn, 61444, 190, &spn_data);
/// assert!(result.is_ok());
/// ```
pub fn insert_spec_spn(
    conn: &Connection,
    pgn: u16,
    spn: u16,
    spn_data: &SpecSPN,
) -> Result<(), CANParserError> {
    conn.execute(
        "INSERT OR IGNORE INTO SpecSPNs (
                id,
                pgn,
                label,
                description,
                units,
                length,
                resolution,
                offset,
                maximum,
                start_bit,
                spn_type
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            spn,
            pgn,
            spn_data.label,
            spn_data.description,
            spn_data.units,
            spn_data.length,
            spn_data.resolution,
            spn_data.offset,
            spn_data.max,
            spn_data.start_bit,
            spn_data.spn_type
        ],
    )?;
    Ok(())
}

/// Inserts a new CAN ID into the database.
///
/// # Arguments
///
/// * `conn` - A reference to a SQLite `Connection` object.
/// * `id` - A reference to a `CANID` object containing the ID information to be inserted.
///
/// # Errors
///
/// Returns a `CANParserError` if the insertion fails.
///
/// # Example
///
/// ```
/// use can_parser::CANID;
/// use rusqlite::Connection;
///
/// let conn = Connection::open_in_memory().unwrap();
/// let id = CANID::new(1234, 56789, 3, 255, 0, true, false, false);
/// assert!(insert_canid(&conn, &id).is_ok());
/// ```
pub fn insert_canid(conn: &Connection, id: &CANID) -> Result<(), CANParserError> {
    conn.execute(
        "INSERT OR IGNORE INTO CANIDs (
                id,
                pgn,
                priority,
                destination_address,
                source_address,
                extended,
                error,
                rtr
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            id.id,
            id.pgn,
            id.pri,
            id.da,
            id.sa,
            id.flags.ext,
            id.flags.err,
            id.flags.rtr
        ],
    )?;
    Ok(())
}

/// Inserts a CAN message into the database.
///
/// # Arguments
///
/// * `conn` - A reference to a SQLite database connection.
/// * `message` - A reference to the CAN message to be inserted.
///
/// # Errors
///
/// Returns a `CANParserError` if the insertion fails.
///
/// # Examples
///
/// ```
/// use can_parser::{CANMessage, CANParserError};
/// use rusqlite::Connection;
///
/// let conn = Connection::open_in_memory().unwrap();
/// let message = CANMessage::new(1234567890, 0x123, vec![0x01, 0x02, 0x03, 0x04]);
///
/// assert!(insert_message(&conn, &message).is_ok());
/// ```
pub fn insert_message(conn: &Connection, message: &CANMessage) -> Result<(), CANParserError> {
    let spn_values = serde_json::to_string(&message.data.spns)?;
    conn.execute(
        "INSERT OR IGNORE INTO messages (
                timestamp,
                can_id,
                length,
                data,
                spn_values
            )
            VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            message.ts,
            message.id.id,
            message.data.len,
            message.data.data,
            spn_values
        ],
    )?;
    Ok(())
}
