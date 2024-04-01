use async_trait::async_trait;

/// All the necessary CRUD operations
#[async_trait]
pub trait Crud<E> {
    /// Create an object in the database and returns its `id`.
    async fn create(&self, entity: &E) -> sqlx::Result<i64>;
}
