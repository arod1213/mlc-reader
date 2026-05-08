use crate::save::migrate_and_save;

pub mod bwarm;
pub mod save;
pub mod update;

fn main() {
    migrate_and_save();
}
