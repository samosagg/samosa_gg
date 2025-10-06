-- Your SQL goes here
ALTER TABLE users
ADD COLUMN slippage INT DEFAULT(3) NOT NULL;