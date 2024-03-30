use sqlx::Database;

/// Trait that can be implemented on a `struct` to associate a basic SQL schema
/// with it.
///
/// # Example
///
/// ```
/// use miniorm::Schema;
/// use sqlx::Postgres;
///
/// struct Todo {
///     description: String,
///     done: bool,
/// }
///
/// impl Schema<Postgres> for Todo {
///     const ID_DECLARATION: &'static str = "id BIGSERIAL PRIMARY KEY";
///     const TABLE_NAME: &'static str = "todo";
///     const COLUMNS: &'static [(&'static str, &'static str)] = &[
///         ("description", "TEXT NOT NULL"),
///         ("done", "BOOLEAN NOT NULL"),
///     ];
/// }
/// ```
///
/// # Note
///
/// This trait can be derived automatically using the [Entity](miniorm_macros::Entity)
/// derive macro.
///
pub trait Schema<DB: Database> {
    /// SQL declatation of the primary key
    const ID_DECLARATION: &'static str;

    /// name of the table in the database
    const TABLE_NAME: &'static str;

    /// list of all the columns and their postgress types
    const COLUMNS: &'static [(&'static str, &'static str)];
}
