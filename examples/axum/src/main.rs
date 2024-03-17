use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get},
    Json, Router,
};
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
        .route("/todos/:id/done", get(mark_as_done))
        .route("/todos/:id", delete(delete_todo))
        .route("/todos/:id", get(read_todo))
        .route("/todos", get(list_todos))
        .with_state(todos);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening...");
    println!("- try: http://{}/todos", listener.local_addr().unwrap());
    println!("- try: http://{}/todos/2", listener.local_addr().unwrap());
    println!(
        "- try: http://{}/todos/3/done",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app).await?;

    Ok(())
}

async fn list_todos(State(todos): State<TodoStore>) -> Result<impl IntoResponse, StatusCode> {
    todos
        .list()
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn read_todo(
    Path(id): Path<i64>,
    State(todos): State<TodoStore>,
) -> Result<impl IntoResponse, StatusCode> {
    todos
        .read(id)
        .await
        .map(|e| Json(e.into_inner()))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn delete_todo(
    Path(id): Path<i64>,
    State(todos): State<TodoStore>,
) -> Result<impl IntoResponse, StatusCode> {
    todos
        .delete(id)
        .await
        .map(|_| Json(()))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn mark_as_done(
    Path(id): Path<i64>,
    State(todos): State<TodoStore>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut todo = todos
        .read(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    todo.inner.done = true;

    todos
        .update(todo)
        .await
        .map(|e| Json(e.into_inner()))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
