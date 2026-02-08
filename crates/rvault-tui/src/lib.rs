pub mod app;
pub mod ui;
pub mod input;

use anyhow::Result;
use crossterm::{
    event,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use app::{App, AppState};

pub fn run() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new();
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> 
where B::Error: Into<io::Error>
{
    // Check initial session
    // If not in Setup mode, check session
    match app.state {
        AppState::Setup { .. } => {},
        _ => {
            if app.check_session() {
                 app.state = AppState::MainTable;
            }
        }
    }

    loop {
        terminal.draw(|f| ui::draw(f, &mut app)).map_err(|e| e.into())?;

        if event::poll(std::time::Duration::from_millis(100))? {
             if let event::Event::Key(key) = event::read()? {
                 app.tick();
                 if app.on_key(key)? {
                     return Ok(());
                 }
             }
        } else {
            app.tick();
        }
    }
}
