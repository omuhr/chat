use clap::Parser;
use serde::Deserialize;

mod tui;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.message.is_none() && !args.get {
        let _ = tui::run_tui(args.url).await;
        return Ok(());
    };

    if let Some(msg) = args.message {
        println!("Sending message:\n\t{msg}");
        let client = reqwest::Client::new();
        let res = client.post(&args.url).body(msg).send().await?;
        println!("Message sent, received response:\n\t{res:?}")
    };

    if args.get {
        let res: Vec<Msg> = reqwest::get(&args.url).await?.json().await?;
        println!("Message history:");
        for msg in res {
            println!("Message {}: {}", msg.id, msg.message)
        }
    }

    return Ok(());
}
