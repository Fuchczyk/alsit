DROP SCHEMA IF EXISTS user_data CASCADE;
DROP SCHEMA IF EXISTS ticket_data CASCADE;

CREATE SCHEMA user_data;
CREATE SCHEMA ticket_data;

GRANT ALL ON SCHEMA user_data TO alsit;
GRANT ALL ON SCHEMA ticket_data TO alsit;

GRANT ALL ON ALL TABLES IN SCHEMA user_data TO alsit;
GRANT ALL ON ALL TABLES IN SCHEMA ticket_data TO alsit;

CREATE TABLE user_data.users (
    id BIGINT UNIQUE NOT NULL PRIMARY KEY,
    username VARCHAR(40) UNIQUE NOT NULL,
    password_hash BYTEA NOT NULL,
    user_salt BYTEA NOT NULL,
    email BYTEA NOT NULL
);

CREATE TABLE ticket_data.tickets (
    id BIGINT UNIQUE NOT NULL PRIMARY KEY,
    owner_id BIGINT NOT NULL,
    lang VARCHAR NOT NULL,
    content VARCHAR NOT NULL,
    exercise_id BIGINT NOT NULL,
    ticket_status VARCHAR NOT NULL,
    results_id BIGINT
);