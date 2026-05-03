mod game;
mod ui;

use crossterm::event::{self, Event, KeyEventKind};
use game::*;
use ratatui::DefaultTerminal;
use std::time::{Duration, Instant};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    ratatui::run(app)?;
    Ok(())
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let config = Config {
        frames: 60,
        col: 10,
        row: 10,
    };
    let mut w = World::init_world(config);
    let duration = Duration::from_secs_f64(1.0 / w.config.frames.max(1) as f64);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|frame| w.render(frame))?;

        let timeout = if w.is_running() {
            duration.saturating_sub(last_tick.elapsed())
        } else {
            Duration::from_millis(250)
        };

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && !w.handle_key(key) {
                    break Ok(());
                }
            }
        }

        if w.is_running() && last_tick.elapsed() >= duration {
            w.tick();
            last_tick = Instant::now();
        }
    }
}
