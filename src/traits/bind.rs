use sqlx::{
    database::{Database, HasArguments},
    query::{Query, QueryAs},
    Encode, Type,
};

/// Trait that can be implemented on a `struct` to bind the fields of
/// of this struct with a [sqlx::query::QueryAs] or a [sqlx::query::Query].
///
/// # Example
///
/// ```
/// use miniorm::{Bind, BindableQuery};
/// use sqlx::Postgres;
///
/// struct Todo {
///     description: String,
///     done: bool,
/// }
///
/// impl Bind<Postgres> for Todo {
///     fn bind<'q, Q>(&self, query: Q, column_name: &'static str) -> Q
///     where
///         Q: ::miniorm::BindableQuery<'q, Postgres> {
///         match column_name {
///             "description" => query.bind(self.description.clone()),
///             "done" => query.bind(self.done.clone()),
///             _ => query,
///         }
///     }
/// }
///
/// ```
///
/// This trait can be derived automatically using the [Entity](miniorm_macros::Entity)
/// derive macro.
///
pub trait Bind<DB: Database> {
    /// binds a specific column using the provided query.
    fn bind<'q, Q>(&self, query: Q, column_name: &'static str) -> Q
    where
        Q: BindableQuery<'q, DB>;
}

/// Trait to abstract calls to a `bind` method, to generalize
/// [`Query`] and [`QueryAs`].
pub trait BindableQuery<'q, DB>
where
    DB: Database,
{
    /// Bind a value for use with this SQL query.
    fn bind<T>(self, value: T) -> Self
    where
        T: 'q + Send + Encode<'q, DB> + Type<DB>;
}

impl<'q, DB> BindableQuery<'q, DB> for Query<'q, DB, <DB as HasArguments<'q>>::Arguments>
where
    DB: Database,
{
    fn bind<T>(self, value: T) -> Self
    where
        T: 'q + Send + Encode<'q, DB> + Type<DB>,
    {
        Query::bind(self, value)
    }
}

impl<'q, DB, O> BindableQuery<'q, DB> for QueryAs<'q, DB, O, <DB as HasArguments<'q>>::Arguments>
where
    DB: Database,
{
    fn bind<T>(self, value: T) -> Self
    where
        T: 'q + Send + Encode<'q, DB> + Type<DB>,
    {
        QueryAs::bind(self, value)
    }
}
