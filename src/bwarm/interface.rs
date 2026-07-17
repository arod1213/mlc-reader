use libsql::{Connection, Statement};

#[allow(async_fn_in_trait)]
pub trait BwarmEntry {
    fn filename() -> String;
    async fn migrate(conn: &Connection) -> Result<(), libsql::Error>;
    async fn prepare(conn: &Connection) -> Result<Statement, libsql::Error>;
    async fn insert_many(objects: &[Self], stmt: &mut Statement) -> Result<(), libsql::Error>
    where
        Self: Sized;
}
