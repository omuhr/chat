use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use futures::TryStreamExt;
use sqlx::Pool;
use sqlx::{
    migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Connection, Row, Sqlite, SqliteConnection,
};
const DB_URL: &str = "sqlite://sqlite.db";

#[get("/")]
async fn dump_log(req: HttpRequest) -> impl Responder {
    if let Some(val) = req.peer_addr() {
        println!("GET: {:?}", val.ip());
    };

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
async fn send_message(
    req: HttpRequest,
    msg: String,
    db: web::Data<Pool<Sqlite>>,
) -> impl Responder {
    let mut conn = db.acquire().await.unwrap();

    if let Some(val) = req.peer_addr() {
        println!("POST: {:?}", val.ip());
    };
    println!("Message received:\n\t{msg}");

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

    // let mut conn = SqliteConnection::connect(DB_URL).await?;
    let pool = SqlitePoolOptions::new()
        // .max_connections(5)
        .connect(DB_URL)
        .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS messages (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        message MEDIUMTEXT NOT NULL
        );",
    )
    .execute(&pool)
    .await?;

    sqlx::query("INSERT INTO messages (message) VALUES (\"more text\")")
        .execute(&pool)
        .await?;

    let mut messages = sqlx::query("SELECT * FROM messages;").fetch(&pool);

    while let Some(message) = messages.try_next().await? {
        let id: u32 = message.try_get("id")?;
        let msg: String = message.try_get("message")?;
        println!("Message {id}: {msg}");
    }

    let _ = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(dump_log)
            .service(send_message)
    })
    // .max_connections(1)
    .bind(("0.0.0.0", 32123))?
    .run()
    .await;

    Ok(())
}
