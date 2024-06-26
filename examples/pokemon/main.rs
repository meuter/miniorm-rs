use miniorm::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::Type, FromRow, MySql};
use std::string::ToString;

#[derive(Debug, Clone, Eq, PartialEq, Type, Serialize, Deserialize)]
pub enum PokemonType {
    Unknown,
    Fire,
    Water,
    Electric,
    Ice,
    Poison,
    Rock,
}

#[derive(Debug, Clone, Eq, PartialEq, FromRow, Entity)]
struct Pokemon {
    #[mysql(TEXT NOT NULL)]
    name: String,

    #[mysql(VARCHAR(40) NOT NULL)]
    #[sqlx(json)]
    ty: PokemonType,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    println!("== MYSQL ==");
    let url = std::env::var("MYSQL_URL").expect("MYSQL_URL env variable not set");
    let db = sqlx::MySqlPool::connect(&url).await?;
    let store: Store<MySql, Pokemon> = miniorm::Store::new(db);

    let pikatchu = Pokemon {
        name: "Pikatchu".to_string(),
        ty: PokemonType::Electric,
    };

    println!("Recreating table...");
    store.recreate_table().await?;

    println!("Inserting...");
    let pikatchu = store.create(pikatchu).await?;

    println!("Retrieveing by id...");
    let mut fetched = store.read(pikatchu.id()).await?;
    assert_eq!(pikatchu, fetched);

    println!("Updating by id...");
    fetched.name = "Pikaaaaaatchuuuuuuu!".to_string();
    let after_update = store.update(fetched).await?;
    assert_eq!(after_update.id(), pikatchu.id());

    println!("Listing all...");
    let all = store.list().await?;
    assert_eq!(all.len(), 1);
    assert_eq!(&after_update, &all[0]);

    println!("Deleting by id...");
    store.delete(pikatchu.id()).await?;

    println!("Checking delete successful");
    assert!(matches!(
        store.read(pikatchu.id()).await,
        Err(sqlx::Error::RowNotFound)
    ));

    Ok(())
}
