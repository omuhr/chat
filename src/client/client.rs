use actix_web::cookie::time::macros::offset;
use clap::Parser;
use crossterm::{
    event::{self, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::{CrosstermBackend, Stylize, Terminal},
    text::{Line, Text},
    widgets::Paragraph,
};
use reqwest::Client;
use serde::Deserialize;
use std::io::stdout;
use std::io::Result as IOResult;
use tokio::{sync::OnceCell, time::Instant};

mod tui;

static SERVER_URL: OnceCell<String> = OnceCell::const_new();

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    #[arg(short, long)]
    /// IP adress of the server
    adress: Option<String>,

    #[arg(short, long)]
    /// Message to be sent to the server
    message: Option<String>,

    #[arg(long, short, action=clap::ArgAction::SetTrue)]
    /// Whether to get and print chat history
    get: bool,

    #[arg(long, short, action=clap::ArgAction::SetFalse)]
    /// Flag whether to run TUI
    tui: bool,

    #[arg(long, short, default_value = "http://127.0.0.0:32123")]
    url: String,
}

#[derive(Debug, Deserialize)]
struct Msg {
    id: u32,
    message: String,
}

async fn message_history() -> Vec<String> {
    reqwest::get(get_url())
        .await
        .unwrap()
        .json::<Vec<Msg>>()
        .await
        .unwrap()
        .iter()
        .map(|msg| format!("{}: {}", msg.id, msg.message))
        .collect::<Vec<String>>()
}

fn get_url() -> String {
    SERVER_URL.get().expect("url is set").into()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    SERVER_URL.set(args.url).expect("must be unset");

    if args.message.is_none() && !args.get {
        let _ = run_tui().await;
        return Ok(());
    };

    if let Some(msg) = args.message {
        println!("Sending message:\n\t{msg}");

        let client = reqwest::Client::new();
        let res = client
            .post(SERVER_URL.get().expect("url is set"))
            .body(msg)
            .send()
            .await?;
        println!("Message sent, received response:\n\t{res:?}")
    };

    if args.get {
        let res: Vec<Msg> = reqwest::get(get_url()).await?.json().await?;
        println!("Message history:");
        for msg in res {
            println!("Message {}: {}", msg.id, msg.message)
        }
    }

    return Ok(());
}
