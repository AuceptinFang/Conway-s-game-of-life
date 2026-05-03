mod game;
mod ui;

use game::*;
use ratatui::DefaultTerminal;
use std::{
    thread::sleep,
    time::{Duration, SystemTime},
};

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

    let duration = Duration::from_secs_f64(1.0 / w.config.frames as f64);
    loop {
        let now = SystemTime::now();
        terminal.draw(|frame| w.render(frame))?;
        /*if crossterm::event::read()?.is_key_press() {
            break Ok(());
        }*/
        w.tick();
        let elapsed = SystemTime::now().duration_since(now).unwrap_or_default();
        sleep(duration.saturating_sub(elapsed));
    }
}
