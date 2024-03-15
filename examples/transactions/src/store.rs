use miniorm::{HasTable, Table};

use crate::transaction::Transaction;

impl HasTable for Transaction {
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
}
