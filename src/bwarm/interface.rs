use csv::StringRecord;
use rusqlite::{Connection, Statement};

#[allow(async_fn_in_trait)]
pub trait BwarmEntry {
    fn filename() -> String;
    fn validate_headers(headers: &StringRecord) -> Result<(), String>;
    fn migrate(conn: &Connection) -> Result<(), rusqlite::Error>;
    fn prepare<'a>(conn: &'a Connection) -> Result<Statement<'a>, rusqlite::Error>;
    fn insert_from_csv(
        fields: &StringRecord,
        stmt: &mut Statement,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>; // true if inserted, false if
    // skipped
}
