use async_trait::async_trait;

/// [C]reate CRUD operation
#[async_trait]
pub trait Create<E> {
    /// Create an object in the database and returns its `id`.
    async fn create(&self, entity: &E) -> sqlx::Result<i64>;
}

/// [R]ead CRUD operation
#[async_trait]
pub trait Read<E> {
    /// Reads and returns an object from the database
    async fn read(&self, id: i64) -> sqlx::Result<E>;

    /// Lists and return all object from the database
    async fn list(&self) -> sqlx::Result<Vec<E>>;
}

/// [C]reate CRUD operation
#[async_trait]
pub trait Update<E> {
    /// Update an object in the database and returns its `id`.
    async fn update(&self, id: i64, entity: &E) -> sqlx::Result<i64>;
}

/// [D]elete CRUD operation
#[async_trait]
pub trait Delete<E> {
    /// Delete the object of type `E` corresponding to the provided `id`
    async fn delete(&self, id: i64) -> sqlx::Result<()>;

    /// Delete all objects of type E and return the number of deleted rows
    async fn delete_all(&self) -> sqlx::Result<u64>;
}

/// CRUD operations
#[async_trait]
pub trait Crud<E>: Create<E> + Read<E> + Update<E> + Delete<E> {}
