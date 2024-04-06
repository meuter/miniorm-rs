use crate::{traits::crud::Crud, WithId};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

pub(crate) struct Handler<E, S> {
    entity: PhantomData<E>,
    store: S,
}

impl<E, S> Handler<E, S> {
    pub(crate) fn new(store: S) -> Self {
        let entity = PhantomData;
        Handler { entity, store }
    }
}

impl<E, S> Handler<E, S>
where
    S: Crud<E> + Sync + Send + Clone + 'static,
    E: Send + 'static,
    E: Serialize + for<'de> Deserialize<'de>,
{
    fn to_status_code(err: sqlx::Error) -> StatusCode {
        match err {
            sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub(crate) async fn create(
        State(store): State<S>,
        Json(payload): Json<E>,
    ) -> Result<impl IntoResponse, StatusCode> {
        store
            .create(payload)
            .await
            .map_err(Self::to_status_code)
            .map(Json)
    }

    pub(crate) async fn read(
        Path(id): Path<i64>,
        State(store): State<S>,
    ) -> Result<impl IntoResponse, StatusCode> {
        store.read(id).await.map_err(Self::to_status_code).map(Json)
    }

    pub(crate) async fn list(State(store): State<S>) -> Result<impl IntoResponse, StatusCode> {
        store.list().await.map_err(Self::to_status_code).map(Json)
    }

    pub(crate) async fn update(
        Path(id): Path<i64>,
        State(store): State<S>,
        Json(payload): Json<E>,
    ) -> Result<impl IntoResponse, StatusCode> {
        let payload = WithId::new(payload, id);
        store
            .update(payload)
            .await
            .map_err(Self::to_status_code)
            .map(Json)
    }

    pub(crate) async fn update_with_id(
        State(store): State<S>,
        Json(payload): Json<WithId<E>>,
    ) -> Result<impl IntoResponse, StatusCode> {
        store
            .update(payload)
            .await
            .map_err(Self::to_status_code)
            .map(Json)
    }

    pub(crate) async fn delete(
        Path(id): Path<i64>,
        State(store): State<S>,
    ) -> Result<impl IntoResponse, StatusCode> {
        store.delete(id).await.map_err(Self::to_status_code)
    }

    pub(crate) async fn delete_all(
        State(store): State<S>,
    ) -> Result<impl IntoResponse, StatusCode> {
        store
            .delete_all()
            .await
            .map_err(Self::to_status_code)
            .map(|_| ())
    }

    pub(crate) fn into_axum_router<R>(self) -> Router<R> {
        Router::new()
            .route("/", get(Self::list))
            .route("/", post(Self::create))
            .route("/", delete(Self::delete_all))
            .route("/", put(Self::update_with_id))
            .route("/:id", delete(Self::delete))
            .route("/:id", get(Self::read))
            .route("/:id", put(Self::update))
            .with_state(self.store)
    }
}
