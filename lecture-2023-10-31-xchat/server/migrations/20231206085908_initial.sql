CREATE TABLE IF NOT EXISTS users (
     id         INTEGER PRIMARY KEY NOT NULL,
     login      TEXT NOT NULL,
     password   TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS chat_messages (
    id          INTEGER PRIMARY KEY NOT NULL,
    user_id     INTEGER NOT NULL,
    timestamp   TEXT NOT NULL,
    text        TEXT NOT NULL,
    FOREIGN KEY(user_id) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS client_logins (
    id          INTEGER PRIMARY KEY NOT NULL,
    user_id     INTEGER NOT NULL,
    timestamp   TEXT NOT NULL,
    FOREIGN KEY(user_id) REFERENCES users(id)
);
