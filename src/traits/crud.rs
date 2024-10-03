use async_trait::async_trait;

use crate::WithId;

/// \[C\]reate CRUD operation
#[async_trait]
pub trait Create<E, I> {
    /// Create an object in the database and returns its `id`.
    async fn create(&self, entity: E) -> sqlx::Result<WithId<E, I>>;
}

/// \[R\]ead CRUD operation
#[async_trait]
pub trait Read<E, I> {
    /// Reads and returns an object from the database
    async fn read(&self, id: I) -> sqlx::Result<WithId<E, I>>;

    /// Lists and return all object from the database
    async fn list(&self) -> sqlx::Result<Vec<WithId<E, I>>>;

    /// Count and return the number of object in the database
    async fn count(&self) -> sqlx::Result<u64>;
}

/// \[U\]pdate CRUD operation
#[async_trait]
pub trait Update<E, I> {
    /// Update an object in the database and returns its `id`.
    async fn update(&self, entity: WithId<E, I>) -> sqlx::Result<WithId<E, I>>;
}

/// \[D\]elete CRUD operation
#[async_trait]
pub trait Delete<E, I> {
    /// Delete the object of type `E` corresponding to the provided `id`
    async fn delete(&self, id: I) -> sqlx::Result<()>;

    /// Delete all objects of type E and return the number of deleted rows
    async fn delete_all(&self) -> sqlx::Result<u64>;
}

/// CRUD operations
#[async_trait]
pub trait Crud<E, I>: Create<E, I> + Read<E, I> + Update<E, I> + Delete<E, I> {}

impl<S, E, I> Crud<E, I> for S where S: Create<E, I> + Read<E, I> + Update<E, I> + Delete<E, I> {}
