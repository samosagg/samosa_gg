-- Your SQL goes here
CREATE TABLE users(
    id UUID PRIMARY KEY NOT NULL,
    tg_id BIGINT,
    tg_username VARCHAR,
    connected_wallet VARCHAR(66),
    address VARCHAR(66) NOT NULL,
    public_key VARCHAR NOT NULL,
    wallet_id VARCHAR NOT NULL,
    slippage BIGINT NOT NULL DEFAULT(20),
    degen_mode BOOLEAN NOT NULL DEFAULT(FALSE)
);

CREATE TABLE subaccounts(
    id UUID PRIMARY KEY NOT NULL,
    user_id UUID NOT NULL,
    address VARCHAR(66) NOT NULL,
    is_primary BOOLEAN DEFAULT(FALSE)
);

CREATE UNIQUE INDEX unique_tg_id ON users (tg_id)
WHERE tg_id IS NOT NULL;

CREATE UNIQUE INDEX unique_connected_wallet ON users (connected_wallet)
WHERE connected_wallet IS NOT NULL;
