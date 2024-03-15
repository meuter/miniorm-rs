use std::str::FromStr;

use crate::{
    miniorm::{Columns, ColunmName, Db, Table, TableName},
    model::{Operation, Transaction},
};
use async_trait::async_trait;
use iso_currency::Currency;
use sqlx::{database::HasArguments, postgres::PgRow, query::QueryAs, FromRow, Postgres, Row};

pub type PgQueryAs<'q, O> = QueryAs<'q, Postgres, O, <Postgres as HasArguments<'q>>::Arguments>;

pub trait Bind {
    fn bind_column<'q, O>(
        &self,
        query: PgQueryAs<'q, O>,
        column_name: ColunmName,
    ) -> PgQueryAs<'q, O>;
}

impl FromRow<'_, PgRow> for Transaction {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        Ok(Transaction {
            date: row.try_get("date")?,
            operation: Operation::from_str(row.try_get("operation")?).unwrap(),
            instrument: serde_json::from_str(row.try_get("instrument")?).unwrap(),
            quantity: row.try_get("quantity")?,
            unit_price: row.try_get("unit_price")?,
            taxes: row.try_get("taxes")?,
            fees: row.try_get("fees")?,
            currency: Currency::from_code(row.try_get("currency")?).unwrap(),
            exchange_rate: row.try_get("exchange_rate")?,
        })
    }
}

impl Bind for Transaction {
    fn bind_column<'q, O>(
        &self,
        query: PgQueryAs<'q, O>,
        column_name: ColunmName,
    ) -> PgQueryAs<'q, O> {
        match column_name {
            "date" => query.bind(self.date),
            "unit_price" => query.bind(self.unit_price),
            _ => query,
        }
    }
}

pub struct TransactionTable;

#[async_trait]
impl Table<Transaction> for TransactionTable {
    const TABLE_NAME: TableName = "transaction";
    const COLUMNS: Columns = &[
        ("id", "BIGSERIAL PRIMARY KEY"),
        ("date", "DATE NOT NULL"),
        ("operation", "VARCHAR(10) NOT NULL"),
        ("instrument", "VARCHAR(50) NOT NULL"),
        ("quantity", "DECIMAL NOT NULL"),
        ("unit_price", "DECIMAL NOT NULL"),
        ("taxes", "DECIMAL NOT NULL"),
        ("fees", "DECIMAL NOT NULL"),
        ("currency", "VARCHAR(3) NOT NULL"),
        ("exchange_rate", "DECIMAL NOT NULL"),
    ];

    async fn add(db: &Db, tx: &Transaction) -> sqlx::Result<i64> {
        let query_as = sqlx::query_as(
            r#"
                INSERT INTO transaction (
                    date,
                    operation,
                    instrument,
                    quantity,
                    unit_price,
                    taxes,
                    fees,
                    currency,
                    exchange_rate
                )
                VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
                RETURNING id
            "#,
        );

        let (id,) = tx
            .bind_column(query_as, "date")
            .bind(format!("{}", tx.operation))
            .bind(serde_json::to_string(&tx.instrument).unwrap())
            .bind(tx.quantity)
            .bind(tx.unit_price)
            .bind(tx.taxes)
            .bind(tx.fees)
            .bind(tx.currency.code())
            .bind(tx.exchange_rate)
            .fetch_one(db)
            .await?;
        Ok(id)
    }

    async fn update(_db: &Db, _id: i64, _entity: Transaction) -> sqlx::Result<i64> {
        todo!()
    }
}
