mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use common::Todo;
use miniorm::prelude::*;
use serial_test::serial;
use std::error::Error;

#[macro_export]
macro_rules! test_rest {
    ($db: block) => {
        async fn get_store_with_sample_data(
        ) -> Result<impl Clone + Crud<Todo> + IntoAxumRouter, Box<dyn Error>> {
            let pool = $db;
            let store = Store::new(pool);
            store.recreate_table().await?;
            store.create(Todo::new("do the laundry")).await.unwrap();
            store.create(Todo::new("wash the dishes")).await.unwrap();
            store.create(Todo::new("go walk the dog")).await.unwrap(); // <- id:3
            store.create(Todo::new("groceries")).await.unwrap();
            Ok(store)
        }

        #[cfg_attr(not(feature = "integration_tests"), ignore)]
        #[serial]
        #[tokio::test]
        async fn get_ok() {
            let store = get_store_with_sample_data().await.unwrap();
            let server = TestServer::new(store.clone().into_axum_router()).unwrap();
            let before = store.count().await.unwrap();
            let actual = server.get("/3").await.json::<WithId<Todo>>();
            let after = store.count().await.unwrap();
            let expected = store.read(3).await.unwrap();
            assert_eq!(before, after);
            assert_eq!(actual, expected);
        }

        #[cfg_attr(not(feature = "integration_tests"), ignore)]
        #[serial]
        #[tokio::test]
        async fn get_not_found() {
            let store = get_store_with_sample_data().await.unwrap();
            let server = TestServer::new(store.clone().into_axum_router()).unwrap();
            server.get("/23").await.assert_status(StatusCode::NOT_FOUND);
        }

        #[cfg_attr(not(feature = "integration_tests"), ignore)]
        #[serial]
        #[tokio::test]
        async fn list() {
            let store = get_store_with_sample_data().await.unwrap();
            let server = TestServer::new(store.clone().into_axum_router()).unwrap();
            let before = store.count().await.unwrap();
            let actual = server.get("/").await.json::<Vec<WithId<Todo>>>();
            let after = store.count().await.unwrap();
            let expected = store.list().await.unwrap();
            assert_eq!(before, after);
            assert_eq!(actual, expected);
        }

        #[cfg_attr(not(feature = "integration_tests"), ignore)]
        #[serial]
        #[tokio::test]
        async fn post() {
            let store = get_store_with_sample_data().await.unwrap();
            let server = TestServer::new(store.clone().into_axum_router()).unwrap();
            let before = store.count().await.unwrap();
            let actual = server
                .post("/")
                .json(&Todo::new("new one"))
                .await
                .json::<WithId<Todo>>();
            let after = store.count().await.unwrap();
            let expected = store.read(actual.id()).await.unwrap();
            assert_eq!(before + 1, after);
            assert_eq!(actual, expected);
        }

        #[cfg_attr(not(feature = "integration_tests"), ignore)]
        #[serial]
        #[tokio::test]
        async fn put() {
            let store = get_store_with_sample_data().await.unwrap();
            let server = TestServer::new(store.clone().into_axum_router()).unwrap();
            let before = store.count().await.unwrap();
            let mut expected = store.read(3).await.unwrap();
            assert!(!expected.is_done());
            expected.mark_as_done();
            let actual = server
                .put("/3")
                .json(expected.inner())
                .await
                .json::<WithId<Todo>>();
            let after = store.count().await.unwrap();
            let expected = store.read(actual.id()).await.unwrap();
            assert_eq!(before, after);
            assert_eq!(actual, expected);
        }

        #[cfg_attr(not(feature = "integration_tests"), ignore)]
        #[serial]
        #[tokio::test]
        async fn put_with_id() {
            let store = get_store_with_sample_data().await.unwrap();
            let server = TestServer::new(store.clone().into_axum_router()).unwrap();
            let before = store.count().await.unwrap();
            let mut expected = store.read(3).await.unwrap();
            assert!(!expected.is_done());
            expected.mark_as_done();
            let actual = server.put("/").json(&expected).await.json::<WithId<Todo>>();
            let after = store.count().await.unwrap();
            let expected = store.read(actual.id()).await.unwrap();
            assert_eq!(before, after);
            assert_eq!(actual, expected);
        }

        #[cfg_attr(not(feature = "integration_tests"), ignore)]
        #[serial]
        #[tokio::test]
        async fn delete_ok() {
            let store = get_store_with_sample_data().await.unwrap();
            let server = TestServer::new(store.clone().into_axum_router()).unwrap();
            let before = store.count().await.unwrap();
            server.delete("/3").await.assert_status_ok();
            let after = store.count().await.unwrap();
            assert!(matches!(store.read(3).await, Err(sqlx::Error::RowNotFound)));
            assert_eq!(before, after + 1);
        }

        #[cfg_attr(not(feature = "integration_tests"), ignore)]
        #[serial]
        #[tokio::test]
        async fn delete_not_found() {
            let store = get_store_with_sample_data().await.unwrap();
            let server = TestServer::new(store.clone().into_axum_router()).unwrap();
            server
                .delete("/23")
                .await
                .assert_status(StatusCode::NOT_FOUND);
        }

        #[cfg_attr(not(feature = "integration_tests"), ignore)]
        #[serial]
        #[tokio::test]
        async fn delete_all() {
            let store = get_store_with_sample_data().await.unwrap();
            let server = TestServer::new(store.clone().into_axum_router()).unwrap();
            let before = store.count().await.unwrap();
            server.delete("/").await.assert_status_ok();
            let after = store.count().await.unwrap();
            assert_eq!(before, 4);
            assert_eq!(after, 0);
            assert!(matches!(store.read(1).await, Err(sqlx::Error::RowNotFound)));
            assert!(matches!(store.read(2).await, Err(sqlx::Error::RowNotFound)));
            assert!(matches!(store.read(3).await, Err(sqlx::Error::RowNotFound)));
            assert!(matches!(store.read(4).await, Err(sqlx::Error::RowNotFound)));
        }
    };
}

mod test_rest {
    use super::*;

    #[cfg(feature = "mysql")]
    mod mysql {
        use super::*;
        use sqlx::MySqlPool;

        test_rest!({
            dotenv::dotenv()?;
            let url = std::env::var("MYSQL_URL").expect("missing MYSQL_URL env");
            MySqlPool::connect(&url).await?
        });
    }

    #[cfg(feature = "postgres")]
    mod postgres {
        use super::*;
        use sqlx::PgPool;

        test_rest!({
            dotenv::dotenv()?;
            let url = std::env::var("POSTGRES_URL").expect("missing POSTGRES_URL env");
            PgPool::connect(&url).await?
        });
    }

    #[cfg(feature = "sqlite")]
    mod sqlite {
        use super::*;
        use sqlx::SqlitePool;

        test_rest!({ SqlitePool::connect(":memory:").await? });
    }
}
