use itertools::Itertools;
use sqlx::{
    database::HasArguments,
    postgres::{PgQueryResult, PgRow},
    prelude::FromRow,
    query::QueryAs,
    Pool, Postgres,
};
use std::marker::PhantomData;

pub use miniorm_macros::{HasTable, ToRow};

pub type Db = Pool<Postgres>;
pub type TableName = &'static str;
pub type ColunmName = &'static str;
pub type ColumnType = &'static str;
pub type Column = (ColunmName, ColumnType);
pub type Columns = &'static [Column];
pub type PgQueryAs<'q, O> = QueryAs<'q, Postgres, O, <Postgres as HasArguments<'q>>::Arguments>;

pub mod traits {
    use super::*;

    pub trait ToRow {
        fn bind<'q, O>(&self, query: PgQueryAs<'q, O>, column_name: ColunmName)
            -> PgQueryAs<'q, O>;
    }

    pub trait HasTable {
        const TABLE: Table;
    }
}

pub struct Table(pub TableName, pub Columns);

impl Table {
    fn table(&self) -> TableName {
        self.0
    }

    fn columns(&self) -> Columns {
        self.1
    }

    fn comma_seperated_columns(&self) -> String {
        self.columns().iter().map(|col| col.0).join(", ")
    }

    fn create_table(&self) -> String {
        let table = self.table();
        let id = "id BIGSERIAL PRIMARY KEY";
        let cols = self
            .columns()
            .iter()
            .map(|col| format!("{} {}", col.0, col.1))
            .join(", ");
        format!("CREATE TABLE IF NOT EXISTS {table} ({id}, {cols})")
    }

    fn drop_table(&self) -> String {
        let table = self.table();
        format!("DROP TABLE IF EXISTS {table}")
    }

    fn insert(&self) -> String {
        let table = self.table();
        let cols = self.comma_seperated_columns();
        let values = (1..=self.columns().len())
            .map(|i| format!("${i}"))
            .join(", ");
        format!("INSERT INTO {table} ({cols}) VALUES ({values}) RETURNING id")
    }

    fn update(&self) -> String {
        let table = self.table();
        let values = self
            .columns()
            .iter()
            .map(|col| col.0)
            .enumerate()
            .map(|(i, col)| format!("{col}=${}", i + 1))
            .join(", ");
        let suffix = format!("WHERE id=${}", self.columns().len() + 1);
        format!("UPDATE {table} SET {values} {suffix} RETURNING id")
    }

    fn select(&self, suffix: &str) -> String {
        let table = self.table();
        let cols = self.comma_seperated_columns();
        format!("SELECT {cols} FROM {table} {suffix}")
    }

    fn delete(&self, suffix: &str) -> String {
        let table = self.table();
        format!("DELETE FROM {table} {suffix}")
    }
}

pub struct CrudStore<'d, E> {
    db: &'d Db,
    entity: PhantomData<E>,
}

impl<'d, E> CrudStore<'d, E> {
    pub fn new(db: &'d Db) -> Self {
        let entity = PhantomData;
        Self { db, entity }
    }
}

impl<'d, E> CrudStore<'d, E>
where
    E: traits::HasTable,
{
    pub async fn recreate_table(&self) -> sqlx::Result<PgQueryResult> {
        self.drop_table().await?;
        self.create_table().await
    }

    pub async fn create_table(&self) -> sqlx::Result<PgQueryResult> {
        let sql = E::TABLE.create_table();
        sqlx::query(&sql).execute(self.db).await
    }

    pub async fn drop_table(&self) -> sqlx::Result<PgQueryResult> {
        let sql = E::TABLE.drop_table();
        sqlx::query(&sql).execute(self.db).await
    }

    pub async fn delete(&self, id: i64) -> sqlx::Result<u64> {
        let sql = E::TABLE.delete("WHERE id=$1");
        Ok(sqlx::query(&sql)
            .bind(id)
            .execute(self.db)
            .await?
            .rows_affected())
    }

    pub async fn delete_all(&self) -> sqlx::Result<u64> {
        let sql = E::TABLE.delete("");
        Ok(sqlx::query(&sql).execute(self.db).await?.rows_affected())
    }
}

impl<'d, E> CrudStore<'d, E>
where
    E: for<'r> FromRow<'r, PgRow> + traits::HasTable + Unpin + Send,
{
    pub async fn read(&self, id: i64) -> sqlx::Result<E> {
        let sql = E::TABLE.select("WHERE id=$1");
        sqlx::query_as(&sql).bind(id).fetch_one(self.db).await
    }

    pub async fn list(&self) -> sqlx::Result<Vec<E>> {
        let sql = E::TABLE.select("ORDER BY id");
        sqlx::query_as(&sql).fetch_all(self.db).await
    }
}

impl<'d, E> CrudStore<'d, E>
where
    E: for<'r> FromRow<'r, PgRow> + traits::ToRow + traits::HasTable,
{
    pub async fn create(&self, entity: &E) -> sqlx::Result<i64> {
        let sql = E::TABLE.insert();
        let mut query = sqlx::query_as(&sql);

        for col in E::TABLE.columns().iter().map(|col| col.0) {
            query = entity.bind(query, col)
        }

        let (id,) = query.fetch_one(self.db).await?;
        Ok(id)
    }

    pub async fn update(&self, id: i64, entity: &E) -> sqlx::Result<i64> {
        let sql = E::TABLE.update();
        let mut query = sqlx::query_as(&sql);

        for col in E::TABLE.columns().iter().map(|col| col.0) {
            query = entity.bind(query, col)
        }

        let (id,) = query.bind(id).fetch_one(self.db).await?;
        Ok(id)
    }
}
