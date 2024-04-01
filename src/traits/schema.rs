use sqlx::Database;

/// Trait that can be implemented on a `struct` to associate a basic SQL schema
/// with it.
///
/// # Example
///
/// ```
/// use miniorm::prelude::*;
/// use sqlx::Postgres;
///
/// struct Todo {
///     description: String,
///     done: bool,
/// }
///
/// impl Schema<Postgres> for Todo {
///     const MINIORM_CREATE_TABLE: &'static str = r#"
///         CREATE TABLE IF NOT EXISTS todo (
///             id BIGSERIAL PRIMARY KEY,
///             description TEXT NOT NULL,
///             done BOOLEAN NOT NULL
///         )"#;
///     const MINIORM_DROP_TABLE: &'static str = r#"
///         DROP TABLE IF EXISTS todo"#;
///     const MINIORM_CREATE: &'static str = r#"
///         INSERT INTO todo (description, done) VALUES ($1,$2) RETURNING id"#;
///     const MINIORM_READ: &'static str = r#"
///         SELECT selection, done FROM todo WHERE id=$1"#;
///     const MINIORM_LIST: &'static str = r#"
///         SELECT selection, done FROM todo ORDER BY id"#;
///     const MINIORM_UPDATE: &'static str = r#"
///         UPDATE todo SET selection=$1, done=$2 WHERE id=$2"#;
///     const MINIORM_DELETE: &'static str = r#"
///         DELETE FROM todo WHERE id=$1"#;
///     const MINIORM_DELETE_ALL: &'static str = r#"
///         DELETE FROM todo"#;
///     const MINIORM_TABLE_NAME: &'static str = "todo";
///     const MINIORM_COLUMNS: &'static [&'static str] = &[
///         "description",
///         "done",
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
    /// SQL query to create the table
    const MINIORM_CREATE_TABLE: &'static str;

    /// SQL query to drop the table
    const MINIORM_DROP_TABLE: &'static str;

    /// SQL query to create a row
    const MINIORM_CREATE: &'static str;

    /// SQL query to read a row by id
    const MINIORM_READ: &'static str;

    /// SQL query to list all rows ordered by id
    const MINIORM_LIST: &'static str;

    /// SQL query to update a row by id
    const MINIORM_UPDATE: &'static str;

    /// SQL query to delete a row by id
    const MINIORM_DELETE: &'static str;

    /// SQL query to delete all rows
    const MINIORM_DELETE_ALL: &'static str;

    /// name of the table in the database
    const MINIORM_TABLE_NAME: &'static str;

    /// list of all the columns and their postgress types
    const MINIORM_COLUMNS: &'static [&'static str];
}
