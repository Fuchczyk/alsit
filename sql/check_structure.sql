CREATE SCHEMA user_data IF NOT EXISTS;
CREATE TABLE IF NOT EXISTS user_data.users (
    username VARCHAR(40) UNIQUE NOT NULL,
    password_hash VARCHAR,
    user_salt VARCHAR,
    email VARCHAR NOT NULL
);