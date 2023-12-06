## DB setup

 1. Install `sqlx-cli`
    ```shell
    cargo install sqlx-cli
    ```

 2. Create DB
    ```shell
    cd server
    DATABASE_URL="sqlite://data.db" sqlx db create
    ```

 3. Run SQL migrations
    ```shell
    cd server
    DATABASE_URL="sqlite://data.db" sqlx migrate run
    ```


## Server compilation

Building server is tricky due to `sqlx` and SQLite paths. In my case, the following command 
(with modification of the given path for SQLite DB file) was OK:

```shell
DATABASE_URL="sqlite:/home/user/rust-course/lecture-2023-10-31-xchat/server/data.db" cargo build
```

Or put a `.env` file into root of server crate with almost that line:
`DATABASE_URL="sqlite:/home/user/rust-course/lecture-2023-10-31-xchat/server/data.db"`.


## Side notes

- Refactoring from `std::thread` into tasks of `tokio` is mostly OK. But then it took some time to find out how to do
    proper non-blocking receiving of data.
- Changing from `std::sync::mpsc::channel` to `flume::unbounded` is easy task.
- Introduction of `sqlx` is just extremely basic. Writing a records into two tables, nothing more than that.
- Security is not involved at all. Instead of hashing password, they are just turned into uppercase. Passwords are just
    checked for not being empty. If one gets an empty password, no welcome message is sent to the client and client is
    automatically disconnected within 5 second delay.
- The login sent by client is used on the server side to mark just text messages with prefix (as in good old chat apps).
