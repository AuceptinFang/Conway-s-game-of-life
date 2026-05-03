use crate::game::*;
use ratatui::Frame;

impl World {
    pub fn render(&self, frame: &mut Frame) {
        frame.render_widget("Welcome to Conway's Game of Lift", frame.area());
        self.draw();
    }
    pub fn draw(&self) {
        let rows = self.config.row;
        let cols = self.config.col;
        for i in 0..cols {
            for j in 0..rows {
                match self.grid[i][j] {
                    true => print!("██"),
                    false => print!(" "),
                }
            }
            println!();
        }
    }
}
