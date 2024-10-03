mod common;

use common::Todo;
use miniorm::prelude::*;
use serial_test::serial;
use std::error::Error;

#[macro_export]
macro_rules! test_crud {
    ($db: block) => {
        async fn get_clean_store() -> Result<impl Crud<Todo, i64>, Box<dyn Error>> {
            let pool = $db;
            let store = Store::new(pool);
            store.recreate_table().await?;
            Ok(store)
        }

        #[cfg_attr(not(feature = "integration_tests"), ignore)]
        #[serial]
        #[tokio::test]
        async fn create() {
            let store = get_clean_store().await.unwrap();
            let todo = store.create(Todo::new("checkout miniorm")).await.unwrap();
            assert_eq!(todo.id(), 1);
            assert_eq!(todo.description(), "checkout miniorm");
            assert!(!todo.is_done());
        }

        #[cfg_attr(not(feature = "integration_tests"), ignore)]
        #[serial]
        #[tokio::test]
        async fn read() {
            let store = get_clean_store().await.unwrap();
            let todo = store.create(Todo::new("checkout miniorm")).await.unwrap();
            assert_eq!(todo, store.read(todo.id()).await.unwrap());
        }

        #[cfg_attr(not(feature = "integration_tests"), ignore)]
        #[serial]
        #[tokio::test]
        async fn list() {
            let store = get_clean_store().await.unwrap();
            let todo1 = store.create(Todo::new("todo1")).await.unwrap();
            let todo2 = store.create(Todo::new("todo2")).await.unwrap();
            let todo3 = store.create(Todo::new("todo3")).await.unwrap();

            let all_todos = store.list().await.unwrap();
            assert_eq!(all_todos, [todo1, todo2, todo3]);
        }

        #[cfg_attr(not(feature = "integration_tests"), ignore)]
        #[serial]
        #[tokio::test]
        async fn count() {
            let store = get_clean_store().await.unwrap();
            assert_eq!(store.count().await.unwrap(), 0);

            store.create(Todo::new("todo1")).await.unwrap();
            assert_eq!(store.count().await.unwrap(), 1);

            store.create(Todo::new("todo2")).await.unwrap();
            assert_eq!(store.count().await.unwrap(), 2);

            store.create(Todo::new("todo3")).await.unwrap();
            assert_eq!(store.count().await.unwrap(), 3);
        }

        #[cfg_attr(not(feature = "integration_tests"), ignore)]
        #[serial]
        #[tokio::test]
        async fn update() {
            let store = get_clean_store().await.unwrap();
            let mut todo = store.create(Todo::new("checkout miniorm")).await.unwrap();
            let id = todo.id();

            assert!(!store.read(id).await.unwrap().is_done());

            todo.mark_as_done();
            assert!(!store.read(id).await.unwrap().is_done());

            store.update(todo).await.unwrap();
            assert!(store.read(id).await.unwrap().is_done());
        }

        #[cfg_attr(not(feature = "integration_tests"), ignore)]
        #[serial]
        #[tokio::test]
        async fn delete() {
            let store = get_clean_store().await.unwrap();
            let todo = store.create(Todo::new("checkout miniorm")).await.unwrap();

            store.delete(todo.id()).await.unwrap();

            assert!(matches!(
                store.delete(todo.id()).await,
                Err(sqlx::Error::RowNotFound)
            ));

            assert!(matches!(
                store.read(todo.id()).await,
                Err(sqlx::Error::RowNotFound)
            ));
        }

        #[cfg_attr(not(feature = "integration_tests"), ignore)]
        #[serial]
        #[tokio::test]
        async fn delete_all() {
            let store = get_clean_store().await.unwrap();
            let todo1 = store.create(Todo::new("todo1")).await.unwrap();
            let todo2 = store.create(Todo::new("todo2")).await.unwrap();
            let todo3 = store.create(Todo::new("todo3")).await.unwrap();

            let all_todos = store.list().await.unwrap();
            assert_eq!(all_todos, [todo1, todo2, todo3]);

            let result = store.delete_all().await.unwrap();
            assert_eq!(result, 3);
            assert!(store.list().await.unwrap().is_empty());
        }
    };
}

mod test_crud {
    use super::*;

    #[cfg(feature = "mysql")]
    mod mysql {
        use super::*;
        use sqlx::MySqlPool;

        test_crud!({
            dotenv::dotenv()?;
            let url = std::env::var("MYSQL_URL").expect("missing MYSQL_URL env");
            MySqlPool::connect(&url).await?
        });
    }

    #[cfg(feature = "postgres")]
    mod postgres {
        use super::*;
        use sqlx::PgPool;

        test_crud!({
            dotenv::dotenv()?;
            let url = std::env::var("POSTGRES_URL").expect("missing POSTGRES_URL env");
            PgPool::connect(&url).await?
        });
    }

    #[cfg(feature = "sqlite")]
    mod sqlite {
        use super::*;
        use sqlx::SqlitePool;

        test_crud!({ SqlitePool::connect(":memory:").await? });
    }
}
