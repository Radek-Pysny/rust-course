-- Add migration script here
CREATE TABLE IF NOT EXISTS chat_messages (
    id          INTEGER PRIMARY KEY NOT NULL,
    login       TEXT NOT NULL,
    timestamp   TEXT NOT NULL,
    text        TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS client_logins (
    id          INTEGER PRIMARY KEY NOT NULL,
    login       TEXT NOT NULL,
    timestamp   TEXT NOT NULL
);
