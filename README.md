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
to create a `CrudStore` that provide the so-called "CRUD" operations:
- (C)reate
- (R)ead
- (U)pdate
- (D)elete

At the moment, `miniorm` only supports the postgres backend. Other backends
could be provided in the future.

# Examples

```rust
use sqlx::FromRow;
use miniorm::Schema;

#[derive(Debug, Clone, Eq, PartialEq, FromRow, Schema)]
struct Todo {
    #[column(TEXT NOT NULL)]
    description: String,

    #[column(BOOLEAN NOT NULL DEFAULT false)]
    done: bool,
}

```

For more complete examples, see:
- [todo example](examples/todo/src/main.rs) for a simple example.
- [stock transaction example](examples/transactions/src/main.rs)
  for a more complex example, where certain fields are stored as
  [`JSONB`](https://www.postgresql.org/docs/current/datatype-json.html) column
  using [`serde_json`](https://docs.rs/serde_json/latest/serde_json/).

