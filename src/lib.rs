use sqlx::{
    postgres::{PgQueryResult, PgRow},
    prelude::FromRow,
    PgPool,
};
use std::marker::PhantomData;

pub use miniorm_macros::{Schema, ToRow};

pub mod traits {
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
}

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
    E: for<'r> FromRow<'r, PgRow> + traits::ToRow + traits::Schema,
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
