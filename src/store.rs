use crate::traits::{Bind, Schema};
use itertools::Itertools;
use sqlx::{
    database::HasArguments, ColumnIndex, Database, Decode, Encode, Executor, FromRow,
    IntoArguments, Pool, Type,
};
use std::marker::PhantomData;

/// A `Store` is a wrapper around a [`Pool`] that allows
/// to perform basic, so-called "CRUD" operations.
///
/// For these operation to be available, the underlying entity type
/// should implement the following traits:
/// - [FromRow] from `sqlx`.
/// - [Schema] from this crate.
///
/// Note that both can be derived automatically; [FromRow] using sqlx
/// and [Schema] using this crate.
pub struct Store<DB: Database, E> {
    db: Pool<DB>,
    entity: PhantomData<E>,
}

impl<DB: Database, E> Store<DB, E> {
    /// Create a new [`Store`]
    pub fn new(db: Pool<DB>) -> Self {
        let entity = PhantomData;
        Self { db, entity }
    }
}

/// Table
impl<DB: Database, E: Schema<DB>> Store<DB, E>
where
    for<'c> &'c mut <DB as sqlx::Database>::Connection: Executor<'c, Database = DB>,
    for<'c> <DB as HasArguments<'c>>::Arguments: IntoArguments<'c, DB>,
{
    /// Recreates the table associated with the entity's [`Schema`]
    pub async fn recreate_table(&self) -> sqlx::Result<<DB as Database>::QueryResult> {
        self.drop_table().await?;
        self.create_table().await
    }

    /// Creates the table associated with the entity's [`Schema`]
    pub async fn create_table(&self) -> sqlx::Result<<DB as Database>::QueryResult> {
        sqlx::query(E::MINIORM_CREATE_TABLE).execute(&self.db).await
    }

    /// Drops the table associated with the entity's [`Schema`]
    pub async fn drop_table(&self) -> sqlx::Result<<DB as Database>::QueryResult> {
        sqlx::query(E::MINIORM_DROP_TABLE).execute(&self.db).await
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
/// Create
///////////////////////////////////////////////////////////////////////////////////////////////////
impl<DB, E> Store<DB, E>
where
    E: for<'r> FromRow<'r, <DB as Database>::Row> + Schema<DB> + Bind<DB>,
    DB: Database,
    for<'c> &'c mut <DB as sqlx::Database>::Connection: Executor<'c, Database = DB>,
    for<'c> <DB as HasArguments<'c>>::Arguments: IntoArguments<'c, DB>,
    for<'c> i64: Type<DB> + Decode<'c, DB>,
    usize: ColumnIndex<<DB as sqlx::Database>::Row>,
{
    /// Create an object in the database and returns its `id`.
    pub async fn create(&self, entity: &E) -> sqlx::Result<i64> {
        let table = E::TABLE_NAME;
        let cols = E::COLUMNS.iter().map(|col| col.0).join(", ");
        let values = (1..=E::COLUMNS.len()).map(|i| format!("${i}")).join(", ");
        if DB::NAME == "MySql" {
            // TODO: RETURNING id does not work in MySql, need to use SELECT LAST_INSERT_ID();
            // https://dev.mysql.com/doc/refman/8.0/en/information-functions.html#function_last-insert-id
            todo!();
        } else {
            let sql = format!("INSERT INTO {table} ({cols}) VALUES ({values}) RETURNING id");
            let mut query = sqlx::query_as(&sql);

            for col in E::COLUMNS.iter().map(|col| col.0) {
                query = entity.bind(query, col)
            }

            let (id,) = query.fetch_one(&self.db).await?;
            Ok(id)
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Read
///////////////////////////////////////////////////////////////////////////////////////////////////
impl<DB, E> Store<DB, E>
where
    DB: Database,
    E: Unpin + Send,
    E: for<'r> FromRow<'r, <DB as Database>::Row> + Schema<DB>,
    for<'c> &'c mut <DB as sqlx::Database>::Connection: Executor<'c, Database = DB>,
    for<'c> <DB as HasArguments<'c>>::Arguments: IntoArguments<'c, DB>,
    for<'c> i64: Type<DB> + Encode<'c, DB>,
{
    fn select_stmt(suffix: &str) -> String {
        let table = E::TABLE_NAME;
        let cols = E::COLUMNS.iter().map(|col| col.0).join(", ");
        format!("SELECT {cols} FROM {table} {suffix}")
    }

    /// Reads and returns an object from the database
    pub async fn read(&self, id: i64) -> sqlx::Result<E> {
        let sql = Self::select_stmt("WHERE id=$1");
        sqlx::query_as(&sql).bind(id).fetch_one(&self.db).await
    }

    /// Lists and return all object from the database
    pub async fn list(&self) -> sqlx::Result<Vec<E>> {
        let sql = Self::select_stmt("ORDER BY id");
        sqlx::query_as(&sql).fetch_all(&self.db).await
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
/// Update
///////////////////////////////////////////////////////////////////////////////////////////////////
impl<DB, E> Store<DB, E>
where
    DB: Database,
    for<'c> &'c mut <DB as sqlx::Database>::Connection: Executor<'c, Database = DB>,
    for<'c> <DB as HasArguments<'c>>::Arguments: IntoArguments<'c, DB>,
    E: for<'r> FromRow<'r, <DB as Database>::Row> + Schema<DB> + Bind<DB>,
    for<'c> i64: Type<DB> + Decode<'c, DB> + Encode<'c, DB>,
    usize: ColumnIndex<<DB as sqlx::Database>::Row>,
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

        let (id,) = query.bind(id).fetch_one(&self.db).await?;
        Ok(id)
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
/// Delete
///////////////////////////////////////////////////////////////////////////////////////////////////
impl<DB, E> Store<DB, E>
where
    DB: Database,
    for<'c> &'c mut <DB as sqlx::Database>::Connection: Executor<'c, Database = DB>,
    for<'c> <DB as HasArguments<'c>>::Arguments: IntoArguments<'c, DB>,
    for<'c> i64: Type<DB> + Encode<'c, DB>,
    E: Schema<DB>,
{
    fn delete_stmt(suffix: &str) -> String {
        let table = E::TABLE_NAME;
        format!("DELETE FROM {table} {suffix}")
    }

    /// Delete the object of type `E` corresponding to the provided `id`
    pub async fn delete(&self, id: i64) -> sqlx::Result<<DB as Database>::QueryResult> {
        let sql = Self::delete_stmt("WHERE id=$1");
        sqlx::query(&sql).bind(id).execute(&self.db).await
    }

    /// Delete all objects of type E
    pub async fn delete_all(&self) -> sqlx::Result<<DB as Database>::QueryResult> {
        let sql = Self::delete_stmt("");
        sqlx::query(&sql).execute(&self.db).await
    }
}
