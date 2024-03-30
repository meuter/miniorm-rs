#![warn(
    missing_docs,
    rustdoc::missing_crate_level_docs,
    rustdoc::broken_intra_doc_links
)]
#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

mod store;

/// module providing the [`Schema`] trait.
pub mod traits;
pub use miniorm_macros::Schema;
pub use store::Store;
