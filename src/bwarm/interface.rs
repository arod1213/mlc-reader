use libsql::{Connection, Statement};

pub trait BwarmEntry {
    fn filename() -> String;
    async fn migrate(conn: &Connection) -> Result<(), libsql::Error>;
    async fn prepare(conn: &Connection) -> Result<Statement, libsql::Error>;
    async fn insert(&self, stmt: &mut Statement) -> Result<(), libsql::Error>;
}
