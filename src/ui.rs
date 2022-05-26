use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyEvent};
use std::{cmp, fs};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::Spans,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

type Terminal = tui::Terminal<tui::backend::CrosstermBackend<std::io::Stdout>>;

#[derive(PartialEq)]
enum UiState {
    Main,
    FilePrompt,
    SearchPrompt,
}

fn prompt_string(s: &UiState) -> String {
    match s {
        UiState::Main => String::from(""),
        UiState::FilePrompt => String::from("ENTER FILENAME: "),
        UiState::SearchPrompt => String::from("SEARCH STRING: "),
    }
}

//#[derive(Debug)]
pub struct App {
    terminal: Terminal,
    filename: String,
    tmpbuf: String,
    content: String,
    search: String,
    lines: usize,
    log: String,
    cur: usize, // current position
    state: UiState,
}

impl App {
    pub fn new(filename: String) -> Result<Self> {
        let content = fs::read_to_string(&filename).context("could not read the file")?;

        let lines = count_newlines(&content);
        let terminal = Self::setup_terminal()?;
        Ok(Self {
            terminal,
            filename,
            tmpbuf: String::from(""),
            content,
            search: String::from(""),
            lines,
            log: String::from("<log text goes here>"),
            cur: 0,
            state: UiState::Main,
        })
    }

    // This it the main loop where we render the UI
    // and react to key events.
    pub fn run(mut self) -> anyhow::Result<()> {
        loop {
            self.render_ui()?;
            if self.handle_event()? {
                break;
            }
        }
        Ok(())
    }

    // Here we are polling for any key event to occur.
    fn handle_event(&mut self) -> anyhow::Result<bool> {
        while crossterm::event::poll(std::time::Duration::from_secs(0))? {
            match crossterm::event::read()? {
                crossterm::event::Event::Key(key) => {
                    if self.handle_key_event(key)? {
                        return Ok(true);
                    }
                }
                crossterm::event::Event::Resize(_, _) => {
                    self.render_ui()?;
                }
                _ => {}
            }
        }
        Ok(false)
    }

    // Take action depending on the key event.
    fn handle_key_event(&mut self, key: KeyEvent) -> anyhow::Result<bool> {
        match self.state {
            UiState::FilePrompt => self.handle_input_key_event(key),
            UiState::SearchPrompt => self.handle_input_key_event(key),
            UiState::Main => self.handle_main_key_event(key),
        }
    }

    fn handle_main_key_event(&mut self, key: KeyEvent) -> anyhow::Result<bool> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Ok(true),
            KeyCode::Char('o') => {
                self.state = UiState::FilePrompt;
                self.log = prompt_string(&self.state);
                Ok(false)
            }
            KeyCode::Char('s') => {
                self.state = UiState::SearchPrompt;
                self.log = prompt_string(&self.state);
                Ok(false)
            }
            KeyCode::Down => {
                self.cur += 1;
                self.log = "Got KeyCode Down".to_string();
                Ok(false)
            }
            KeyCode::Up => {
                if self.cur > 0 {
                    self.cur -= 1
                };
                self.log = "Got KeyCode Up".to_string();
                Ok(false)
            }
            x => {
                self.log = format!("Got KeyCode {:?}", x);
                Ok(false)
            }
        }
    }

    fn handle_input_key_event(&mut self, key: KeyEvent) -> anyhow::Result<bool> {
        match key.code {
            KeyCode::Esc => Ok(true),
            KeyCode::Enter if self.state == UiState::FilePrompt => {
                self.filename.clear();
                self.filename = self.tmpbuf.clone();
                self.log = format!("Got: {}", self.tmpbuf);
                self.tmpbuf.clear();
                self.cur = 0;
                let content = match fs::read_to_string(&self.filename) {
                    Ok(txt) => txt,
                    Err(e) => format!("ERROR: {:?}", e),
                };
                self.lines = count_newlines(&content);
                self.content = content;
                self.state = UiState::Main;
                Ok(false)
            }
            KeyCode::Enter if self.state == UiState::SearchPrompt => {
                self.search = self.tmpbuf.clone();
                self.log = format!("Got: {}", self.tmpbuf);
                self.tmpbuf.clear();
                self.state = UiState::Main;
                Ok(false)
            }
            KeyCode::Backspace => {
                self.tmpbuf.pop();
                self.log = format!("{}: {}", prompt_string(&self.state), self.tmpbuf);
                Ok(false)
            }
            KeyCode::Char(c) => {
                self.tmpbuf.push(c);
                self.log = format!("{} {}", prompt_string(&self.state), self.tmpbuf);
                Ok(false)
            }
            _x => Ok(false),
        }
    }

    // Render the UI
    fn render_ui(&mut self) -> anyhow::Result<()> {
        self.terminal.draw(|f| {
            main_ui(
                f,
                &mut self.cur,
                self.lines,
                &self.content,
                self.log.clone(),
            )
        })?;
        Ok(())
    }

    fn setup_terminal() -> anyhow::Result<Terminal> {
        crossterm::terminal::enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen,)?;
        let backend = tui::backend::CrosstermBackend::new(stdout);
        let terminal = tui::Terminal::new(backend)?;
        Ok(terminal)
    }

    fn teardown_terminal(&mut self) -> anyhow::Result<()> {
        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(
            self.terminal.backend_mut(),
            crossterm::terminal::LeaveAlternateScreen,
        )?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}

fn main_ui<B: Backend>(
    f: &mut Frame<B>,
    cur_pos: &mut usize,
    lines: usize,
    content: &String,
    logtext: String,
) {
    //
    // Create the Layout of the UI.
    //
    // We have three parts:
    //  - a frame with a help text for displaying the commands that can be used
    //  - a frame where the file content is shown
    //  - a frame where various internal log info is shown
    //
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(10),
                Constraint::Percentage(80),
                Constraint::Percentage(10),
            ]
            .as_ref(),
        )
        .split(f.size());

    //
    // Help frame
    //
    let helptext = "Quit=q/Esq , Scroll=Up/Down , OpenFile:o , Search:s";
    let help = Paragraph::new(helptext)
        .block(Block::default().title("Help").borders(Borders::ALL))
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    f.render_widget(help, chunks[0]);

    //
    // File content frame
    //
    let v: Vec<&str> = content.lines().collect();

    // Calculate the max amount of scrolling to be done
    // with respect to the number of lines and the amount
    // of lines displayed.
    let height = chunks[1].height as usize;
    let max_pos = if lines <= height {
        // The whole file is contained within the Frame.
        0
    } else {
        // Only allow scrolling until the last line of
        // the file is at the bottom of the Frame.
        cmp::min(lines - height + 2_usize, *cur_pos)
    };
    // Adjust cur_pos accordingly.
    *cur_pos = max_pos;

    let text: Vec<Spans> = (&v[max_pos..])
        .iter()
        .map(|line| Spans::from(*line))
        .collect();
    let para = Paragraph::new(text)
        .block(Block::default().title("File Content").borders(Borders::ALL))
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    f.render_widget(para, chunks[1]);

    //
    // Log frame
    //
    let logtext = logtext;
    let log = Paragraph::new(logtext)
        .block(Block::default().title("Log").borders(Borders::ALL))
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    f.render_widget(log, chunks[2]);
}

impl Drop for App {
    fn drop(&mut self) {
        let _x = self.teardown_terminal();
    }
}

fn count_newlines(s: &str) -> usize {
    s.as_bytes().iter().filter(|&&c| c == b'\n').count()
}
