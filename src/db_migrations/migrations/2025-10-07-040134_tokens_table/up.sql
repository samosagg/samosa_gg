-- Your SQL goes here
CREATE TABLE tokens(
    id UUID PRIMARY KEY NOT NULL,
    name VARCHAR(30) NOT NULL,
    symbol VARCHAR(30) NOT NULL,
    address VARCHAR(66) NOT NULL,
    coin_addr VARCHAR
)