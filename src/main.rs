use std::{
    io::{self, stdout},
    time::{Duration, Instant},
};

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};

struct App {
    current_tab: usize,
    tabs: Vec<String>,
    status: String,
    branch: String,
}

impl App {
    fn new() -> App {
        App {
            current_tab: 0,
            tabs: vec![
                "Status".to_string(),
                "Changes".to_string(),
                "History".to_string(),
                "Settings".to_string(),
            ],
            status: "Ready".to_string(),
            branch: "main".to_string(),
        }
    }
}

fn main() -> Result<()> {
    // Terminal initialization
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, &app))?;
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Tab => {
                        app.current_tab = (app.current_tab + 1) % app.tabs.len();
                    }
                    KeyCode::BackTab => {
                        app.current_tab = if app.current_tab > 0 {
                            app.current_tab - 1
                        } else {
                            app.tabs.len() - 1
                        };
                    }
                    _ => {}
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Header
    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                "Git TUI ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("[{}]", app.branch),
                Style::default().fg(Color::Yellow),
            ),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    // Main content
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(chunks[1]);

    // Sidebar
    let items: Vec<ListItem> = app
        .tabs
        .iter()
        .map(|i| {
            let lines = vec![Line::from(vec![Span::styled(
                i,
                Style::default().fg(if app.current_tab == app.tabs.iter().position(|x| x == i).unwrap() {
                    Color::Yellow
                } else {
                    Color::White
                }),
            )])];
            ListItem::new(lines).style(Style::default())
        })
        .collect();

    let sidebar = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Menu"))
        .highlight_style(Style::default().bg(Color::DarkGray));
    f.render_widget(sidebar, main_chunks[0]);

    // Main content area
    let content = match app.current_tab {
        0 => "Status View\n\n• Working directory clean\n• 3 files staged\n• 2 files modified",
        1 => "Changes View\n\n• Modified: src/main.rs\n• Staged: README.md\n• Untracked: .gitignore",
        2 => "History View\n\n• feat: Add new feature\n• fix: Bug fix\n• chore: Update dependencies",
        3 => "Settings View\n\n• Editor: vim\n• Theme: dark\n• Auto-commit: enabled",
        _ => unreachable!(),
    };

    let main_content = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(app.tabs[app.current_tab].clone()));
    f.render_widget(main_content, main_chunks[1]);

    // Footer
    let footer = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                format!("Status: {}", app.status),
                Style::default().fg(Color::Green),
            ),
            Span::raw(" | "),
            Span::styled("Press 'q' to quit", Style::default().fg(Color::Gray)),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}
