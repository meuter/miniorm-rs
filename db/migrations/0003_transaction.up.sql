-- Create the transaction table
CREATE TABLE IF NOT EXISTS transaction (
    id              BIGSERIAL PRIMARY KEY,
    date            DATE NOT NULL,
    operation       operation NOT NULL,
--     instrument      VARCHAR(50) NOT NULL,
    quantity        INTEGER NOT NULL,
    unit_price      MONEY NOT NULL,
    taxes           MONEY NOT NULL,
    fees            MONEY NOT NULL,
    currency        VARCHAR(3) NOT NULL
--     exchange_rate   REAL NOT NULL
);

