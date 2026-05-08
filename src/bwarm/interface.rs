use sqlite::{Connection, Statement};

pub trait BwarmEntry {
    fn filename() -> String;
    fn migrate(conn: &Connection) -> Result<(), sqlite::Error>;
    fn stmt<'a>(conn: &'a Connection) -> Result<Statement<'a>, sqlite::Error>;
    fn bind(&self, stmt: &mut Statement) -> Result<(), sqlite::Error>;
    fn insert(&self, stmt: &mut Statement) -> Result<(), sqlite::Error> {
        self.bind(stmt)?;
        stmt.next()?;
        Ok(())
    }
}
