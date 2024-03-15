use crate::traits::Schema;
use itertools::Itertools;
use sqlx::{
    postgres::{PgQueryResult, PgRow},
    prelude::FromRow,
    PgPool,
};
use std::marker::PhantomData;

/// A `CrudStore` is a wrapper around the [`PgPool`] that allows
/// to perform basic, so-called "CRUD" operations.
///
/// For these operation to be available, the underlying entity type
/// should implement the following traits:
/// - [FromRow] from `sqlx`.
/// - [Schema] from this crate.
///
/// Note that both can be derived automatically; [FromRow] using sqlx
/// and [Schema] using this crate.
pub struct CrudStore<'d, E> {
    db: &'d PgPool,
    entity: PhantomData<E>,
}

impl<'d, E> CrudStore<'d, E> {
    /// Create a new [`CrudStore`]
    pub fn new(db: &'d PgPool) -> Self {
        let entity = PhantomData;
        Self { db, entity }
    }
}

/// Table
impl<'d, E> CrudStore<'d, E>
where
    E: Schema,
{
    /// Recreates the table associated with the entity's [`Schema`]
    pub async fn recreate_table(&self) -> sqlx::Result<PgQueryResult> {
        self.drop_table().await?;
        self.create_table().await
    }

    /// Creates the table associated with the entity's [`Schema`]
    pub async fn create_table(&self) -> sqlx::Result<PgQueryResult> {
        let table = E::TABLE_NAME;
        let id = "id BIGSERIAL PRIMARY KEY";
        let cols = E::COLUMNS
            .iter()
            .map(|col| format!("{} {}", col.0, col.1))
            .join(", ");
        let sql = format!("CREATE TABLE IF NOT EXISTS {table} ({id}, {cols})");
        sqlx::query(&sql).execute(self.db).await
    }

    /// Drops the table associated with the entity's [`Schema`]
    pub async fn drop_table(&self) -> sqlx::Result<PgQueryResult> {
        let table = E::TABLE_NAME;
        let sql = format!("DROP TABLE IF EXISTS {table}");
        sqlx::query(&sql).execute(self.db).await
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
/// Create
///////////////////////////////////////////////////////////////////////////////////////////////////
impl<'d, E> CrudStore<'d, E>
where
    E: for<'r> FromRow<'r, PgRow> + Schema,
{
    /// Create an object in the database and returns its `id`.
    pub async fn create(&self, entity: &E) -> sqlx::Result<i64> {
        let table = E::TABLE_NAME;
        let cols = E::COLUMNS.iter().map(|col| col.0).join(", ");
        let values = (1..=E::COLUMNS.len()).map(|i| format!("${i}")).join(", ");
        let sql = format!("INSERT INTO {table} ({cols}) VALUES ({values}) RETURNING id");
        let mut query = sqlx::query_as(&sql);

        for col in E::COLUMNS.iter().map(|col| col.0) {
            query = entity.bind(query, col)
        }

        let (id,) = query.fetch_one(self.db).await?;
        Ok(id)
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Read
///////////////////////////////////////////////////////////////////////////////////////////////////
impl<'d, E> CrudStore<'d, E>
where
    E: for<'r> FromRow<'r, PgRow> + Schema + Unpin + Send,
{
    fn select_stmt(suffix: &str) -> String {
        let table = E::TABLE_NAME;
        let cols = E::COLUMNS.iter().map(|col| col.0).join(", ");
        format!("SELECT {cols} FROM {table} {suffix}")
    }

    /// Reads and returns an object from the database
    pub async fn read(&self, id: i64) -> sqlx::Result<E> {
        let sql = Self::select_stmt("WHERE id=$1");
        sqlx::query_as(&sql).bind(id).fetch_one(self.db).await
    }

    /// Lists and return all object from the database
    pub async fn list(&self) -> sqlx::Result<Vec<E>> {
        let sql = Self::select_stmt("ORDER BY id");
        sqlx::query_as(&sql).fetch_all(self.db).await
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
/// Update
///////////////////////////////////////////////////////////////////////////////////////////////////
impl<'d, E> CrudStore<'d, E>
where
    E: for<'r> FromRow<'r, PgRow> + Schema,
{
    /// Update an object in the database and returns its `id`.
    pub async fn update(&self, id: i64, entity: &E) -> sqlx::Result<i64> {
        let table = E::TABLE_NAME;
        let values = E::COLUMNS
            .iter()
            .map(|col| col.0)
            .enumerate()
            .map(|(i, col)| format!("{col}=${}", i + 1))
            .join(", ");
        let suffix = format!("WHERE id=${}", E::COLUMNS.len() + 1);
        let sql = format!("UPDATE {table} SET {values} {suffix} RETURNING id");
        let mut query = sqlx::query_as(&sql);

        for col in E::COLUMNS.iter().map(|col| col.0) {
            query = entity.bind(query, col)
        }

        let (id,) = query.bind(id).fetch_one(self.db).await?;
        Ok(id)
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
/// Delete
///////////////////////////////////////////////////////////////////////////////////////////////////
impl<'d, E> CrudStore<'d, E>
where
    E: Schema,
{
    fn delete_stmt(suffix: &str) -> String {
        let table = E::TABLE_NAME;
        format!("DELETE FROM {table} {suffix}")
    }

    /// Delete the object of type `E` corresponding to the provided `id`
    pub async fn delete(&self, id: i64) -> sqlx::Result<PgQueryResult> {
        let sql = Self::delete_stmt("WHERE id=$1");
        sqlx::query(&sql).bind(id).execute(self.db).await
    }

    /// Delete all objects of type E
    pub async fn delete_all(&self) -> sqlx::Result<u64> {
        let sql = Self::delete_stmt("");
        Ok(sqlx::query(&sql).execute(self.db).await?.rows_affected())
    }
}
