use crate::game::*;
use ratatui::{DefaultTerminal, Frame};

pub fn render(frame: &mut Frame) {
    frame.render_widget("Welcome to Conway's Game of Lift", frame.area());
}

pub fn draw_world(world: &World) {
    let rows = world.config.row;
    let cols = world.config.col;
    for i in 0..cols {
        for j in 0..rows {
            match world.grid[i][j] {
                true => print!("██"),
                false => print!(" "),
            }
        }
        println!();
    }
}
