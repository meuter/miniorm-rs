/// The [`miniorm`] crate provides a *very* simple
/// [ORM](https://en.wikipedia.org/wiki/Object%E2%80%93relational_mapping)
/// on top of [`sqlx`].
///
/// [`sqlx`] already provides a [`FromRow`] trait that can be automatically
/// derived in order to read an entity from the database. Howver, there
/// is no corresponding `ToRow` macro that would also allow to insert or
/// update entities in the database.
///
/// This is where [`miniorm`] comes in. It provides a trait [`Schema`]
/// that can also be automatically derived to describe the schema
/// of the table that should be used for a given entity (i.e. `struct`).
///
/// # Example
///
/// ```
/// #[derive(Debug, Clone, Eq, PartialEq, sqlx::FromRow, miniorm::Schema)]
/// struct Todo {
///     #[column(TEXT NOT NULL)]
///     description: String,
///
///     #[column(BOOLEAN NOT NULL DEFAULT false)]
///     done: bool,
/// }
/// ```
///
/// At the moment, [`miniorm`] only support Postgres, but could
/// be extended to other backends in the future.
///
mod store;

pub mod traits;
pub use miniorm_macros::Schema;
pub use store::CrudStore;
