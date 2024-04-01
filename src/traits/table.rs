use async_trait::async_trait;
use sqlx::Database;

/// Trait representing the fact that this object can create and drop
/// tables on a specific database type.
#[async_trait]
pub trait Table<DB: Database> {
    /// Recreates the table associated with the entity's [`Schema`]
    async fn recreate_table(&self) -> sqlx::Result<<DB as Database>::QueryResult> {
        self.drop_table().await?;
        self.create_table().await
    }

    /// Creates the table associated with the entity's [`Schema`]
    async fn create_table(&self) -> sqlx::Result<<DB as Database>::QueryResult>;

    /// Drops the table associated with the entity's [`Schema`]
    async fn drop_table(&self) -> sqlx::Result<<DB as Database>::QueryResult>;
}
