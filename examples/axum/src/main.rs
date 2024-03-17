use axum::{response::Html, routing::get, Router};
use miniorm::{CrudStore, Schema};
use sqlx::{FromRow, PgPool};
use tokio::net::TcpListener;

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
    let db = PgPool::connect(&url).await?;
    let store: CrudStore<'_, Todo> = CrudStore::new(&db);

    // build our application with a route
    let app = Router::new().route("/", get(handler));

    // run it
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}
