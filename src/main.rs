mod game;
mod ui;

use game::*;
use ratatui::{DefaultTerminal, Frame};
use std::{
    thread::sleep,
    time::{Duration, SystemTime},
};
use ui::*;
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
    let mut w = init_world(config);

    let duration = Duration::from_secs_f64(1.0 / w.config.frames as f64);
    loop {
        terminal.draw(render)?;
        if crossterm::event::read()?.is_key_press() {
            break Ok(());
        }
        let now = SystemTime::now();
        tick(&mut w);
        draw_world(&w);
        let elapsed = SystemTime::now().duration_since(now).unwrap_or_default();
        sleep(duration.saturating_sub(elapsed));
    }
}

fn render(frame: &mut Frame) {
    frame.render_widget("Welcome to Conway's Game of Lift", frame.area());
}
