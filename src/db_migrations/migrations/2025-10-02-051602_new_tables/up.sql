-- Your SQL goes here
CREATE TABLE users(
    id UUID PRIMARY KEY NOT NULL,
    telegram_id BIGINT,
    telegram_username VARCHAR,
    secondary_wallet_address VARCHAR(66),
    degen_mode BOOLEAN DEFAULT FALSE NOT NULL
);

CREATE TABLE wallets(
    id UUID PRIMARY KEY NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    wallet_id VARCHAR NOT NULL,
    address VARCHAR(66) NOT NULL,
    public_key VARCHAR NOT NULL,
    is_primary BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE subaccounts(
    id UUID PRIMARY KEY NOT NULL,
    wallet_id UUID NOT NULL REFERENCES wallets(id) ON DELETE CASCADE,
    address VARCHAR(66) NOT NULL,
    is_primary BOOLEAN DEFAULT FALSE
);

CREATE UNIQUE INDEX unique_telegram_id ON users (telegram_id)
WHERE telegram_id IS NOT NULL;

CREATE UNIQUE INDEX unique_secondary_wallet_address ON users (secondary_wallet_address)
WHERE secondary_wallet_address IS NOT NULL;