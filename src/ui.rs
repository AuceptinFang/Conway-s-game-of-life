use crate::game::*;

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
