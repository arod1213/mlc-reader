pub mod party;
pub mod types;

pub mod works;
#[cfg(test)]
pub(crate) mod test_util {
    use crate::migration::migrate_full_db;
    use rusqlite::Connection;

    pub fn empty_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrate_full_db(&conn).unwrap();
        conn
    }

    pub fn assert_runs<T>(res: rusqlite::Result<T>) {
        match res {
            Ok(_) | Err(rusqlite::Error::QueryReturnedNoRows) => {}
            Err(e) => panic!("query failed: {e}"),
        }
    }
}
