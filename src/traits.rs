use itertools::Itertools;
use sqlx::database::HasArguments;
use sqlx::{query::QueryAs, Postgres};

pub type Query<'q, O> = QueryAs<'q, Postgres, O, <Postgres as HasArguments<'q>>::Arguments>;

pub trait ToRow {
    fn bind<'q, O>(&self, query: Query<'q, O>, column_name: &'static str) -> Query<'q, O>;
}

pub trait Schema {
    const TABLE_NAME: &'static str;
    const COLUMNS: &'static [(&'static str, &'static str)];

    fn comma_seperated_columns() -> String {
        Self::COLUMNS.iter().map(|col| col.0).join(", ")
    }

    fn create_table() -> String {
        let table = Self::TABLE_NAME;
        let id = "id BIGSERIAL PRIMARY KEY";
        let cols = Self::COLUMNS
            .iter()
            .map(|col| format!("{} {}", col.0, col.1))
            .join(", ");
        format!("CREATE TABLE IF NOT EXISTS {table} ({id}, {cols})")
    }

    fn drop_table() -> String {
        let table = Self::TABLE_NAME;
        format!("DROP TABLE IF EXISTS {table}")
    }

    fn insert() -> String {
        let table = Self::TABLE_NAME;
        let cols = Self::comma_seperated_columns();
        let values = (1..=Self::COLUMNS.len())
            .map(|i| format!("${i}"))
            .join(", ");
        format!("INSERT INTO {table} ({cols}) VALUES ({values}) RETURNING id")
    }

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

    fn select(suffix: &str) -> String {
        let table = Self::TABLE_NAME;
        let cols = Self::comma_seperated_columns();
        format!("SELECT {cols} FROM {table} {suffix}")
    }

    fn delete(suffix: &str) -> String {
        let table = Self::TABLE_NAME;
        format!("DELETE FROM {table} {suffix}")
    }
}
