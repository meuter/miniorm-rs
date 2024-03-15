use iso_currency::Currency;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::chrono::NaiveDate};

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

#[derive(Clone, Debug, Eq, PartialEq, FromRow)]
pub struct Transaction {
    pub date: NaiveDate,
    #[sqlx(json)]
    pub operation: Operation,
    #[sqlx(json)]
    pub instrument: Instrument,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub taxes: Decimal,
    pub fees: Decimal,
    #[sqlx(json)]
    pub currency: Currency,
    pub exchange_rate: Decimal,
}
