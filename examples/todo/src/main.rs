use std::error::Error;

use async_trait::async_trait;
use dotenv::dotenv;
use miniorm::{CrudStore, HasTable, Table};
use miniorm_macros::Bind;
use sqlx::{FromRow, PgPool};

#[derive(Debug, Bind, Clone, FromRow, Eq, PartialEq)]
struct Todo {
    description: String,
    done: bool,
}

#[async_trait]
impl HasTable for Todo {
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
    let store = CrudStore::<'_, Todo>::new(&db);

    let todo = Todo {
        description: "checkout miniorm".into(),
        done: false,
    };

    println!("Recreating table...");
    store.recreate_table().await?;

    println!("Inserting...");
    let id = store.create(&todo).await?;

    println!("Retrieveing by id...");
    let fetched = store.read(id).await?;
    assert_eq!(todo, fetched);

    println!("Listing all...");
    let all = store.list().await?;
    assert_eq!(all.len(), 1);
    assert_eq!(&todo, &all[0]);

    println!("Deleting by id...");
    let deleted = store.delete(id).await?;
    assert_eq!(deleted, 1);

    println!("Checking delete successful");
    assert!(matches!(
        store.read(id).await,
        Err(sqlx::Error::RowNotFound)
    ));

    Ok(())
}
