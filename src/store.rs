use crate::traits;
use sqlx::{
    postgres::{PgQueryResult, PgRow},
    prelude::FromRow,
    PgPool,
};
use std::marker::PhantomData;

pub struct CrudStore<'d, E> {
    db: &'d PgPool,
    entity: PhantomData<E>,
}

impl<'d, E> CrudStore<'d, E> {
    pub fn new(db: &'d PgPool) -> Self {
        let entity = PhantomData;
        Self { db, entity }
    }
}

impl<'d, E> CrudStore<'d, E>
where
    E: traits::Schema,
{
    pub async fn recreate_table(&self) -> sqlx::Result<PgQueryResult> {
        self.drop_table().await?;
        self.create_table().await
    }

    pub async fn create_table(&self) -> sqlx::Result<PgQueryResult> {
        let sql = E::create_table();
        sqlx::query(&sql).execute(self.db).await
    }

    pub async fn drop_table(&self) -> sqlx::Result<PgQueryResult> {
        let sql = E::drop_table();
        sqlx::query(&sql).execute(self.db).await
    }

    pub async fn delete(&self, id: i64) -> sqlx::Result<u64> {
        let sql = E::delete("WHERE id=$1");
        Ok(sqlx::query(&sql)
            .bind(id)
            .execute(self.db)
            .await?
            .rows_affected())
    }

    pub async fn delete_all(&self) -> sqlx::Result<u64> {
        let sql = E::delete("");
        Ok(sqlx::query(&sql).execute(self.db).await?.rows_affected())
    }
}

impl<'d, E> CrudStore<'d, E>
where
    E: for<'r> FromRow<'r, PgRow> + traits::Schema + Unpin + Send,
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
    E: for<'r> FromRow<'r, PgRow> + traits::Schema,
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
