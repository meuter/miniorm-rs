#![warn(
    missing_docs,
    rustdoc::missing_crate_level_docs,
    rustdoc::broken_intra_doc_links
)]

//! The `miniorm` crate provides a *very* simple
//! [ORM](https://en.wikipedia.org/wiki/Object%E2%80%93relational_mapping)
//! on top of [`sqlx`].
//!
//! [`sqlx`] already provides a [`FromRow`](sqlx::FromRow) trait that can be derived
//! automatically in order to convert a row from the database into an object. Howver,
//! there is no corresponding `ToRow` macro that would allow convert an object back
//! into a row to be inserted into the database.
//!
//! This is where `miniorm` comes in. It provides a trait [`Schema`]
//! that can also be automatically derived to describe the schema
//! of the table that should be used for a given entity (i.e. `struct`).
//!
//! Any struct that implements the [`FromRow`](sqlx::FromRow) and [`Schema`] traits can
//! be used to create a [`CrudStore`] that provide the so-called "CRUD" operations:
//! - (C)reate
//! - (R)ead
//! - (U)pdate
//! - (D)elete
//!
//! At the moment, `miniorm` only supports the postgres backend. Other backends
//! could be provided in the future.
//!
//! # Examples
//!
//! ```rust
//! use sqlx::FromRow;
//! use miniorm::Schema;
//!
//! #[derive(Debug, Clone, Eq, PartialEq, FromRow, Schema)]
//! struct Todo {
//!     #[column(TEXT NOT NULL)]
//!     description: String,
//!
//!     #[column(BOOLEAN NOT NULL DEFAULT false)]
//!     done: bool,
//! }
//!
//! ```
//!
//! For more complete examples, see:
//! - [todo example](../src/miniorm_example_todo/main.rs.html) for a simple example.
//! - [stock transaction example](../src/miniorm_example_transactions/main.rs.html)
//!   for a more complex example, where certain fields are stored as
//!   [`JSONB`](https://www.postgresql.org/docs/current/datatype-json.html) column
//!   using [`serde_json`].

mod store;

/// module providing the [`Schema`] trait.
pub mod traits;
pub use miniorm_macros::Schema;
pub use store::CrudStore;
