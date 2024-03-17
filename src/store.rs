use crate::{traits::Schema, WithId};
use itertools::Itertools;
use sqlx::{
    postgres::{PgQueryResult, PgRow},
    FromRow, PgPool,
};
use std::marker::PhantomData;

/// A `Store` is a wrapper around the [`PgPool`] that allows
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
pub struct Store<E> {
    db: PgPool,
    entity: PhantomData<E>,
}

impl<E> Store<E> {
    /// Create a new [`Store`]
    pub fn new(db: PgPool) -> Self {
        let entity = PhantomData;
        Self { db, entity }
    }
}

/// Table
impl<E> Store<E>
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
impl<E> Store<E>
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
        Ok(WithId::new(entity, id))
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Read
///////////////////////////////////////////////////////////////////////////////////////////////////
impl<E> Store<E>
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
impl<E> Store<E>
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
            query = entity.inner().bind(query, col)
        }

        let entity = query.bind(entity.id()).fetch_one(&self.db).await?;
        Ok(entity)
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
/// Delete
///////////////////////////////////////////////////////////////////////////////////////////////////
impl<E> Store<E>
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

#[cfg(feature = "axum")]
mod handlers {

    use crate::{traits::Schema, Store};
    use axum::{
        extract::{Path, State},
        http::StatusCode,
        response::IntoResponse,
        Json,
    };
    use serde::Serialize;
    use sqlx::{postgres::PgRow, FromRow};

    pub(crate) async fn list<E>(
        State(store): State<Store<E>>,
    ) -> Result<impl IntoResponse, StatusCode>
    where
        E: Schema + for<'r> FromRow<'r, PgRow> + Unpin + Send + Serialize,
    {
        store
            .list()
            .await
            .map(Json)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    // TODO: create

    pub(crate) async fn read<E>(
        Path(id): Path<i64>,
        State(todos): State<Store<E>>,
    ) -> Result<impl IntoResponse, StatusCode>
    where
        E: Schema + for<'r> FromRow<'r, PgRow> + Unpin + Send + Serialize,
    {
        todos
            .read(id)
            .await
            .map(|e| Json(e.into_inner()))
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub(crate) async fn delete<E>(
        Path(id): Path<i64>,
        State(todos): State<Store<E>>,
    ) -> Result<impl IntoResponse, StatusCode>
    where
        E: Schema + for<'r> FromRow<'r, PgRow> + Unpin + Send + Serialize,
    {
        todos
            .delete(id)
            .await
            .map(|_| Json(()))
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    // TODO: update

    pub(crate) async fn delete_all<E>(
        State(todos): State<Store<E>>,
    ) -> Result<impl IntoResponse, StatusCode>
    where
        E: Schema + for<'r> FromRow<'r, PgRow> + Unpin + Send + Serialize,
    {
        todos
            .delete_all()
            .await
            .map(|_| Json(()))
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }
}

#[cfg(feature = "axum")]
impl<E> Store<E>
where
    E: Schema
        + for<'r> FromRow<'r, PgRow>
        + Unpin
        + Send
        + serde::Serialize
        + Clone
        + Sync
        + 'static,
{
    /// Converts the store into an [`axum::Router`]
    /// that will handle all standard REST request to realize CRUD operations
    /// on the store.
    ///
    /// - `GET` on `/` will list all entities from the store in json format.
    /// - `GET` on `/:id` will read the entity with the provided `id` from the store
    ///   and return it in json format
    /// - `DELETE` on `/` will delete all entities from the store.
    /// - `DELETE` on `/:id` will delete the entity with the provided `id` from the store.
    /// - TODO: POST and PUT
    pub fn into_axum_router<S>(self) -> axum::Router<S> {
        use axum::routing::*;

        Router::new()
            .route("/", get(handlers::list))
            .route("/", delete(handlers::delete_all))
            .route("/:id", delete(handlers::delete))
            .route("/:id", get(handlers::read))
            .with_state(self)
    }
}
