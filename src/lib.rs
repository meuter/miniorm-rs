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
pub use traits::{Bind, BindableQuery, Create, Schema};
