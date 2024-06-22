use clap::Parser;
use serde::Deserialize;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    /// Message to be sent to the server
    #[arg(short, long)]
    msg: Option<String>,

    #[arg(long, short, action=clap::ArgAction::SetFalse)]
    /// Flag whether to get and print chat history
    should_print_history: bool,
}

#[derive(Debug, Deserialize)]
struct Msg {
    id: u32,
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if let Some(msg) = args.msg {
        println!("Sending message:\n\t{msg}");

        let client = reqwest::Client::new();
        let res = client
            .post("http://127.0.0.0:32123")
            .body(msg)
            .send()
            .await?;
        println!("Message sent, received response:\n\t{res:?}")
    };

    if args.should_print_history {
        let res: Vec<Msg> = reqwest::get("http://127.0.0.0:32123").await?.json().await?;
        println!("Message history:");
        for msg in res {
            println!("Message {}: {}", msg.id, msg.message)
        }
    }

    Ok(())
}
