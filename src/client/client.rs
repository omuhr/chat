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

    fn get_content_length(&self) -> usize {
        self.content.chars().count()
    }

    fn get_char_indices(&self) -> Vec<usize> {
        return self.content.char_indices().map(|(idx, _)| idx).collect();
    }

    fn insert_character_at_cursor(&mut self, character: char) {
        if self.cursor_pos == self.get_content_length() {
            self.content.push(character);
        } else {
            let idx = self.get_char_indices()[self.cursor_pos];
            self.content.insert(idx, character);
        }
        self.shift_cursor_right();
    }

    fn remove_character_at_cursor(&mut self) -> Option<char> {
        if self.cursor_pos == 0 {
            return None;
        }

        self.shift_cursor_left();

        let idx = self.get_char_indices()[self.cursor_pos];
        Some(self.content.remove(idx))
    }

    fn shift_cursor_left(&mut self) {
        self.cursor_pos = self.cursor_pos.saturating_sub(1);
    }

    fn shift_cursor_right(&mut self) {
        if self.get_content_length() != self.cursor_pos {
            self.cursor_pos += 1;
        }
    }

    async fn send_message(&mut self, client: &Client) {
        client
            .post(get_url())
            .body(self.content.clone())
            .send()
            .await
            .unwrap();
        self.content = String::new();
        self.cursor_pos = 0;
    }
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

async fn run_tui() -> IOResult<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut input_field = InputField::new();
    let client = reqwest::Client::new();
    let prompt = "> ";

    // let mut msg_hist = message_history().await;
    let mut msg_hist = message_history().await;

    let mut now = Instant::now();

    let mut vertical_scroll: u16 = 0;

    loop {
        if now.elapsed().as_secs() > 1 {
            msg_hist = message_history().await;
            now = Instant::now();
        }
        terminal.draw(|frame| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Min(0), Constraint::Length(1)])
                .split(frame.size());
            let scrollback_area = layout[0];
            let input_field_area = layout[1];

            vertical_scroll = (msg_hist.len() as u16).saturating_sub(scrollback_area.height);

            let mut scrollback_text = vec![
                Line::from("");
                (scrollback_area.height as usize)
                    .saturating_sub(msg_hist.len()) // Number of blank filler lines needed
            ];

            let mut msg_hist_lines: Vec<Line> = msg_hist
                // FIXME cloning at each frame render, consider storing as vec of Line from the start.
                .clone()
                .into_iter()
                .map(|line| Line::from(line))
                .collect();

            scrollback_text.append(&mut msg_hist_lines);

            frame.render_widget(
                Paragraph::new(scrollback_text).scroll((vertical_scroll, 0)),
                scrollback_area,
            );

            frame.render_widget(
                Paragraph::new(format!("{}{}", prompt, input_field.content.as_str()))
                    .black()
                    .on_dark_gray(),
                input_field_area,
            );

            frame.set_cursor(
                input_field_area.x + prompt.len() as u16 + input_field.cursor_pos as u16,
                input_field_area.y,
            );
            std::thread::sleep(std::time::Duration::from_millis(10)); // Throtling the loop
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('c') => {
                        if key.modifiers == KeyModifiers::CONTROL {
                            break;
                        }
                        input_field.insert_character_at_cursor('c')
                    }
                    KeyCode::Char(c) => input_field.insert_character_at_cursor(c),
                    KeyCode::Enter => {
                        input_field.send_message(&client).await;
                        msg_hist = message_history().await;
                    }
                    KeyCode::Backspace => {
                        let _ = input_field.remove_character_at_cursor();
                    }
                    KeyCode::Left => input_field.shift_cursor_left(),
                    KeyCode::Right => input_field.shift_cursor_right(),
                    _ => {}
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
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
