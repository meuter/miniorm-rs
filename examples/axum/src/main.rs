use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use miniorm::{CrudStore, Schema};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

type BoxResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone, Eq, PartialEq, FromRow, Schema, Serialize, Deserialize)]
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

type TodoStore = CrudStore<Todo>;

#[tokio::main]
async fn main() -> BoxResult<()> {
    // connect to postgres
    dotenv::dotenv().ok();
    let url = std::env::var("DATABASE_URL").expect("missing DATABASE_URL env");
    let db = PgPool::connect(&url).await?;

    // initialize todo store
    let todos = TodoStore::new(db);
    todos.recreate_table().await?;
    todos.create(Todo::new("do the laundry")).await?;
    todos.create(Todo::new("wash the dishes")).await?;
    todos.create(Todo::new("go walk the dog")).await?;
    todos.create(Todo::new("groceries")).await?;

    // create and start the app
    let app = Router::new()
        .route("/todos", get(list_todos))
        .with_state(todos);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await?;

    Ok(())
}

async fn list_todos(State(todos): State<TodoStore>) -> Result<impl IntoResponse, StatusCode> {
    if let Ok(all_todos) = todos.list().await {
        Ok(Json(all_todos))
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
