use dotenv::dotenv;
use iso_currency::Currency;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use sqlx::{types::chrono::NaiveDate, PgPool};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct Ticker(pub String);

pub struct Stock {
    pub ticker: Ticker,
    pub currency: Currency,
}

pub enum Instrument {
    Cash(Currency),
    Stock(Stock),
}

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, Deserialize, Serialize)]
#[sqlx(type_name = "operation", rename_all = "lowercase")]
pub enum Operation {
    Buy,
    Sell,
    Dividend,
    Deposit,
    Withdrawal,
}

pub struct Transaction {
    pub date: NaiveDate,
    pub operation: Operation,
    pub instrment: Instrument,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub taxes: Decimal,
    pub fees: Decimal,
    pub currency: Currency,
    pub exchange_rate: Decimal,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;

    let url = std::env::var("DATABASE_URL").expect("missing DATABASE_URL env");
    let pool = PgPool::connect(&url).await?;

    sqlx::migrate!("./db/migrations").run(&pool).await?;

    let aapl = Stock {
        ticker: Ticker("AAPL".into()),
        currency: Currency::USD,
    };

    let tx = Transaction {
        date: NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
        operation: Operation::Buy,
        instrment: Instrument::Stock(aapl),
        quantity: dec!(10.0),
        unit_price: dec!(170.0),
        taxes: dec!(10.2),
        fees: dec!(5.5),
        currency: Currency::USD,
        exchange_rate: dec!(0.9),
    };

    sqlx::query(
        r#"
        INSERT INTO transaction (
            date,
            operation,
            quantity,
            unit_price,
            taxes,
            fees,
            currency
        ) VALUES ($1,$2,$3,$4,$5,$6,$7)"#,
    )
    .bind(tx.date)
    .bind(tx.operation)
    .bind(tx.quantity)
    .bind(tx.unit_price)
    .bind(tx.taxes)
    .bind(tx.fees)
    .bind(tx.currency.code())
    .execute(&pool)
    .await?;

    Ok(())
}
