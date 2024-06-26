use miniorm::prelude::*;
use sqlx::FromRow;

/// A todo including a `description` and a `done` flag
#[derive(Debug, Clone, Eq, PartialEq, FromRow, Entity)]
struct Todo {
    #[sqlite(TEXT NOT NULL)]
    description: String,

    #[sqlite(BOOLEAN NOT NULL DEFAULT false)]
    done: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("== SQLITE ==");
    let db = sqlx::SqlitePool::connect(":memory:").await?;
    let store = miniorm::Store::new(db);

    let todo = Todo {
        description: "checkout miniorm".into(),
        done: false,
    };

    println!("Recreating table...");
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
