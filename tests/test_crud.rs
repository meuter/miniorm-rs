mod common;

#[macro_export]
macro_rules! test_todo_crud {
    ($new_store: expr) => {
        #[cfg_attr(not(feature = "integration_tests"), ignore)]
        #[serial]
        #[tokio::test]
        async fn create() {
            #[allow(unused_mut)]
            let mut store = $new_store;
            let todo = store.create(Todo::new("checkout miniorm")).await.unwrap();
            assert_eq!(todo.id(), 1);
            assert_eq!(todo.description(), "checkout miniorm");
            assert!(!todo.is_done());
        }

        #[cfg_attr(not(feature = "integration_tests"), ignore)]
        #[serial]
        #[tokio::test]
        async fn read() {
            #[allow(unused_mut)]
            let mut store = $new_store;
            let todo = store.create(Todo::new("checkout miniorm")).await.unwrap();
            assert_eq!(todo, store.read(todo.id()).await.unwrap());
        }

        #[cfg_attr(not(feature = "integration_tests"), ignore)]
        #[serial]
        #[tokio::test]
        async fn list() {
            #[allow(unused_mut)]
            let mut store = $new_store;
            let todo1 = store.create(Todo::new("todo1")).await.unwrap();
            let todo2 = store.create(Todo::new("todo2")).await.unwrap();
            let todo3 = store.create(Todo::new("todo3")).await.unwrap();

            let all_todos = store.list().await.unwrap();
            assert_eq!(all_todos, [todo1, todo2, todo3]);
        }

        #[cfg_attr(not(feature = "integration_tests"), ignore)]
        #[serial]
        #[tokio::test]
        async fn update() {
            #[allow(unused_mut)]
            let mut store = $new_store;
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
            #[allow(unused_mut)]
            let mut store = $new_store;
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
            #[allow(unused_mut)]
            let mut store = $new_store;
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

    mod test_mysql {
        use crate::common::Todo;
        use miniorm::prelude::*;
        use serial_test::serial;
        use sqlx::{MySql, MySqlPool};
        use std::error::Error;

        pub async fn get_store() -> Result<Store<MySql, Todo>, Box<dyn Error>> {
            dotenv::dotenv()?;
            let url = std::env::var("MYSQL_URL").expect("missing MYSQL_URL env");
            let db = MySqlPool::connect(&url).await?;
            let store = miniorm::Store::new(db);
            store.recreate_table().await?;
            Ok(store)
        }

        test_todo_crud!(get_store().await.unwrap());
    }

    mod test_pgstore {
        use crate::common::Todo;
        use miniorm::prelude::*;
        use serial_test::serial;
        use sqlx::{PgPool, Postgres};
        use std::error::Error;

        pub async fn get_store() -> Result<Store<Postgres, Todo>, Box<dyn Error>> {
            dotenv::dotenv()?;
            let url = std::env::var("POSTGRES_URL").expect("missing POSTGRES_URL env");
            let db = PgPool::connect(&url).await?;
            let store = miniorm::Store::new(db);
            store.recreate_table().await?;
            Ok(store)
        }

        test_todo_crud!(get_store().await.unwrap());
    }

    mod test_sqlitestore {
        use crate::common::Todo;
        use miniorm::prelude::*;
        use serial_test::serial;
        use sqlx::{Sqlite, SqlitePool};
        use std::error::Error;

        async fn get_store() -> Result<Store<Sqlite, Todo>, Box<dyn Error>> {
            let url = ":memory:";
            let connection = SqlitePool::connect(url).await?;
            let store = Store::new(connection);
            store.recreate_table().await?;
            Ok(store)
        }

        test_todo_crud!(get_store().await.unwrap());
    }
}
