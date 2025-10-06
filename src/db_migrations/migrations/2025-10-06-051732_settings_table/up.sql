-- Your SQL goes here
CREATE TABLE settings(
    id UUID PRIMARY KEY NOT NULL,
    user_id UUID NOT NULL,
    degen_mode BOOLEAN DEFAULT false NOT NULL,
    notifications BOOLEAN DEFAULT true NOT NULL
);