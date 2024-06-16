use futures::TryStreamExt;
use sqlx::{migrate::MigrateDatabase, Connection, Row, Sqlite, SqliteConnection};

const DB_URL: &str = "sqlite://sqlite.db";

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    println!("Hello, world!");

    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        println!("Creating database {}", DB_URL);
        match Sqlite::create_database(DB_URL).await {
            Ok(_) => println!("Create db success"),
            Err(error) => panic!("error: {}", error),
        }
    } else {
        println!("Database already exists");
    }

    let mut conn = SqliteConnection::connect(DB_URL).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS messages (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        message MEDIUMTEXT NOT NULL
        );",
    )
    .execute(&mut conn)
    .await?;

    sqlx::query("INSERT INTO messsages (message) VALUES (\"more text\")")
        .execute(&mut conn)
        .await?;

    let mut messages = sqlx::query("SELECT * FROM messages;").fetch(&mut conn);

    while let Some(message) = messages.try_next().await? {
        let id: u32 = message.try_get("id")?;
        let msg: String = message.try_get("message")?;
        println!("Message {id}: {msg}");
    }

    Ok(())
}
