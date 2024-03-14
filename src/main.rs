use dotenv::dotenv;
use iso_currency::Currency;
use rust_decimal::Decimal;
use sqlx::{types::chrono::NaiveDate, PgPool, Row};

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

    let res = sqlx::query("SELECT 1 + 1 AS SUM").fetch_one(&pool).await?;
    let sum: i32 = res.get("sum");

    println!("Hello, world! {sum}");
    Ok(())
}
