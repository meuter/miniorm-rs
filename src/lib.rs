mod store;

pub mod traits;
pub use miniorm_macros::{Schema, ToRow};
pub use store::CrudStore;
