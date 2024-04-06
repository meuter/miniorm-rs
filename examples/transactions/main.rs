use iso_currency::Currency;
use miniorm::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use sqlx::{types::chrono::NaiveDate, FromRow};

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Ticker(pub String);

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Stock {
    pub ticker: Ticker,
    pub currency: Currency,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Instrument {
    Cash(Currency),
    Stock(Stock),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Operation {
    Buy,
    Sell,
    Dividend,
    Deposit,
    Withdrawal,
}

#[derive(Clone, Debug, Eq, PartialEq, FromRow, Entity)]
pub struct Transaction {
    #[postgres(DATE NOT NULL)]
    pub date: NaiveDate,

    #[sqlx(json)]
    #[postgres(JSONB NOT NULL)]
    pub operation: Operation,

    #[sqlx(json)]
    #[postgres(JSONB NOT NULL)]
    pub instrument: Instrument,

    #[postgres(DECIMAL NOT NULL)]
    pub quantity: Decimal,

    #[postgres(DECIMAL NOT NULL)]
    pub unit_price: Decimal,

    #[postgres(DECIMAL NOT NULL)]
    pub taxes: Decimal,

    #[postgres(DECIMAL NOT NULL)]
    pub fees: Decimal,

    #[sqlx(json)]
    #[postgres(JSONB NOT NULL)]
    pub currency: Currency,

    #[postgres(DECIMAL NOT NULL)]
    pub exchange_rate: Decimal,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    println!("== POSTGRES ==");
    let url = std::env::var("POSTGRES_URL").expect("POSTGRES_URL env variable not set");
    let db = sqlx::PgPool::connect(&url).await?;
    let store = miniorm::Store::new(db);

    println!("Recreating table...");
    store.recreate_table().await?;

    let aapl = Stock {
        ticker: Ticker("AAPL".into()),
        currency: Currency::USD,
    };

    let tx = Transaction {
        date: NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
        operation: Operation::Buy,
        instrument: Instrument::Stock(aapl),
        quantity: dec!(10),
        unit_price: dec!(170.0),
        taxes: dec!(10.2),
        fees: dec!(5.5),
        currency: Currency::USD,
        exchange_rate: dec!(0.9),
    };

    println!("Inserting...");
    let tx = store.create(tx).await?;

    println!("Retrieveing by id...");
    let mut fetched = store.read(tx.id()).await?;
    assert_eq!(tx, fetched);

    println!("Updating by id");
    fetched.operation = Operation::Sell;
    let after_update = store.update(fetched).await?;

    println!("Listing all...");
    let all = store.list().await?;
    assert_eq!(all.len(), 1);
    assert_eq!(&after_update, &all[0]);

    println!("Deleting by id...");
    store.delete(tx.id()).await?;

    println!("Checking delete successful");
    assert!(matches!(
        store.read(tx.id()).await,
        Err(sqlx::Error::RowNotFound)
    ));

    Ok(())
}
