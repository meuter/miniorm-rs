use async_trait::async_trait;
use itertools::Itertools;
use sqlx::{
    database::HasArguments, postgres::PgRow, prelude::FromRow, query::QueryAs, Pool, Postgres,
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

struct Schema(TableName, Columns);

impl Schema {
    fn columns(&self) -> String {
        self.1.iter().map(|col| col.0).join(", ")
    }
}

#[async_trait]
pub trait Table<E>
where
    E: for<'r> FromRow<'r, PgRow> + Send + Unpin + Bind + Sync,
{
    // const SCHEMA: Schema;
    const TABLE_NAME: TableName;
    const COLUMNS: Columns;

    fn values() -> String {
        (1..=Self::COLUMNS.len())
            .map(|i| format!("${i}"))
            .join(", ")
    }

    fn columns_including_id() -> String {
        let helper = Schema(Self::TABLE_NAME, Self::COLUMNS);
        "id, ".to_string() + &helper.columns()
    }

    fn schema() -> String {
        let id = "id BIGSERIAL PRIMARY KEY,".to_string();
        let other_cols = Self::COLUMNS
            .iter()
            .map(|col| format!("{} {}", col.0, col.1))
            .join(", ");
        id + &other_cols
    }

    async fn recreate(db: &Db) -> sqlx::Result<()> {
        Self::drop_table(db).await?;
        Self::create_table(db).await?;
        Ok(())
    }

    async fn create_table(db: &Db) -> sqlx::Result<()> {
        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {} ({})",
            Self::TABLE_NAME,
            Self::schema()
        );
        sqlx::query(&sql).execute(db).await?;
        Ok(())
    }

    async fn drop_table(db: &Db) -> sqlx::Result<()> {
        let sql = format!("DROP TABLE IF EXISTS {}", Self::TABLE_NAME);
        sqlx::query(&sql).execute(db).await?;
        Ok(())
    }

    async fn create(db: &Db, entity: &E) -> sqlx::Result<i64> {
        let helper = Schema(Self::TABLE_NAME, Self::COLUMNS);
        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({}) RETURNING ID",
            Self::TABLE_NAME,
            helper.columns(),
            Self::values(),
        );
        let mut query_as = sqlx::query_as(&sql);

        for col in Self::COLUMNS {
            query_as = entity.bind(query_as, col.0)
        }

        let (id,) = query_as.fetch_one(db).await?;
        Ok(id)
    }

    async fn read(db: &Db, id: i64) -> sqlx::Result<E> {
        let sql = format!(
            "SELECT {} FROM {} WHERE id=$1",
            Self::columns_including_id(),
            Self::TABLE_NAME
        );
        sqlx::query_as(&sql).bind(id).fetch_one(db).await
    }

    async fn list(db: &Db) -> sqlx::Result<Vec<E>> {
        let sql = format!(
            "SELECT {} FROM {} ORDER BY id",
            Self::columns_including_id(),
            Self::TABLE_NAME,
        );
        sqlx::query_as(&sql).fetch_all(db).await
    }

    async fn update(db: &Db, id: i64, entity: E) -> sqlx::Result<i64>;

    async fn delete(db: &Db, id: i64) -> sqlx::Result<u64> {
        let sql = format!("DELETE FROM {} WHERE id=$1", Self::TABLE_NAME);
        Ok(sqlx::query(&sql)
            .bind(id)
            .execute(db)
            .await?
            .rows_affected())
    }
}
