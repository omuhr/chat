use sqlx::{Connection, SqliteConnection, };

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    println!("Hello, world!");

    let mut conn = SqliteConnection::connect("sqlite::memory:").await?;

    sqlx::query("BEGIN").execute(&mut conn).await?;

    sqlx::query(
        "CREATE TABLE messages (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        message MEDIUMTEXT NOT NULL
        );"
        ).execute(&mut conn).await?;

    sqlx::query(
        "INSERT INTO messages (message) VALUES (\"text\")"
        ).execute(&mut conn).await?;

    let mut messages = sqlx::query(
        "SELECT * FROM messages;"
        ).fetch(&mut conn);

    // while let Some(message) = messages.try_next().await? {
    //     println!("{:?}", message);
    // }

    Ok(())
}
