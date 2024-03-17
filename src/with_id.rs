use sqlx::{postgres::PgRow, FromRow, Row};

/// `WithId` is a wrapper of struct that provides an
/// additional id field which is used in the database
/// to identify the entity.
pub struct WithId<E> {
    /// the `id` used to identify the `inner` entity
    pub id: i64,
    /// the wrapped entity
    pub inner: E,
}

impl<E> WithId<E> {
    /// Creates a wrapped entity with the provided id.
    ///
    /// # Arguments
    /// - `inner`: the entity to be wrapped
    /// - `id`: the id
    ///
    /// # Example
    ///
    /// ```
    /// #[derive(Clone, Debug, Eq, PartialEq)]
    /// struct Todo {
    ///     description: String,
    ///     done: bool,
    /// }
    ///
    /// let todo = Todo{
    ///     description: "checkout miniorm".into(),
    ///     done: false
    /// };
    /// let todo_with_id = miniorm::WithId::new(todo.clone(), 1);
    /// assert_eq!(todo_with_id.id, 1);
    /// assert_eq!(todo_with_id.inner, todo);
    /// ```
    pub fn new(inner: E, id: i64) -> Self {
        WithId { inner, id }
    }
}

impl<'r, E> FromRow<'r, PgRow> for WithId<E>
where
    E: FromRow<'r, PgRow>,
{
    fn from_row(row: &'r PgRow) -> sqlx::Result<Self> {
        let inner = E::from_row(row)?;
        let id = row.try_get("id")?;
        Ok(Self { inner, id })
    }
}

impl<E: Clone> Clone for WithId<E> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            inner: self.inner.clone(),
        }
    }
}

impl<E: PartialEq> PartialEq for WithId<E> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.inner == other.inner
    }
}

impl<E: Eq> Eq for WithId<E> {}

impl<E: std::fmt::Debug> std::fmt::Debug for WithId<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WithId")
            .field("id", &self.id)
            .field("inner", &self.inner)
            .finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn is_clone_if_inner_is_clone() {
        #[derive(Clone)]
        struct Foo;

        let foo = WithId::new(Foo, 10);
        let _ = foo.clone();
    }

    #[test]
    fn is_eq_if_inner_is_eq_and_debug_if_inner_is_debug() {
        #[derive(Eq, PartialEq, Debug)]
        struct Foo(u8);

        let left = WithId::new(Foo(1), 10);
        let right = WithId::new(Foo(1), 10);
        assert_eq!(left, right);
        let right = WithId::new(Foo(1), 11);
        assert_ne!(left, right);
        let right = WithId::new(Foo(2), 10);
        assert_ne!(left, right);
    }
}
