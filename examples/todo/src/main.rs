use miniorm::Schema;
use sqlx::FromRow;

/// A todo including a `description` and a `done` flag
#[derive(Debug, Clone, Eq, PartialEq, FromRow, Schema)]
struct Todo {
    #[column(TEXT NOT NULL)]
    description: String,

    #[column(BOOLEAN NOT NULL DEFAULT false)]
    done: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let url = std::env::var("DATABASE_URL").expect("missing DATABASE_URL env");
    let db = sqlx::PgPool::connect(&url).await?;
    let store = miniorm::CrudStore::<Todo>::new(db.clone());

    let todo = Todo {
        description: "checkout miniorm".into(),
        done: false,
    };

    println!("Recreating table...");
    store.recreate_table().await?;

    println!("Inserting...");
    let todo = store.create(todo).await?;

    println!("Retrieveing by id...");
    let mut fetched = store.read(todo.id).await?;
    assert_eq!(todo, fetched);

    fetched.inner.done = true;
    let fetched = store.update(fetched).await?;
    assert_eq!(fetched.id, todo.id);
    assert!(fetched.inner.done);

    println!("Listing all...");
    let all = store.list().await?;
    assert_eq!(all.len(), 1);
    assert_eq!(&fetched, &all[0]);

    println!("Deleting by id...");
    let res = store.delete(todo.id).await?;
    assert_eq!(res.rows_affected(), 1);

    println!("Checking delete successful");
    assert!(matches!(
        store.read(todo.id).await,
        Err(sqlx::Error::RowNotFound)
    ));

    Ok(())
}
