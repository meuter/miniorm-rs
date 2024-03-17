use std::net::SocketAddr;

// mod router {
//     use axum::{
//         routing::{delete, get},
//         Router,
//     };
//     use miniorm::{traits::Schema, CrudStore};
//     use serde::Serialize;
//     use sqlx::{postgres::PgRow, FromRow};
//
//     pub fn create_rest_router<E, S>(store: CrudStore<E>) -> Router<S>
//     where
//         E: Schema + for<'r> FromRow<'r, PgRow> + Unpin + Send + Serialize + Clone + Sync + 'static,
//     {
//         Router::new()
//             .route("/", get(handler::list))
//             .route("/", delete(handler::delete_all))
//             .route("/:id", delete(handler::delete))
//             .route("/:id", get(handler::read))
//             .with_state(store)
//     }
//
//     mod handler {
//
//         use axum::{
//             extract::{Path, State},
//             http::StatusCode,
//             response::IntoResponse,
//             Json,
//         };
//         use miniorm::{traits::Schema, CrudStore};
//         use serde::Serialize;
//         use sqlx::{postgres::PgRow, FromRow};
//
//         pub(crate) async fn list<E>(
//             State(store): State<CrudStore<E>>,
//         ) -> Result<impl IntoResponse, StatusCode>
//         where
//             E: Schema + for<'r> FromRow<'r, PgRow> + Unpin + Send + Serialize,
//         {
//             store
//                 .list()
//                 .await
//                 .map(Json)
//                 .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
//         }
//
//         pub(crate) async fn read<E>(
//             Path(id): Path<i64>,
//             State(todos): State<CrudStore<E>>,
//         ) -> Result<impl IntoResponse, StatusCode>
//         where
//             E: Schema + for<'r> FromRow<'r, PgRow> + Unpin + Send + Serialize,
//         {
//             todos
//                 .read(id)
//                 .await
//                 .map(|e| Json(e.into_inner()))
//                 .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
//         }
//
//         pub(crate) async fn delete<E>(
//             Path(id): Path<i64>,
//             State(todos): State<CrudStore<E>>,
//         ) -> Result<impl IntoResponse, StatusCode>
//         where
//             E: Schema + for<'r> FromRow<'r, PgRow> + Unpin + Send + Serialize,
//         {
//             todos
//                 .delete(id)
//                 .await
//                 .map(|_| Json(()))
//                 .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
//         }
//
//         pub(crate) async fn delete_all<E>(
//             State(todos): State<CrudStore<E>>,
//         ) -> Result<impl IntoResponse, StatusCode>
//         where
//             E: Schema + for<'r> FromRow<'r, PgRow> + Unpin + Send + Serialize,
//         {
//             todos
//                 .delete_all()
//                 .await
//                 .map(|_| Json(()))
//                 .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
//         }
//     }
// }

use axum::Router;
use miniorm::{CrudStore, Schema};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use tokio::net::TcpListener;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // connect to postgres
    dotenv::dotenv().ok();
    let url = std::env::var("DATABASE_URL").expect("missing DATABASE_URL env");
    let db = PgPool::connect(&url).await?;

    // initialize todo store
    let todos = CrudStore::new(db);
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
