use std::{io, panic};

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::time::Duration;

use crate::ui::picker::Picker;

use super::EventHandler;
pub type CrosstermTerminal = ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stderr>>;

pub struct Tui {
    running: bool,
    terminal: CrosstermTerminal,
    pub events: EventHandler,
}

impl Tui {
    pub fn start() -> Result<Self> {
        let backend = CrosstermBackend::new(std::io::stderr());
        let terminal = Terminal::new(backend)?;
        let events = EventHandler::new(Duration::from_millis(15));
        let tui = Self::new(terminal, events);
        Ok(tui)
    }

    pub fn new(terminal: CrosstermTerminal, events: EventHandler) -> Self {
        Self {
            terminal,
            events,
            running: false,
        }
    }

    pub fn enter(&mut self) -> Result<()> {
        if self.running {
            return Ok(());
        }
        self.running = true;
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stderr(), EnterAlternateScreen, EnableMouseCapture)?;

        let panic_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic| {
            Self::reset().expect("Failed to reset the terminal");
            panic_hook(panic);
        }));

        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        Ok(())
    }

    fn reset() -> Result<()> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture)?;
        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        if !self.running {
            return Ok(());
        }
        Self::reset()?;
        self.terminal.show_cursor()?;
        self.running = false;
        Ok(())
    }

    pub fn draw(&mut self, picker: &mut Picker) -> Result<()> {
        self.terminal.draw(|frame| picker.render(frame))?;
        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        self.exit()
            .unwrap_or_else(|e| eprintln!("Failed to exit TUI: {}", e));
    }
}
