use dotenv::dotenv;
use iso_currency::Currency;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::NaiveDate;

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

#[derive(Clone, Debug, Eq, PartialEq, sqlx::FromRow, miniorm::Schema)]
pub struct Transaction {
    #[column(DATE NOT NULL)]
    pub date: NaiveDate,

    #[sqlx(json)]
    #[column(JSONB NOT NULL)]
    pub operation: Operation,

    #[sqlx(json)]
    #[column(JSONB NOT NULL)]
    pub instrument: Instrument,

    #[column(DECIMAL NOT NULL)]
    pub quantity: Decimal,

    #[column(DECIMAL NOT NULL)]
    pub unit_price: Decimal,

    #[column(DECIMAL NOT NULL)]
    pub taxes: Decimal,

    #[column(DECIMAL NOT NULL)]
    pub fees: Decimal,

    #[sqlx(json)]
    #[column(JSONB NOT NULL)]
    pub currency: Currency,

    #[column(DECIMAL NOT NULL)]
    pub exchange_rate: Decimal,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv()?;

    let url = std::env::var("DATABASE_URL").expect("missing DATABASE_URL env");
    let db = sqlx::PgPool::connect(&url).await?;
    let store = miniorm::CrudStore::new(&db);

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
    let id = store.create(&tx).await?;

    println!("Retrieveing by id...");
    let fetched = store.read(id).await?;
    assert_eq!(tx, fetched);

    println!("Listing all...");
    let all = store.list().await?;
    assert_eq!(all.len(), 1);
    assert_eq!(&tx, &all[0]);

    println!("Deleting by id...");
    let res = store.delete(id).await?;
    assert_eq!(res.rows_affected(), 1);

    println!("Checking delete successful");
    assert!(matches!(
        store.read(id).await,
        Err(sqlx::Error::RowNotFound)
    ));

    Ok(())
}
