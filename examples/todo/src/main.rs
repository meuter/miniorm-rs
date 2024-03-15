use std::error::Error;

use dotenv::dotenv;
use miniorm::{CrudStore, HasTable, ToRow};
use sqlx::{FromRow, PgPool};

#[derive(Debug, ToRow, Clone, FromRow, Eq, PartialEq, HasTable)]
struct Todo {
    #[column(TEXT NOT NULL)]
    description: String,
    #[column(BOOLEAN NOT NULL DEFAULT false)]
    done: bool,
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
    let mut fetched = store.read(id).await?;
    assert_eq!(todo, fetched);

    fetched.done = true;
    let id_after_update = store.update(id, &fetched).await?;
    assert_eq!(id_after_update, id);

    println!("Listing all...");
    let all = store.list().await?;
    assert_eq!(all.len(), 1);
    assert_eq!(&fetched, &all[0]);

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
