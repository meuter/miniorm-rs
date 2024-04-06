use axum::Router;
use miniorm::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[derive(Debug, Clone, Eq, PartialEq, FromRow, Entity, Serialize, Deserialize)]
struct Todo {
    #[postgres(TEXT NOT NULL)]
    description: String,

    #[postgres(BOOLEAN NOT NULL DEFAULT false)]
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
    // connect to postgres
    dotenv::dotenv().ok();
    let url = std::env::var("POSTGRES_URL").expect("missing POSTGRES_URL env");
    let db = PgPool::connect(&url).await?;

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
