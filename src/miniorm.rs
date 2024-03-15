use async_trait::async_trait;
use itertools::Itertools;
use sqlx::{
    database::HasArguments,
    postgres::{PgQueryResult, PgRow},
    prelude::FromRow,
    query::QueryAs,
    Pool, Postgres,
};

pub type Db = Pool<Postgres>;
pub type TableName = &'static str;
pub type ColunmName = &'static str;
pub type ColumnType = &'static str;
pub type Column = (ColunmName, ColumnType);
pub type Columns = &'static [Column];
pub type PgQueryAs<'q, O> = QueryAs<'q, Postgres, O, <Postgres as HasArguments<'q>>::Arguments>;

pub trait Bind {
    fn bind<'q, O>(&self, query: PgQueryAs<'q, O>, column_name: ColunmName) -> PgQueryAs<'q, O>;
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

#[async_trait]
pub trait Store<E>
where
    E: for<'r> FromRow<'r, PgRow> + Send + Unpin + Bind + Sync,
{
    const TABLE: Table;

    async fn recreate(db: &Db) -> sqlx::Result<()> {
        Self::drop_table(db).await?;
        Self::create_table(db).await?;
        Ok(())
    }

    async fn create_table(db: &Db) -> sqlx::Result<PgQueryResult> {
        let sql = Self::TABLE.create_table();
        sqlx::query(&sql).execute(db).await
    }

    async fn drop_table(db: &Db) -> sqlx::Result<PgQueryResult> {
        let sql = Self::TABLE.drop_table();
        sqlx::query(&sql).execute(db).await
    }

    async fn create(db: &Db, entity: &E) -> sqlx::Result<i64> {
        let sql = Self::TABLE.insert();
        let mut query_as = sqlx::query_as(&sql);

        for col in Self::TABLE.columns().iter().map(|col| col.0) {
            query_as = entity.bind(query_as, col)
        }

        let (id,) = query_as.fetch_one(db).await?;
        Ok(id)
    }

    async fn read(db: &Db, id: i64) -> sqlx::Result<E> {
        let sql = Self::TABLE.select("WHERE id=$1");
        sqlx::query_as(&sql).bind(id).fetch_one(db).await
    }

    async fn list(db: &Db) -> sqlx::Result<Vec<E>> {
        let sql = Self::TABLE.select("ORDER BY id");
        sqlx::query_as(&sql).fetch_all(db).await
    }

    async fn update(db: &Db, id: i64, entity: E) -> sqlx::Result<i64>;

    async fn delete(db: &Db, id: i64) -> sqlx::Result<u64> {
        let sql = Self::TABLE.delete("WHERE id=$1");
        Ok(sqlx::query(&sql)
            .bind(id)
            .execute(db)
            .await?
            .rows_affected())
    }
}
