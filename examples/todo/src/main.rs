use std::error::Error;

use async_trait::async_trait;
use dotenv::dotenv;
use miniorm::{Store, Table};
use miniorm_macros::Bind;
use sqlx::{FromRow, PgPool};

#[derive(Debug, Bind, Clone, FromRow, Eq, PartialEq)]
struct Todo {
    description: String,
    done: bool,
}

struct TodoStore;

#[async_trait]
impl Store<Todo> for TodoStore {
    const TABLE: Table = Table(
        "todo",
        &[
            ("description", "TEXT NOT NULL"),
            ("done", "BOOLEAN NOT NULL"),
        ],
    );
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let url = std::env::var("DATABASE_URL").expect("missing DATABASE_URL env");
    let db = PgPool::connect(&url).await?;

    let todo = Todo {
        description: "checkout miniorm".into(),
        done: false,
    };

    println!("Recreating table...");
    TodoStore::recreate_table(&db).await?;

    println!("Inserting...");
    let id = TodoStore::create(&db, &todo).await?;

    println!("Retrieveing by id...");
    let fetched = TodoStore::read(&db, id).await?;
    assert_eq!(todo, fetched);

    println!("Listing all...");
    let all = TodoStore::list(&db).await?;
    assert_eq!(all.len(), 1);
    assert_eq!(&todo, &all[0]);

    println!("Deleting by id...");
    let deleted = TodoStore::delete(&db, id).await?;
    assert_eq!(deleted, 1);

    println!("Checking delete successful");
    assert!(matches!(
        TodoStore::read(&db, id).await,
        Err(sqlx::Error::RowNotFound)
    ));

    Ok(())
}
