use crate::traits::Schema;
use itertools::Itertools;
use sqlx::{
    postgres::{PgQueryResult, PgRow},
    FromRow, PgPool, Row,
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
#[derive(Clone, Debug)]
pub struct CrudStore<E> {
    db: PgPool,
    entity: PhantomData<E>,
}

pub struct WithId<E> {
    pub id: i64,
    pub inner: E,
}

impl<E> WithId<E> {
    pub fn new(inner: E, id: i64) -> Self {
        WithId { inner, id }
    }
}

impl<'r, E> FromRow<'r, PgRow> for WithId<E>
where
    E: FromRow<'r, PgRow>,
{
    fn from_row(row: &'r PgRow) -> sqlx::Result<Self> {
        let inner = E::from_row(row)?;
        let id = row.try_get("id")?;
        Ok(Self { inner, id })
    }
}

impl<E> CrudStore<E> {
    /// Create a new [`CrudStore`]
    pub fn new(db: PgPool) -> Self {
        let entity = PhantomData;
        Self { db, entity }
    }
}

/// Table
impl<E> CrudStore<E>
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
        sqlx::query(&sql).execute(&self.db).await
    }

    /// Drops the table associated with the entity's [`Schema`]
    pub async fn drop_table(&self) -> sqlx::Result<PgQueryResult> {
        let table = E::TABLE_NAME;
        let sql = format!("DROP TABLE IF EXISTS {table}");
        sqlx::query(&sql).execute(&self.db).await
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
/// Create
///////////////////////////////////////////////////////////////////////////////////////////////////
impl<E> CrudStore<E>
where
    E: for<'r> FromRow<'r, PgRow> + Schema,
{
    /// Create an object in the database and returns its `id`.
    pub async fn create(&self, entity: E) -> sqlx::Result<WithId<E>> {
        let table = E::TABLE_NAME;
        let cols = E::COLUMNS.iter().map(|col| col.0).join(", ");
        let values = (1..=E::COLUMNS.len()).map(|i| format!("${i}")).join(", ");
        let sql = format!("INSERT INTO {table} ({cols}) VALUES ({values}) RETURNING id");
        let mut query = sqlx::query_as(&sql);

        for col in E::COLUMNS.iter().map(|col| col.0) {
            query = entity.bind(query, col)
        }

        let (id,) = query.fetch_one(&self.db).await?;
        Ok(WithId { inner: entity, id })
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Read
///////////////////////////////////////////////////////////////////////////////////////////////////
impl<E> CrudStore<E>
where
    E: for<'r> FromRow<'r, PgRow> + Schema + Unpin + Send,
{
    fn select_stmt(suffix: &str) -> String {
        let table = E::TABLE_NAME;
        let cols = E::COLUMNS.iter().map(|col| col.0).join(", ");
        format!("SELECT id, {cols} FROM {table} {suffix}")
    }

    /// Reads and returns an object from the database
    pub async fn read(&self, id: i64) -> sqlx::Result<WithId<E>> {
        let sql = Self::select_stmt("WHERE id=$1");
        sqlx::query_as(&sql).bind(id).fetch_one(&self.db).await
    }

    /// Lists and return all object from the database
    pub async fn list(&self) -> sqlx::Result<Vec<WithId<E>>> {
        let sql = Self::select_stmt("ORDER BY id");
        sqlx::query_as(&sql).fetch_all(&self.db).await
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
/// Update
///////////////////////////////////////////////////////////////////////////////////////////////////
impl<E> CrudStore<E>
where
    E: for<'r> FromRow<'r, PgRow> + Schema + Unpin + Send,
{
    /// Update an object in the database and returns its `id`.
    pub async fn update(&self, entity: WithId<E>) -> sqlx::Result<WithId<E>> {
        let table = E::TABLE_NAME;
        let values = E::COLUMNS
            .iter()
            .map(|col| col.0)
            .enumerate()
            .map(|(i, col)| format!("{col}=${}", i + 1))
            .join(", ");
        let suffix = format!("WHERE id=${}", E::COLUMNS.len() + 1);
        let cols = E::COLUMNS.iter().map(|col| col.0).join(", ");
        let sql = format!("UPDATE {table} SET {values} {suffix} RETURNING id, {cols}");

        let mut query = sqlx::query_as(&sql);

        for col in E::COLUMNS.iter().map(|col| col.0) {
            query = entity.inner.bind(query, col)
        }

        let entity = query.bind(entity.id).fetch_one(&self.db).await?;
        Ok(entity)
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
/// Delete
///////////////////////////////////////////////////////////////////////////////////////////////////
impl<E> CrudStore<E>
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
        sqlx::query(&sql).bind(id).execute(&self.db).await
    }

    /// Delete all objects of type E
    pub async fn delete_all(&self) -> sqlx::Result<u64> {
        let sql = Self::delete_stmt("");
        Ok(sqlx::query(&sql).execute(&self.db).await?.rows_affected())
    }
}
