use sqlx::database::HasArguments;
use sqlx::Database;

/// Convenience alias to [`QueryAs`]
pub type QueryAs<'q, DB, O> = sqlx::query::QueryAs<'q, DB, O, <DB as HasArguments<'q>>::Arguments>;

/// Trait that can derived by a struct to specify the database scema
/// that should be used to store objects of that type.
///
/// # Example
///
/// This trait can be automatically derived as follows:
///
/// ```
/// #[derive(miniorm::Schema)]
/// struct Todo {
///     #[column(TEXT NOT NULL)]
///     description: String,
///
///     #[column(BOOLEAN NOT NULL)]
///     done: bool,
/// }
/// ```
///
/// whith is equivalent to:
///
/// ```
/// struct Todo {
///     description: String,
///     done: bool,
/// }
///
/// impl miniorm::traits::Schema<sqlx::Postgres> for Todo {
///     const TABLE_NAME: &'static str = "todo";
///     const COLUMNS: &'static [(&'static str, &'static str)] = &[
///         ("description", "TEXT NOT NULL"),
///         ("done", "BOOLEAN NOT NULL"),
///     ];
///
///     fn bind<'q, O>(
///         &self,
///         query: miniorm::traits::QueryAs<'q, sqlx::Postgres, O>,
///         column_name: &'static str,
///     ) -> miniorm::traits::QueryAs<'q, sqlx::Postgres, O> {
///         match column_name {
///             "description" => query.bind(self.description.clone()),
///             "done" => query.bind(self.done.clone()),
///             _ => query,
///         }
///     }
/// }
///
/// ```
pub trait Schema<DB: Database> {
    /// name of the table in the database
    const TABLE_NAME: &'static str;

    /// list of all the columns and their postgress types
    const COLUMNS: &'static [(&'static str, &'static str)];

    /// binds a specific column using the provided query.
    fn bind<'q, O>(
        &self,
        query: QueryAs<'q, DB, O>,
        column_name: &'static str,
    ) -> QueryAs<'q, DB, O>;
}
