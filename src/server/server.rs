use actix_web::{get, post, App, HttpResponse, HttpServer, Responder};
use futures::TryStreamExt;
use sqlx::{migrate::MigrateDatabase, Connection, Row, Sqlite, SqliteConnection};

const DB_URL: &str = "sqlite://sqlite.db";

#[get("/")]
async fn dump_log() -> impl Responder {
    let mut conn = SqliteConnection::connect(DB_URL).await.unwrap();
    let mut messages = sqlx::query("SELECT * FROM messages;").fetch(&mut conn);

    let mut response = Vec::new();
    while let Some(message) = messages.try_next().await.unwrap() {
        let id: u32 = message.try_get("id").unwrap();
        let msg: String = message.try_get("message").unwrap();
        response.push(format!("{{ \"id\": {id}, \"message\": \"{msg}\" }}"));
    }
    let response = response.join(", ");
    HttpResponse::Ok().body(format!("[ {response} ]"))
}

#[post("/")]
async fn send_message(msg: String) -> impl Responder {
    println!("Message received:\n\t{msg}");

    let mut conn = SqliteConnection::connect(DB_URL).await.unwrap();
    sqlx::query("INSERT INTO messages (message) VALUES (?)")
        .bind(msg.clone())
        .execute(&mut conn)
        .await
        .unwrap();

    HttpResponse::Ok().body(msg)
}

#[actix_web::main]
async fn main() -> Result<(), sqlx::Error> {
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

    sqlx::query("INSERT INTO messages (message) VALUES (\"more text\")")
        .execute(&mut conn)
        .await?;

    let mut messages = sqlx::query("SELECT * FROM messages;").fetch(&mut conn);

    while let Some(message) = messages.try_next().await? {
        let id: u32 = message.try_get("id")?;
        let msg: String = message.try_get("message")?;
        println!("Message {id}: {msg}");
    }

    let _ = HttpServer::new(|| App::new().service(dump_log).service(send_message))
        .bind(("0.0.0.0", 32123))?
        .run()
        .await;

    Ok(())
}
