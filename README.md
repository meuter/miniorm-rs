# miniorm

[![Build](https://github.com/meuter/miniorm/actions/workflows/build.yml/badge.svg)](https://github.com/meuter/miniorm/actions/workflows/build.yml)
[![Test](https://github.com/meuter/miniorm/actions/workflows/test.yml/badge.svg)](https://github.com/meuter/miniorm/actions/workflows/test.yml)
[![Clippy](https://github.com/meuter/miniorm/actions/workflows/clippy.yml/badge.svg)](https://github.com/meuter/miniorm/actions/workflows/clippy.yml)
[![Doc](https://github.com/meuter/miniorm/actions/workflows/docs.yml/badge.svg)](https://github.com/meuter/miniorm/actions/workflows/docs.yml)

[![Crates.io](https://img.shields.io/crates/v/miniorm)](https://crates.io/crates/miniorm)
[![Docs.rs](https://docs.rs/miniorm/badge.svg)](https://docs.rs/miniorm)
[![Crates.io](https://img.shields.io/crates/d/miniorm)](https://crates.io/crates/miniorm)
[![Crates.io](https://img.shields.io/crates/l/miniorm)](https://github.com/meuter/miniorm-rs/blob/main/LICENSE)

## Introduction

The `miniorm` crate provides a *very* simple
[ORM](https://en.wikipedia.org/wiki/Object%E2%80%93relational_mapping)
on top of [`sqlx`](https://docs.rs/sqlx/latest/sqlx/).

[`sqlx`](https://docs.rs/sqlx/latest/sqlx/) already provides a 
[`FromRow`](https://docs.rs/sqlx/latest/sqlx/trait.FromRow.html) trait 
that can be derived automatically in order to convert a row from the 
database into an object. However, there is no corresponding macro 
that would allow convert an object back into a row to be inserted into
the database.

This is where `miniorm` comes in. It provides multiple traits that 
can also be automatically derived. Using these traits, `miniorm` provides
a `Store` type that builds on top and all the standard "CRUD" operations:
- (C)reate
- (R)ead
- (U)pdate
- (D)elete

At the moment, `miniorm` supports the three most common database backend:
- Sqlite
- MySql
- Postgres

Each backend should be enabled using the corresponding feature flag.

## Example

```rust
use sqlx::FromRow;
use miniorm::prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, FromRow, Entity)]
struct Todo {
    #[column(TEXT NOT NULL)]
    description: String,

    #[column(BOOLEAN NOT NULL DEFAULT false)]
    done: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = sqlx::SqlitePool::connect(":memory:").await?;
    let store = miniorm::Store::new(db);

    let todo = Todo {
        description: "checkout miniorm".into(),
        done: false,
    };

    store.recreate_table().await?;

    println!("Inserting...");
    let todo = store.create(todo).await?;

    println!("Retrieveing by id...");
    let mut fetched = store.read(todo.id()).await?;
    assert_eq!(todo, fetched);

    println!("Updating by id...");
    fetched.done = true;
    let after_update = store.update(fetched).await?;
    assert_eq!(after_update.id(), todo.id());

    println!("Listing all...");
    let all = store.list().await?;
    assert_eq!(all.len(), 1);
    assert_eq!(&after_update, &all[0]);

    println!("Deleting by id...");
    store.delete(todo.id()).await?;

    println!("Checking delete successful");
    assert!(matches!(
        store.read(todo.id()).await,
        Err(sqlx::Error::RowNotFound)
    ));

    Ok(())
}
```

# But wait, there's more!

One can turn a `Store` into an [`Router`](https://docs.rs/axum/latest/axum/struct.Router.html)
that can be installed to serve these CRUD operations over a REST api.
For this your entity type should implement [`Serialize`](https://docs.rs/serde/latest/serde/trait.Serialize.html)
and [`Deserialize`](https://docs.rs/serde/latest/serde/trait.Deserialize.html) from
[`serde`](https://docs.rs/serde/latest/serde/index.html)

This requires the `axum` feature flag.

```rust, no_run
use axum::Router;
use miniorm::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[derive(Debug, Clone, Eq, PartialEq, FromRow, Entity, Serialize, Deserialize)]
struct Todo {
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // connect to db 
    let db = sqlx::SqlitePool::connect(":memory:").await?;

    // initialize todo store
    let todos = Store::new(db);
    todos.recreate_table().await?;
    todos.create(Todo::new("do the laundry")).await?;
    todos.create(Todo::new("wash the dishes")).await?;
    todos.create(Todo::new("go walk the dog")).await?;
    todos.create(Todo::new("groceries")).await?;

    // create the app
    let app = Router::new().nest("/todos", todos.into_axum_router());
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on http://{}", addr);

    // serve the app
    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await?;

    Ok(())
}
```

