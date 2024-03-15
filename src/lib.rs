#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
#![warn(
    missing_docs,
    rustdoc::missing_crate_level_docs,
    rustdoc::broken_intra_doc_links
)]

mod store;

pub mod traits;
pub use miniorm_macros::Schema;
pub use store::CrudStore;
