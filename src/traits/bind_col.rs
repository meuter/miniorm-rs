use super::sqlx_ext::Bind;
use sqlx::database::Database;

/// Trait that can be implemented on a `struct` to bind the fields of
/// of this struct with a [sqlx::query::QueryAs] or a [sqlx::query::Query].
///
/// # Example
///
/// ```
/// use miniorm::{BindColumn, Bind};
/// use sqlx::Postgres;
///
/// struct Todo {
///     description: String,
///     done: bool,
/// }
///
/// impl BindColumn<Postgres> for Todo {
///     fn bind_column<'q, Q>(&self, query: Q, column_name: &'static str) -> Q
///     where
///         Q: ::miniorm::Bind<'q, Postgres> {
///         match column_name {
///             "description" => query.bind(self.description.clone()),
///             "done" => query.bind(self.done.clone()),
///             _ => query,
///         }
///     }
/// }
///
/// ```
///
/// This trait can be derived automatically using the [Entity](miniorm_macros::Entity)
/// derive macro.
///
pub trait BindColumn<DB: Database> {
    /// binds a specific column using the provided query.
    fn bind_column<'q, Q>(&self, query: Q, column_name: &'static str) -> Q
    where
        Q: Bind<'q, DB>;
}
