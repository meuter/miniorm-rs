use itertools::Itertools;
use sqlx::database::HasArguments;
use sqlx::{query::QueryAs, Postgres};

/// Convenience alias to [`QueryAs`] specialized [`Postgres`]
pub type Query<'q, O> = QueryAs<'q, Postgres, O, <Postgres as HasArguments<'q>>::Arguments>;

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
/// impl miniorm::traits::Schema for Todo {
///     const TABLE_NAME: &'static str = "todo";
///     const COLUMNS: &'static [(&'static str, &'static str)] = &[
///         ("description", "TEXT NOT NULL"),
///         ("done", "BOOLEAN NOT NULL"),
///     ];
///
///     fn bind<'q, O>(
///         &self,
///         query: miniorm::traits::Query<'q, O>,
///         column_name: &'static str,
///     ) -> miniorm::traits::Query<'q, O> {
///         match column_name {
///             "description" => query.bind(self.description.clone()),
///             "done" => query.bind(self.done.clone()),
///             _ => query,
///         }
///     }
/// }
///
/// ```
pub trait Schema {
    /// name of the table in the database
    const TABLE_NAME: &'static str;

    /// list of all the columns and their postgress types
    const COLUMNS: &'static [(&'static str, &'static str)];

    /// binds a specific column using the provided query.
    fn bind<'q, O>(&self, query: Query<'q, O>, column_name: &'static str) -> Query<'q, O>;

    /// generates a string that contains all columns names separated by
    /// commas.
    ///
    /// # Example
    ///
    /// ```
    /// use miniorm::traits::Schema;
    ///
    /// #[derive(miniorm::Schema)]
    /// struct Todo {
    ///     #[column(TEXT NOT NULL)]
    ///     description: String,
    ///
    ///     #[column(BOOLEAN NOT NULL)]
    ///     done: bool,
    /// }
    ///
    /// assert_eq!(Todo::comma_seperated_columns(), "description, done");
    /// ```
    ///
    fn comma_seperated_columns() -> String {
        Self::COLUMNS.iter().map(|col| col.0).join(", ")
    }

    /// Generate a `CREATE TABLE ...` sql statement that can
    /// be used to create the table corresponding to this object.
    fn create_table_stmt() -> String {
        let table = Self::TABLE_NAME;
        let id = "id BIGSERIAL PRIMARY KEY";
        let cols = Self::COLUMNS
            .iter()
            .map(|col| format!("{} {}", col.0, col.1))
            .join(", ");
        format!("CREATE TABLE IF NOT EXISTS {table} ({id}, {cols})")
    }

    /// Generate the `DROP TABLE...` sql statement to drop the
    /// table corresponding to this object.
    fn drop_table_stmt() -> String {
        let table = Self::TABLE_NAME;
        format!("DROP TABLE IF EXISTS {table}")
    }

    /// Generate the `INSERT INTO...` sql statement to insert
    /// an object into the table
    fn insert_stmt() -> String {
        let table = Self::TABLE_NAME;
        let cols = Self::comma_seperated_columns();
        let values = (1..=Self::COLUMNS.len())
            .map(|i| format!("${i}"))
            .join(", ");
        format!("INSERT INTO {table} ({cols}) VALUES ({values}) RETURNING id")
    }

    /// Generate the `UPDATE...` sql statement to insert an object into the
    /// table
    fn update() -> String {
        let table = Self::TABLE_NAME;
        let values = Self::COLUMNS
            .iter()
            .map(|col| col.0)
            .enumerate()
            .map(|(i, col)| format!("{col}=${}", i + 1))
            .join(", ");
        let suffix = format!("WHERE id=${}", Self::COLUMNS.len() + 1);
        format!("UPDATE {table} SET {values} {suffix} RETURNING id")
    }

    /// Generate a `SELECT...` statement
    fn select(suffix: &str) -> String {
        let table = Self::TABLE_NAME;
        let cols = Self::comma_seperated_columns();
        format!("SELECT {cols} FROM {table} {suffix}")
    }

    /// Generate a delete stamenent
    fn delete(suffix: &str) -> String {
        let table = Self::TABLE_NAME;
        format!("DELETE FROM {table} {suffix}")
    }
}
