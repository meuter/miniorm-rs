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

struct Todo2 {
    description: String,
    done: bool,
}

impl miniorm::traits::Schema for Todo2 {
    const TABLE_NAME: &'static str = "todo";
    const COLUMNS: &'static [(&'static str, &'static str)] = &[
        ("description", "TEXT NOT NULL"),
        ("done", "BOOLEAN NOT NULL"),
    ];

    fn bind<'q, O>(
        &self,
        query: miniorm::traits::Query<'q, O>,
        column_name: &'static str,
    ) -> miniorm::traits::Query<'q, O> {
        match column_name {
            "description" => query.bind(self.description.clone()),
            "done" => query.bind(self.done),
            _ => query,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let url = std::env::var("DATABASE_URL").expect("missing DATABASE_URL env");
    let db = sqlx::PgPool::connect(&url).await?;
    let store = miniorm::CrudStore::new(&db);

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
    let res = store.delete(id).await?;
    assert_eq!(res.rows_affected(), 1);

    println!("Checking delete successful");
    assert!(matches!(
        store.read(id).await,
        Err(sqlx::Error::RowNotFound)
    ));

    Ok(())
}
