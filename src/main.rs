use async_trait::async_trait;
use dotenv::dotenv;
use iso_currency::Currency;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use sqlx::{
    prelude::FromRow,
    types::chrono::{NaiveDate, NaiveTime},
    PgPool, Pool, Postgres,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clone, Debug)]
pub struct Ticker(pub String);

#[derive(Clone, Debug)]
pub struct Stock {
    pub ticker: Ticker,
    pub currency: Currency,
}

#[derive(Clone, Debug)]
pub enum Instrument {
    Cash(Currency),
    Stock(Stock),
}

#[derive(Copy, Clone, Debug, sqlx::Type, Eq, PartialEq)]
#[sqlx(type_name = "operation", rename_all = "lowercase")]
pub enum Operation {
    Buy,
    Sell,
    Dividend,
    Deposit,
    Withdrawal,
}

#[derive(Clone, Debug, FromRow, Eq, PartialEq)]
pub struct Transaction {
    pub date: NaiveDate,
    pub operation: Operation,
    // TODO
    // pub instrment: Instrument,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub taxes: Decimal,
    pub fees: Decimal,
    // pub currency: Currency,
    pub exchange_rate: Decimal,
}

pub struct TransactionRepository;

pub type Db = Pool<Postgres>;

#[async_trait]
pub trait Repository<E> {
    const TABLE: &'static str;

    async fn recreate(db: &Db) -> sqlx::Result<()> {
        Self::drop(db).await?;
        Self::create(db).await?;
        Ok(())
    }
    async fn create(db: &Db) -> sqlx::Result<()>;
    async fn drop(db: &Db) -> sqlx::Result<()>;

    async fn add(db: &Db, entity: &E) -> sqlx::Result<i64>;
    async fn get(db: &Db, id: i64) -> sqlx::Result<E>;
    async fn list(db: &Db) -> sqlx::Result<Vec<E>>;
    async fn update(db: &Db, id: i64, entity: E) -> sqlx::Result<i64>;
    async fn delete(db: &Db, id: i64) -> sqlx::Result<i64>;
}

#[async_trait]
impl Repository<Transaction> for TransactionRepository {
    const TABLE: &'static str = "transaction";

    async fn create(db: &Db) -> sqlx::Result<()> {
        sqlx::query(
            r#"
            CREATE TYPE operation AS ENUM (
                'buy',
                'sell',
                'dividend',
                'deposit',
                'withdrawal'
            )
        "#,
        )
        .execute(db)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS transaction (
                id              BIGSERIAL PRIMARY KEY,
                date            DATE NOT NULL,
                operation       operation NOT NULL,
                quantity        DECIMAL NOT NULL,
                unit_price      DECIMAL NOT NULL,
                taxes           DECIMAL NOT NULL,
                fees            DECIMAL NOT NULL,
                exchange_rate   DECIMAL NOT NULL
            )
        "#,
        )
        .execute(db)
        .await?;
        Ok(())
    }

    async fn drop(db: &Db) -> sqlx::Result<()> {
        sqlx::query("DROP TABLE IF EXISTS transaction")
            .execute(db)
            .await?;
        sqlx::query("DROP TYPE IF EXISTS operation")
            .execute(db)
            .await?;
        Ok(())
    }

    async fn add(db: &Db, tx: &Transaction) -> sqlx::Result<i64> {
        let (id,) = sqlx::query_as(
            r#"
                INSERT INTO transaction (
                    date,
                    operation,
                    quantity,
                    unit_price,
                    taxes,
                    fees,
                    exchange_rate
                )
                VALUES ($1,$2,$3,$4,$5,$6,$7)
                RETURNING id
            "#,
        )
        .bind(tx.date)
        .bind(tx.operation)
        .bind(tx.quantity)
        .bind(tx.unit_price)
        .bind(tx.taxes)
        .bind(tx.fees)
        .bind(tx.exchange_rate)
        .fetch_one(db)
        .await?;
        Ok(id)
    }

    async fn get(db: &Db, id: i64) -> sqlx::Result<Transaction> {
        let transaction: Transaction = sqlx::query_as(
            r#"
                SELECT
                    date,
                    operation,
                    quantity,
                    unit_price,
                    taxes,
                    fees,
                    exchange_rate
                FROM
                    transaction
                WHERE id=$1"#,
        )
        .bind(id)
        .fetch_one(db)
        .await?;
        Ok(transaction)
    }

    async fn list(db: &Db) -> sqlx::Result<Vec<Transaction>> {
        todo!()
    }
    async fn update(db: &Db, id: i64, entity: Transaction) -> sqlx::Result<i64> {
        todo!()
    }
    async fn delete(db: &Db, id: i64) -> sqlx::Result<i64> {
        todo!()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;

    let url = std::env::var("DATABASE_URL").expect("missing DATABASE_URL env");
    let pool = PgPool::connect(&url).await?;

    TransactionRepository::recreate(&pool).await?;

    let aapl = Stock {
        ticker: Ticker("AAPL".into()),
        currency: Currency::USD,
    };

    let tx = Transaction {
        date: NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
        operation: Operation::Buy,
        // instrment: Instrument::Stock(aapl),
        quantity: dec!(10),
        unit_price: dec!(170.0),
        taxes: dec!(10.2),
        fees: dec!(5.5),
        // currency: Currency::USD,
        exchange_rate: dec!(0.9),
    };

    let id = TransactionRepository::add(&pool, &tx).await?;
    let fetched = TransactionRepository::get(&pool, id).await?;
    assert_eq!(tx, fetched);
    Ok(())
}
