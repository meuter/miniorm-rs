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
#[cfg(feature = "postgres")]
mod postgres {
    use crate::{Bind, Crud, Schema, Store};
    use async_trait::async_trait;
    use sqlx::{postgres::PgRow, FromRow, Postgres};

    #[async_trait]
    impl<E> Crud<E> for Store<Postgres, E>
    where
        E: for<'r> FromRow<'r, PgRow> + Schema<Postgres> + Bind<Postgres> + Sync,
    {
        async fn create(&self, entity: &E) -> sqlx::Result<i64> {
            let mut query = sqlx::query_as(E::MINIORM_CREATE);
            for col in E::COLUMNS.iter().map(|col| col.0) {
                query = entity.bind(query, col)
            }
            let (id,) = query.fetch_one(&self.db).await?;
            Ok(id)
        }
    }
}

#[cfg(feature = "sqlite")]
mod sqlite {
    use crate::{Bind, Crud, Schema, Store};
    use async_trait::async_trait;
    use sqlx::{sqlite::SqliteRow, FromRow, Sqlite};

    #[async_trait]
    impl<E> Crud<E> for Store<Sqlite, E>
    where
        E: for<'r> FromRow<'r, SqliteRow> + Schema<Sqlite> + Bind<Sqlite> + Sync,
    {
        async fn create(&self, entity: &E) -> sqlx::Result<i64> {
            let mut query = sqlx::query_as(E::MINIORM_CREATE);
            for col in E::COLUMNS.iter().map(|col| col.0) {
                query = entity.bind(query, col)
            }
            let (id,) = query.fetch_one(&self.db).await?;
            Ok(id)
        }
    }
}

#[cfg(feature = "mysql")]
mod mysql {
    use crate::{Bind, Crud, Schema, Store};
    use async_trait::async_trait;
    use sqlx::{mysql::MySqlRow, FromRow, MySql};

    #[async_trait]
    impl<E> Crud<E> for Store<MySql, E>
    where
        E: for<'r> FromRow<'r, MySqlRow> + Schema<MySql> + Bind<MySql> + Sync,
    {
        async fn create(&self, entity: &E) -> sqlx::Result<i64> {
            let mut query = sqlx::query(E::MINIORM_CREATE);
            for col in E::COLUMNS.iter().map(|col| col.0) {
                query = entity.bind(query, col)
            }
            let res = query.execute(&self.db).await?;
            Ok(res.last_insert_id() as i64)
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
    /// Reads and returns an object from the database
    pub async fn read(&self, id: i64) -> sqlx::Result<E> {
        sqlx::query_as(E::MINIORM_READ)
            .bind(id)
            .fetch_one(&self.db)
            .await
    }

    /// Lists and return all object from the database
    pub async fn list(&self) -> sqlx::Result<Vec<E>> {
        sqlx::query_as(E::MINIORM_LIST).fetch_all(&self.db).await
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
    /// Delete the object of type `E` corresponding to the provided `id`
    pub async fn delete(&self, id: i64) -> sqlx::Result<<DB as Database>::QueryResult> {
        sqlx::query(E::MINIORM_DELETE)
            .bind(id)
            .execute(&self.db)
            .await
    }

    /// Delete all objects of type E
    pub async fn delete_all(&self) -> sqlx::Result<<DB as Database>::QueryResult> {
        sqlx::query(E::MINIORM_DELETE_ALL).execute(&self.db).await
    }
}
