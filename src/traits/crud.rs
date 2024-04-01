use async_trait::async_trait;

/// [C]reate CRUD operation
#[async_trait]
pub trait Create<E> {
    /// Create an object in the database and returns its `id`.
    async fn create(&self, entity: &E) -> sqlx::Result<i64>;
}

/// [C]reate CRUD operation
#[async_trait]
pub trait Update<E> {
    /// Update an object in the database and returns its `id`.
    async fn update(&self, id: i64, entity: &E) -> sqlx::Result<i64>;
}
