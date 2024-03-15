use crate::{
    miniorm::{Bind, ColunmName, Db, PgQueryAs, Store, Table},
    model::Transaction,
};
use async_trait::async_trait;
use sqlx::{postgres::PgRow, FromRow, Row};

pub struct TransactionStore;

#[async_trait]
impl Store<Transaction> for TransactionStore {
    const TABLE: Table = Table(
        "transaction",
        &[
            ("date", "DATE NOT NULL"),
            ("operation", "JSONB NOT NULL"),
            ("instrument", "JSONB NOT NULL"),
            ("quantity", "DECIMAL NOT NULL"),
            ("unit_price", "DECIMAL NOT NULL"),
            ("taxes", "DECIMAL NOT NULL"),
            ("fees", "DECIMAL NOT NULL"),
            ("currency", "JSONB NOT NULL"),
            ("exchange_rate", "DECIMAL NOT NULL"),
        ],
    );

    async fn update(_db: &Db, _id: i64, _entity: Transaction) -> sqlx::Result<i64> {
        todo!()
    }
}

impl FromRow<'_, PgRow> for Transaction {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        Ok(Transaction {
            date: row.try_get("date")?,
            operation: serde_json::from_value(row.try_get("operation")?).unwrap(),
            instrument: serde_json::from_value(row.try_get("instrument")?).unwrap(),
            quantity: row.try_get("quantity")?,
            unit_price: row.try_get("unit_price")?,
            taxes: row.try_get("taxes")?,
            fees: row.try_get("fees")?,
            currency: serde_json::from_value(row.try_get("currency")?).unwrap(),
            exchange_rate: row.try_get("exchange_rate")?,
        })
    }
}

impl Bind for Transaction {
    fn bind<'q, O>(&self, query: PgQueryAs<'q, O>, column_name: ColunmName) -> PgQueryAs<'q, O> {
        match column_name {
            "date" => query.bind(self.date),
            "operation" => query.bind(serde_json::to_value(self.operation).unwrap()),
            "instrument" => query.bind(serde_json::to_value(&self.instrument).unwrap()),
            "quantity" => query.bind(self.quantity),
            "unit_price" => query.bind(self.unit_price),
            "taxes" => query.bind(self.taxes),
            "fees" => query.bind(self.fees),
            "currency" => query.bind(serde_json::to_value(self.currency).unwrap()),
            "exchange_rate" => query.bind(self.exchange_rate),
            _ => query,
        }
    }
}
