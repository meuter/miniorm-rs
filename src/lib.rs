#![warn(
    missing_docs,
    rustdoc::missing_crate_level_docs,
    rustdoc::broken_intra_doc_links
)]
#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

mod store;
mod traits;

pub use miniorm_macros::Entity;
pub use store::Store;
pub use traits::{BindColumn, BindableQuery, Create, Crud, Delete, Schema, Update};

/// Prelude including all the necessary traits for convenience
pub mod prelude {
    pub use super::store::Store;
    pub use super::traits::{BindColumn, BindableQuery, RowsAffected, Schema};
    pub use super::traits::{Create, Crud, Delete, Update};
    pub use miniorm_macros::Entity;
}
