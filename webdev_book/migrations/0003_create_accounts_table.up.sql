CREATE EXTENSION citext;

CREATE DOMAIN domain_email AS citext
    CONSTRAINT email_format CHECK (VALUE ~ '^\w+@[a-zA-Z_]+?\.[a-zA-Z]{2,3}$');


-- Add up migration script here
CREATE TABLE IF NOT EXISTS accounts
(
    id       serial       NOT NULL PRIMARY KEY,
    email    domain_email NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL
);