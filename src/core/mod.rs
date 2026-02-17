// core logic - ai, database, and safety checks

mod ai;
mod db;
mod safety;

pub use ai::{Ai, Provider};
pub use db::{Db, QueryResult};
pub use safety::Safety;
