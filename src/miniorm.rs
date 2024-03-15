use async_trait::async_trait;
use itertools::Itertools;
use sqlx::{postgres::PgRow, prelude::FromRow, Pool, Postgres};

pub type Db = Pool<Postgres>;
pub type TableName = &'static str;
pub type ColunmName = &'static str;
pub type ColumnType = &'static str;
pub type Column = (ColunmName, ColumnType);
pub type Columns = &'static [Column];

#[async_trait]
pub trait Table<E>
where
    E: for<'r> FromRow<'r, PgRow> + Send + Unpin,
{
    const TABLE_NAME: TableName;
    const COLUMNS: Columns;

    fn columns() -> String {
        Self::COLUMNS.iter().map(|col| col.0).join(", ")
    }

    fn schema() -> String {
        Self::COLUMNS
            .iter()
            .map(|col| format!("{} {}", col.0, col.1))
            .join(", ")
    }

    async fn recreate(db: &Db) -> sqlx::Result<()> {
        Self::drop(db).await?;
        Self::create(db).await?;
        Ok(())
    }

    async fn create(db: &Db) -> sqlx::Result<()> {
        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {} ({})",
            Self::TABLE_NAME,
            Self::schema()
        );
        sqlx::query(&sql).execute(db).await?;
        Ok(())
    }

    async fn drop(db: &Db) -> sqlx::Result<()> {
        let sql = format!("DROP TABLE IF EXISTS {}", Self::TABLE_NAME);
        sqlx::query(&sql).execute(db).await?;
        Ok(())
    }

    async fn add(db: &Db, entity: &E) -> sqlx::Result<i64>;

    async fn get(db: &Db, id: i64) -> sqlx::Result<E> {
        let sql = format!(
            "SELECT {} FROM {} WHERE id=$1",
            Self::columns(),
            Self::TABLE_NAME
        );
        sqlx::query_as(&sql).bind(id).fetch_one(db).await
    }

    async fn list(db: &Db) -> sqlx::Result<Vec<E>> {
        let sql = format!(
            "SELECT {} FROM {} ORDER BY id",
            Self::columns(),
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
