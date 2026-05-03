pub struct World {
    pub grid: Vec<Vec<bool>>,
    pub config: Config,
}

struct Cell {
    x: usize,
    y: usize,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub frames: usize,
    pub col: usize,
    pub row: usize,
}
pub fn get_seeds() -> Vec<Cell> {
    vec![Cell { x: 0, y: 0 }]
}
pub fn init_world(config: Config) -> World {
    let seeds = get_seeds();
    let mut grid = vec![vec![false; config.col]; config.row];
    seeds.iter().for_each(|c| {
        grid[c.y][c.x] = true;
    });
    World { grid, config }
}
pub fn tick(w: &mut World) {
    let mut new_grid = init_world(w.config.clone()).grid;
    w.grid.iter().enumerate().for_each(|(i, row)| {
        row.iter().enumerate().for_each(|(j, cell)| {
            let new_cell = match cell {
                true => matches!(get_around(w, i, j), 2 | 3),
                false => get_around(w, i, j) == 3,
            };
            new_grid[i][j] = new_cell;
        })
    });
    std::mem::swap(&mut w.grid, &mut new_grid);
}
pub fn get_around(w: &World, x: usize, y: usize) -> usize {
    let rows = w.config.row;
    let cols = w.config.col;
    [-1i32, 0, 1]
        .iter()
        .flat_map(|&dx| [-1i32, 0, 1].iter().map(move |&dy| (dx, dy)))
        .filter(|&(dx, dy)| dx != 0 || dy != 0)
        .filter_map(|(dx, dy)| {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx >= 0 && ny >= 0 && nx < rows as i32 && ny < cols as i32 {
                Some((nx as usize, ny as usize))
            } else {
                None
            }
        })
        .filter(|&(nx, ny)| w.grid[nx][ny])
        .count()
}
