DROP SCHEMA IF EXISTS user_data CASCADE;
CREATE SCHEMA user_data;
CREATE TABLE user_data.users (
    username VARCHAR(40) UNIQUE NOT NULL,
    password_hash BYTEA,
    user_salt BYTEA,
    email BYTEA
);