use crossterm::event::{KeyCode, KeyEvent};
use std::fs;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::Spans,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

type Terminal = tui::Terminal<tui::backend::CrosstermBackend<std::io::Stdout>>;

//#[derive(Debug)]
pub struct App {
    terminal: Terminal,
    filename: String,
    log: String,
}

impl App {
    pub fn new(filename: String) -> anyhow::Result<Self> {
        let terminal = Self::setup_terminal()?;
        Ok(Self {
            terminal,
            filename,
            log: String::from("<log text goes here>"),
        })
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        loop {
            self.render_ui()?;
            if self.handle_event()? {
                break;
            }
        }
        Ok(())
    }

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

    fn handle_key_event(&mut self, key: KeyEvent) -> anyhow::Result<bool> {
        match key.code {
            KeyCode::Char('q') => {
                return Ok(true);
            }
            x => {
                self.log = format!("Got KeyCode {:?}", x);
                return Ok(false);
            }
        }
    }

    fn render_ui(&mut self) -> anyhow::Result<()> {
        self.terminal
            .draw(|f| ui(f, &self.filename, self.log.clone()))?;
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

fn ui<B: Backend>(f: &mut Frame<B>, filename: &String, logtext: String) {
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

    let help = Block::default().title("Help").borders(Borders::ALL);
    f.render_widget(help, chunks[0]);

    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");
    let mut text = Vec::new();

    for line in contents.split("\n") {
        text.push(Spans::from(line));
    }

    let para = Paragraph::new(text)
        .block(Block::default().title("Paragraph").borders(Borders::ALL))
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    f.render_widget(para, chunks[1]);

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
