use crate::{
    prelude::{BindColumn, Create, Delete, Read, Schema, Table, Update},
    traits::sqlx::{RowsAffected, SupportsReturning},
    WithId,
};
use async_trait::async_trait;
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

///////////////////////////////////////////////////////////////////////////////////////////////////
/// Table
///////////////////////////////////////////////////////////////////////////////////////////////////
#[async_trait]
impl<DB: Database, E: Sync + Schema<DB>> Table<DB> for Store<DB, E>
where
    for<'c> &'c mut <DB as sqlx::Database>::Connection: Executor<'c, Database = DB>,
    for<'c> <DB as HasArguments<'c>>::Arguments: IntoArguments<'c, DB>,
{
    async fn create_table(&self) -> sqlx::Result<<DB as Database>::QueryResult> {
        sqlx::query(E::MINIORM_CREATE_TABLE).execute(&self.db).await
    }

    async fn drop_table(&self) -> sqlx::Result<<DB as Database>::QueryResult> {
        sqlx::query(E::MINIORM_DROP_TABLE).execute(&self.db).await
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
/// Create
///////////////////////////////////////////////////////////////////////////////////////////////////
#[async_trait]
impl<DB, E> Create<E> for Store<DB, E>
where
    DB: Database + SupportsReturning,
    E: for<'r> FromRow<'r, <DB as Database>::Row> + Schema<DB> + BindColumn<DB> + Sync + Send,
    for<'c> &'c mut <DB as sqlx::Database>::Connection: Executor<'c, Database = DB>,
    for<'c> <DB as HasArguments<'c>>::Arguments: IntoArguments<'c, DB>,
    for<'c> i64: Type<DB> + Decode<'c, DB> + Encode<'c, DB>,
    usize: ColumnIndex<<DB as sqlx::Database>::Row>,
{
    async fn create(&self, entity: E) -> sqlx::Result<WithId<E,i64>> {
        let (id,) = E::MINIORM_COLUMNS
            .iter()
            .fold(sqlx::query_as(E::MINIORM_CREATE), |query, col| {
                entity.bind_column(query, col)
            })
            .fetch_one(&self.db)
            .await?;
        Ok(WithId::new(entity, id))
    }
}

#[cfg(feature = "mysql")]
mod mysql {
    use async_trait::async_trait;
    use sqlx::{mysql::MySqlRow, FromRow, MySql};

    use crate::{
        prelude::{BindColumn, Create, Schema},
        Store, WithId,
    };

    #[async_trait]
    impl<E> Create<E> for Store<MySql, E>
    where
        E: for<'r> FromRow<'r, MySqlRow> + Schema<MySql> + BindColumn<MySql> + Sync + Send,
    {
        async fn create(&self, entity: E) -> sqlx::Result<WithId<E,i64>> {
            let res = E::MINIORM_COLUMNS
                .iter()
                .fold(sqlx::query(E::MINIORM_CREATE), |query, col| {
                    entity.bind_column(query, col)
                })
                .execute(&self.db)
                .await?;
            let id = res.last_insert_id() as i64;
            Ok(WithId::new(entity, id))
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Read
///////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait]
impl<DB, E> Read<E> for Store<DB, E>
where
    DB: Database,
    E: Unpin + Send + Sync + Send,
    E: for<'r> FromRow<'r, <DB as Database>::Row> + Schema<DB>,
    for<'c> &'c str: ColumnIndex<<DB as Database>::Row>,
    for<'c> &'c mut <DB as sqlx::Database>::Connection: Executor<'c, Database = DB>,
    for<'c> <DB as HasArguments<'c>>::Arguments: IntoArguments<'c, DB>,
    for<'c> i64: Type<DB> + Decode<'c, DB> + Encode<'c, DB>,
{
    async fn read(&self, id: i64) -> sqlx::Result<WithId<E,i64>> {
        sqlx::query_as(E::MINIORM_READ)
            .bind(id)
            .fetch_one(&self.db)
            .await
    }

    async fn list(&self) -> sqlx::Result<Vec<WithId<E,i64>>> {
        sqlx::query_as(E::MINIORM_LIST).fetch_all(&self.db).await
    }

    async fn count(&self) -> sqlx::Result<u64> {
        #[derive(FromRow)]
        struct CountResult {
            count: i64,
        }

        let result: CountResult = sqlx::query_as(E::MINIORM_COUNT).fetch_one(&self.db).await?;
        Ok(result.count as u64)
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
/// Update
///////////////////////////////////////////////////////////////////////////////////////////////////
#[async_trait]
impl<DB, E> Update<E> for Store<DB, E>
where
    DB: Database,
    for<'c> &'c mut <DB as sqlx::Database>::Connection: Executor<'c, Database = DB>,
    for<'c> <DB as HasArguments<'c>>::Arguments: IntoArguments<'c, DB>,
    E: for<'r> FromRow<'r, <DB as Database>::Row> + Schema<DB> + BindColumn<DB> + Sync + Send,
    for<'c> i64: Type<DB> + Decode<'c, DB> + Encode<'c, DB>,
    usize: ColumnIndex<<DB as sqlx::Database>::Row>,
{
    async fn update(&self, entity: WithId<E,i64>) -> sqlx::Result<WithId<E, i64>> {
        E::MINIORM_COLUMNS
            .iter()
            .fold(sqlx::query(E::MINIORM_UPDATE), |query, col| {
                entity.bind_column(query, col)
            })
            .bind(entity.id())
            .execute(&self.db)
            .await?;
        Ok(entity)
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
/// Delete
///////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait]
impl<DB, E> Delete<E> for Store<DB, E>
where
    DB: Database,
    E: Schema<DB> + Sync,
    <DB as Database>::QueryResult: RowsAffected,
    for<'c> &'c mut <DB as sqlx::Database>::Connection: Executor<'c, Database = DB>,
    for<'c> <DB as HasArguments<'c>>::Arguments: IntoArguments<'c, DB>,
    for<'c> i64: Type<DB> + Encode<'c, DB>,
{
    async fn delete(&self, id: i64) -> sqlx::Result<()> {
        let res = sqlx::query(E::MINIORM_DELETE)
            .bind(id)
            .execute(&self.db)
            .await?;
        if res.rows_affected() == 0 {
            Err(sqlx::Error::RowNotFound)
        } else {
            Ok(())
        }
    }

    async fn delete_all(&self) -> sqlx::Result<u64> {
        let res = sqlx::query(E::MINIORM_DELETE_ALL).execute(&self.db).await?;
        Ok(res.rows_affected() as u64)
    }
}

#[cfg(feature = "axum")]
impl<DB: Database, E> crate::traits::axum::IntoAxumRouter for Store<DB, E>
where
    E: Schema<DB>
        + for<'r> FromRow<'r, <DB as Database>::Row>
        + crate::traits::bind_col::BindColumn<DB>,
    E: serde::Serialize + for<'de> serde::Deserialize<'de>,
    E: Clone + Sync + Send + Unpin + 'static,
    Store<DB, E>: crate::traits::crud::Crud<E> + Clone,
{
    fn into_axum_router<S>(self) -> axum::Router<S> {
        crate::handler::Handler::new(self).into_axum_router()
    }
}

impl<DB: Database, E> Clone for Store<DB, E> {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            entity: self.entity,
        }
    }
}
