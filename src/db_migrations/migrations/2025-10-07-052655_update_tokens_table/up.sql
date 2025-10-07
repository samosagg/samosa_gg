-- Your SQL goes here
ALTER TABLE tokens
ADD COLUMN decimals INT NOT NULL DEFAULT 6;