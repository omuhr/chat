use clap::Parser;
use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{CrosstermBackend, Stylize, Terminal},
    widgets::Paragraph,
};
use serde::Deserialize;
use std::io::stdout;
use std::io::Result as IOResult;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    #[arg(short, long)]
    /// Message to be sent to the server
    msg: Option<String>,

    #[arg(long, short, action=clap::ArgAction::SetFalse)]
    /// Flag whether to get and print chat history
    should_print_history: bool,

    #[arg(long, short, action=clap::ArgAction::SetTrue)]
    /// Flag whether to run TUI
    tui: bool,
}

#[derive(Debug, Deserialize)]
struct Msg {
    id: u32,
    message: String,
}

async fn run_tui() -> IOResult<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    loop {
        terminal.draw(|frame| {
            let area = frame.size();
            frame.render_widget(
                Paragraph::new("Chat! (press 'q' to quit)")
                    .black()
                    .on_dark_gray(),
                area,
            );
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.tui {
        run_tui().await;
        return Ok(());
    }

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

    return Ok(());
}
