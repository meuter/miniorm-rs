-- Create the operation type
CREATE TYPE operation AS ENUM (
    'buy',
    'sell',
    'dividend',
    'deposit',
    'withdrawal'
);
