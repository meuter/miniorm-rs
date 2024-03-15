mod store;
mod transaction;

use dotenv::dotenv;
use iso_currency::Currency;
use miniorm::Store;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::{types::chrono::NaiveDate, PgPool};
use transaction::{Instrument, Operation, Transaction};

use crate::{
    store::TransactionStore,
    transaction::{Stock, Ticker},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv()?;

    let url = std::env::var("DATABASE_URL").expect("missing DATABASE_URL env");
    let pool = PgPool::connect(&url).await?;

    println!("Recreating table...");
    TransactionStore::recreate_table(&pool).await?;

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
    let id = TransactionStore::create(&pool, &tx).await?;

    println!("Retrieveing by id...");
    let fetched = TransactionStore::read(&pool, id).await?;
    assert_eq!(tx, fetched);

    println!("Listing all...");
    let all = TransactionStore::list(&pool).await?;
    assert_eq!(all.len(), 1);
    assert_eq!(&tx, &all[0]);

    println!("Deleting by id...");
    let deleted = TransactionStore::delete(&pool, id).await?;
    assert_eq!(deleted, 1);

    println!("Checking delete successful");
    assert!(matches!(
        TransactionStore::read(&pool, id).await,
        Err(sqlx::Error::RowNotFound)
    ));

    Ok(())
}
