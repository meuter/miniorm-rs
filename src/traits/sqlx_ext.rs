use sqlx::{
    database::HasArguments,
    query::{Query, QueryAs},
    Database, Encode, Type,
};

/// Trait that should arguable be in [`sqlx`] denoting all types of queries
/// ([`Query`] and [`QueryAs`]) on which the `bind` method can be called.
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

/// Trait that should arguably be in [`sqlx`] denoting all types of query results
/// ([`PgRow`], [`SqliteRow`] and [`MySql`]) on which the `rows_affected` can be called.
pub trait RowsAffected {
    /// Returns the number of affected rows
    fn rows_affected(&self) -> u64;
}

#[cfg(feature = "postgres")]
impl RowsAffected for sqlx::postgres::PgQueryResult {
    fn rows_affected(&self) -> u64 {
        sqlx::postgres::PgQueryResult::rows_affected(self)
    }
}

#[cfg(feature = "mysql")]
impl RowsAffected for sqlx::mysql::MySqlQueryResult {
    fn rows_affected(&self) -> u64 {
        sqlx::mysql::MySqlQueryResult::rows_affected(self)
    }
}

#[cfg(feature = "sqlite")]
impl RowsAffected for sqlx::sqlite::SqliteQueryResult {
    fn rows_affected(&self) -> u64 {
        sqlx::sqlite::SqliteQueryResult::rows_affected(self)
    }
}
