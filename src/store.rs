use crate::traits::Schema;
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
        let sql = E::create_table();
        sqlx::query(&sql).execute(self.db).await
    }

    /// Drops the table associated with the entity's [`Schema`]
    pub async fn drop_table(&self) -> sqlx::Result<PgQueryResult> {
        let sql = E::drop_table();
        sqlx::query(&sql).execute(self.db).await
    }

    /// Delete the object of type `E` corresponding to the provided `id`
    pub async fn delete(&self, id: i64) -> sqlx::Result<PgQueryResult> {
        let sql = E::delete("WHERE id=$1");
        sqlx::query(&sql).bind(id).execute(self.db).await
    }

    /// Delete all objects of type E
    pub async fn delete_all(&self) -> sqlx::Result<u64> {
        let sql = E::delete("");
        Ok(sqlx::query(&sql).execute(self.db).await?.rows_affected())
    }
}

impl<'d, E> CrudStore<'d, E>
where
    E: for<'r> FromRow<'r, PgRow> + Schema + Unpin + Send,
{
    pub async fn read(&self, id: i64) -> sqlx::Result<E> {
        let sql = E::select("WHERE id=$1");
        sqlx::query_as(&sql).bind(id).fetch_one(self.db).await
    }

    pub async fn list(&self) -> sqlx::Result<Vec<E>> {
        let sql = E::select("ORDER BY id");
        sqlx::query_as(&sql).fetch_all(self.db).await
    }
}

impl<'d, E> CrudStore<'d, E>
where
    E: for<'r> FromRow<'r, PgRow> + Schema,
{
    pub async fn create(&self, entity: &E) -> sqlx::Result<i64> {
        let sql = E::insert();
        let mut query = sqlx::query_as(&sql);

        for col in E::COLUMNS.iter().map(|col| col.0) {
            query = entity.bind(query, col)
        }

        let (id,) = query.fetch_one(self.db).await?;
        Ok(id)
    }

    pub async fn update(&self, id: i64, entity: &E) -> sqlx::Result<i64> {
        let sql = E::update();
        let mut query = sqlx::query_as(&sql);

        for col in E::COLUMNS.iter().map(|col| col.0) {
            query = entity.bind(query, col)
        }

        let (id,) = query.bind(id).fetch_one(self.db).await?;
        Ok(id)
    }
}
