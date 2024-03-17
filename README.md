# miniorm

[![Build](https://github.com/meuter/miniorm/actions/workflows/build.yml/badge.svg)](https://github.com/meuter/miniorm/actions/workflows/build.yml)
[![Test](https://github.com/meuter/miniorm/actions/workflows/test.yml/badge.svg)](https://github.com/meuter/miniorm/actions/workflows/test.yml)
[![Clippy](https://github.com/meuter/miniorm/actions/workflows/clippy.yml/badge.svg)](https://github.com/meuter/miniorm/actions/workflows/clippy.yml)
[![Doc](https://github.com/meuter/miniorm/actions/workflows/docs.yml/badge.svg)](https://github.com/meuter/miniorm/actions/workflows/docs.yml)

[![Crates.io](https://img.shields.io/crates/v/miniorm)](https://crates.io/crates/miniorm)
[![Docs.rs](https://docs.rs/miniorm/badge.svg)](https://docs.rs/miniorm)
[![Crates.io](https://img.shields.io/crates/d/miniorm)](https://crates.io/crates/miniorm)
[![Crates.io](https://img.shields.io/crates/l/miniorm)](https://github.com/meuter/miniorm-rs/blob/main/LICENSE)

The `miniorm` crate provides a *very* simple
[ORM](https://en.wikipedia.org/wiki/Object%E2%80%93relational_mapping)
on top of [`sqlx`](https://docs.rs/sqlx/latest/sqlx/).

[`sqlx`](https://docs.rs/sqlx/latest/sqlx/) already provides a 
[`FromRow`](https://docs.rs/sqlx/latest/sqlx/trait.FromRow.html) trait 
that can be derived automatically in order to convert a row from the 
database into an object. Howver, there is no corresponding `ToRow` macro 
that would allow convert an object back into a row to be inserted into
the database.

This is where `miniorm` comes in. It provides a trait `Schema`
that can also be automatically derived to describe the schema
of the table that should be used for a given entity (i.e. `struct`).

Any struct that implements the `FromRow` and `Schema` traits can be used 
to create a `Store` that provide the so-called "CRUD" operations:
- (C)reate
- (R)ead
- (U)pdate
- (D)elete

You can even create an Axum router that will allow so serve these CRUD 
operations via a REST API.

At the moment, `miniorm` only supports the postgres backend. Other backends
could be provided in the future.

# Examples

## Todo

```rust
mod todo {
    use sqlx::FromRow;
    use miniorm::Schema;

    #[derive(Debug, Clone, Eq, PartialEq, FromRow, Schema)]
    pub struct Todo {
        #[column(TEXT NOT NULL)]
        description: String,

        #[column(BOOLEAN NOT NULL DEFAULT false)]
        done: bool,
    }

    impl Todo {
        pub fn new(description: impl AsRef<str>) -> Self {
            let description = description.as_ref().to_string();
            let done = false;
            Todo { description, done }
        }
        
        pub fn is_done(&self) -> bool {
            self.done
        }

        pub fn description(&self) -> &str {
            &self.description
        }

        pub fn mark_as_done(&mut self) {
            self.done = true;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use todo::Todo;

    // connect to postgres
    dotenv::dotenv()?;
    let url = std::env::var("DATABASE_URL").expect("missing DATABASE_URL env");
    let db = sqlx::PgPool::connect(&url).await?;

    // create a store
    let store = miniorm::Store::new(db);
    store.recreate_table().await?;

    // [C]reate a todo
    let created = store.create(Todo::new("checkout miniorm")).await?;
    assert_eq!(created.id(), 1);
    assert_eq!(created.description(), "checkout miniorm");
    assert_eq!(created.is_done(), false);

    // [R]ead a todo
    let mut fetched = store.read(created.id()).await?;
    assert_eq!(fetched, created);
    assert_eq!(fetched.is_done(), false);

    // mark todo as done
    fetched.mark_as_done();
    assert_eq!(fetched.is_done(), true);

    // the change is not reflected in db yet
    assert_eq!(store.read(fetched.id()).await?.is_done(), false);

    // [U]pdate a todo
    let updated = store.update(fetched).await?;
    assert_eq!(updated.id(), created.id());
    assert_eq!(updated.is_done(), true);

    // now the change is reflected in the db
    assert_eq!(store.read(updated.id()).await?.is_done(), true);

    // [D]elete the todo
    let res = store.delete(updated.id()).await?;
    assert_eq!(res.rows_affected(), 1);

    Ok(())
}
```

## Todo REST API with Axum

```rust
mod todo {
    use miniorm::Schema;
    use serde::{Deserialize, Serialize};
    use sqlx::FromRow;

    #[derive(Debug, Clone, Eq, PartialEq, FromRow, Schema, Serialize, Deserialize)] // <- required for the REST API
    pub struct Todo {
        #[column(TEXT NOT NULL)]
        description: String,

        #[column(BOOLEAN NOT NULL DEFAULT false)]
        done: bool,
    }

    impl Todo {
        pub fn new(description: impl AsRef<str>) -> Self {
            let description = description.as_ref().to_string();
            let done = false;
            Todo { description, done }
        }

        pub fn description(&self) -> &str {
            &self.description
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use todo::Todo;
    use tower::ServiceExt;

    // connect to postgres
    dotenv::dotenv()?;
    let url = std::env::var("DATABASE_URL").expect("missing DATABASE_URL env");
    let db = sqlx::PgPool::connect(&url).await?;

    // create a store and seed with somei todos
    let store = miniorm::Store::new(db);
    store.recreate_table().await?;
    store.create(Todo::new("do the laundry")).await?;
    store.create(Todo::new("wash the dishes")).await?;
    store.create(Todo::new("go walk the dog")).await?;
    store.create(Todo::new("groceries")).await?;

    // create the app to serve the REST api on `/todos`
    let app = axum::Router::new().nest("/todos", store.into_axum_router());

    // `GET /todos/3`
    let request = Request::builder().uri("/todos/3").body(Body::empty())?;
    let response = app.oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);

    // check todo
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let todo: Todo = serde_json::from_slice(&body).unwrap();
    assert_eq!(todo.description(), "go walk the dog");

    Ok(())
}

```

