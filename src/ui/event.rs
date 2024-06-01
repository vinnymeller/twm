use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};
use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use anyhow::Result;

#[derive(Debug, Clone, Copy)]
pub enum Event {
    Tick,
    Key(KeyEvent),
}

pub struct EventHandler {
    receiver: mpsc::Receiver<Event>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (sender, receiver) = mpsc::channel();
        let _ = {
            let sender = sender.clone();

            thread::spawn(move || {
                let mut last_tick = Instant::now();
                loop {
                    let timeout = tick_rate
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or(tick_rate);
                    if event::poll(timeout).unwrap_or_default() {
                        let _ = match event::read() {
                            //.expect("Unable to read event") {
                            Ok(CrosstermEvent::Key(e)) => {
                                if e.kind == event::KeyEventKind::Press {
                                    sender.send(Event::Key(e))
                                } else {
                                    Ok(())
                                }
                            }
                            _ => Ok(()),
                        };
                    }

                    if last_tick.elapsed() >= tick_rate {
                        let _ = sender.send(Event::Tick);
                        last_tick = Instant::now();
                    }
                }
            })
        };
        Self { receiver }
    }

    pub fn next(&self) -> Result<Event> {
        self.receiver.recv().map_err(Into::into)
    }
}
