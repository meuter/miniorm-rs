use sqlx::database::{Database, HasArguments};

/// Convenience type for simplify the definition of [`Bind`]
pub type QueryAs<'q, DB, O> = sqlx::query::QueryAs<'q, DB, O, <DB as HasArguments<'q>>::Arguments>;

/// Trait that can be implemented on a `struct` to bind the fields of
/// of this struct with a [sqlx::query::QueryAs].
///
/// # Example
///
/// ```
/// use miniorm::{Bind, QueryAs};
/// use sqlx::Postgres;
///
/// struct Todo {
///     description: String,
///     done: bool,
/// }
///
/// impl Bind<Postgres> for Todo {
///     fn bind<'q, O>(
///         &self,
///         query: QueryAs<'q, Postgres, O>,
///         column_name: &'static str
///     ) -> QueryAs<'q, Postgres, O>{
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
pub trait Bind<DB: Database> {
    /// binds a specific column using the provided query.
    fn bind<'q, O>(
        &self,
        query: QueryAs<'q, DB, O>,
        column_name: &'static str,
    ) -> QueryAs<'q, DB, O>;
}
