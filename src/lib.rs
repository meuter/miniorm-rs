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

/// Prelude including all the necessary traits for convenience
pub mod prelude {
    pub use super::store::Store;
    pub use super::traits::bind_col::BindColumn;
    pub use super::traits::crud::{Create, Crud, Delete, Read, Update};
    pub use super::traits::schema::Schema;
    pub use super::traits::sqlx::Bind;
    pub use miniorm_macros::Entity;
}
