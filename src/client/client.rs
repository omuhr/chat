use clap::Parser;
use crossterm::{
    event::{self, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::{CrosstermBackend, Stylize, Terminal},
    widgets::Paragraph,
};
use reqwest::Client;
use serde::Deserialize;
use std::io::stdout;
use std::io::Result as IOResult;

const SERVER_URL: &str = "http://127.0.0.0:32123";

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

struct InputField {
    content: String,
    cursor_pos: usize,
}

impl InputField {
    fn new() -> Self {
        InputField {
            content: String::new(),
            cursor_pos: 0,
        }
    }

    fn append_character(&mut self, character: char) {
        self.content.push(character);
    }

    fn pop_character(&mut self) -> Option<char> {
        if self.content.is_empty() {
            return None;
        }
        self.content.pop()
    }

    async fn send_message(&mut self, client: &Client) {
        client
            .post(SERVER_URL)
            .body(self.content.clone())
            .send()
            .await
            .unwrap();
        self.content = String::new();
    }
}

async fn run_tui() -> IOResult<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut input_field = InputField::new();
    let client = reqwest::Client::new();
    let prompt = "> ";

    loop {
        terminal.draw(|frame| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Min(0), Constraint::Length(1)])
                .split(frame.size());

            frame.render_widget(
                Paragraph::new(format!("{}{}", prompt, input_field.content.as_str()))
                    .black()
                    .on_dark_gray(),
                layout[1],
            );
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('c') => {
                            if key.modifiers == KeyModifiers::CONTROL {
                                break;
                            }
                            input_field.append_character('c')
                        }
                        KeyCode::Char(c) => input_field.append_character(c),
                        KeyCode::Enter => input_field.send_message(&client).await,
                        KeyCode::Backspace => {
                            let _ = input_field.pop_character();
                        }
                        _ => {}
                    }
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
        let _ = run_tui().await;
        return Ok(());
    }

    if let Some(msg) = args.msg {
        println!("Sending message:\n\t{msg}");

        let client = reqwest::Client::new();
        let res = client.post(SERVER_URL).body(msg).send().await?;
        println!("Message sent, received response:\n\t{res:?}")
    };

    if args.should_print_history {
        let res: Vec<Msg> = reqwest::get(SERVER_URL).await?.json().await?;
        println!("Message history:");
        for msg in res {
            println!("Message {}: {}", msg.id, msg.message)
        }
    }

    return Ok(());
}
