use crossterm::event::{KeyCode, KeyEvent};
use std::fs;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Spans,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

type Terminal = tui::Terminal<tui::backend::CrosstermBackend<std::io::Stdout>>;

enum UiState {
    Main,
    FilePrompt,
}

//#[derive(Debug)]
pub struct App {
    terminal: Terminal,
    filename: String,
    tmpname: String,
    log: String,
    cur: usize, // current position
    state: UiState,
}

impl App {
    pub fn new(filename: String) -> anyhow::Result<Self> {
        let terminal = Self::setup_terminal()?;
        Ok(Self {
            terminal,
            filename,
            tmpname: String::from(""),
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
            UiState::Main => self.handle_main_key_event(key),
        }
    }

    fn handle_main_key_event(&mut self, key: KeyEvent) -> anyhow::Result<bool> {
        match key.code {
            KeyCode::Char('q') => Ok(true),
            KeyCode::Char('o') => {
                self.log = format!("ENTER FILENAME: ");
                self.state = UiState::FilePrompt;
                Ok(false)
            }
            KeyCode::Down => {
                self.cur += 1;
                self.log = format!("Got KeyCode Down");
                Ok(false)
            }
            KeyCode::Up => {
                if self.cur > 0 {
                    self.cur -= 1
                };
                self.log = format!("Got KeyCode Up");
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
            KeyCode::Char('q') => Ok(true),
            KeyCode::Enter => {
                self.filename.clear();
                self.filename = self.tmpname.clone();
                self.tmpname.clear();
                self.cur = 0;
                self.log = format!("Got filename: {}", self.filename);
                self.state = UiState::Main;
                Ok(false)
            }
            KeyCode::Backspace => {
                self.tmpname.pop();
                self.log = format!("ENTER FILENAME: {}", self.tmpname);
                Ok(false)
            }
            KeyCode::Char(c) => {
                self.tmpname.push(c);
                self.log = format!("ENTER FILENAME: {}", self.tmpname);
                Ok(false)
            }
            _x => Ok(false),
        }
    }

    // Render the UI
    fn render_ui(&mut self) -> anyhow::Result<()> {
        self.terminal
            .draw(|f| main_ui(f, self.cur, &self.filename, self.log.clone()))?;
        // match self.state {
        //     UiState::Main => {
        //         self.terminal
        //             .draw(|f| main_ui(f, self.cur, &self.filename, self.log.clone()))?;
        //     }
        //     UiState::FilePrompt => {
        //         let mut filename = String::from("");
        //         self.terminal.draw(|f| get_filename(f, &mut filename))?;
        //         self.state = UiState::Main;
        //     }
        // }
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

fn main_ui<B: Backend>(f: &mut Frame<B>, cur_pos: usize, filename: &String, logtext: String) {
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
    let helptext = "Quit=q , Scroll=Up/Down , OpenFile:o";
    let help = Paragraph::new(helptext)
        .block(Block::default().title("Help").borders(Borders::ALL))
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    f.render_widget(help, chunks[0]);

    // FIXME stupid to read the file content every time we render the UI!!
    let contents = match fs::read_to_string(filename) {
        Ok(txt) => txt,
        Err(e) => format!("ERROR: {:?}", e),
    };
    let v: Vec<&str> = contents.split("\n").collect();
    let text: Vec<Spans> = (&v[cur_pos..])
        .iter()
        .map(|line| Spans::from(*line))
        .collect();

    //
    // File content frame
    //
    let para = Paragraph::new(text)
        .block(Block::default().title("File Content").borders(Borders::ALL))
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    f.render_widget(para, chunks[1]);

    //
    // Log frame
    //
    let logtext = format!("{}", logtext);
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

fn get_filename<B: Backend>(f: &mut Frame<B>, filename: &mut String) {
    let size = f.size();
    let area = centered_rect(60, 20, size);
    let block = Block::default().title("Popup").borders(Borders::ALL);
    f.render_widget(Clear, area);
    f.render_widget(block, area);
    filename.push_str("test.txt");
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
