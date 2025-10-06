-- Your SQL goes here
CREATE TABLE users(
    id UUID PRIMARY KEY NOT NULL,
    wallet_id VARCHAR NOT NULL,
    wallet_address VARCHAR(66) NOT NULL,
    wallet_public_key VARCHAR NOT NULL,
    telegram_id BIGINT,
    telegram_username VARCHAR,
    secondary_wallet_address VARCHAR(66)
);

CREATE TABLE sub_accounts(
    id UUID PRIMARY KEY NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    address VARCHAR(66) NOT NULL,
    is_primary BOOLEAN DEFAULT FALSE
);

CREATE UNIQUE INDEX unique_telegram_id ON users (telegram_id)
WHERE telegram_id IS NOT NULL;

CREATE UNIQUE INDEX unique_secondary_wallet_address ON users (secondary_wallet_address)
WHERE secondary_wallet_address IS NOT NULL;