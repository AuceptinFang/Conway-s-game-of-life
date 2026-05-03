use crossterm::event::{KeyCode, KeyEvent};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MOD {
    RUNNING(usize),
    EDITING(usize),
}

pub struct World {
    pub grid: Vec<Vec<bool>>,
    pub config: Config,
    pub state: MOD,
    pub generation: usize,
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

fn get_seeds() -> Vec<Cell> {
    vec![
        Cell { x: 1, y: 0 },
        Cell { x: 2, y: 1 },
        Cell { x: 0, y: 2 },
        Cell { x: 1, y: 2 },
        Cell { x: 2, y: 2 },
    ]
}

impl World {
    pub fn init_world(config: Config) -> World {
        let mut world = World {
            grid: vec![vec![false; config.col]; config.row],
            config,
            state: MOD::EDITING(0),
            generation: 0,
        };
        world.reset();
        world
    }

    pub fn tick(&mut self) {
        let mut new_grid = vec![vec![false; self.config.col]; self.config.row];
        self.grid.iter().enumerate().for_each(|(y, row)| {
            row.iter().enumerate().for_each(|(x, cell)| {
                let new_cell = match cell {
                    true => matches!(self.get_around(y, x), 2 | 3),
                    false => self.get_around(y, x) == 3,
                };
                new_grid[y][x] = new_cell;
            })
        });
        std::mem::swap(&mut self.grid, &mut new_grid);
        self.generation += 1;
    }

    pub fn get_around(&self, y: usize, x: usize) -> usize {
        let rows = self.config.row as i32;
        let cols = self.config.col as i32;
        [-1i32, 0, 1]
            .iter()
            .flat_map(|&dy| [-1i32, 0, 1].iter().map(move |&dx| (dy, dx)))
            .filter(|&(dy, dx)| dy != 0 || dx != 0)
            .filter_map(|(dy, dx)| {
                let next_y = y as i32 + dy;
                let next_x = x as i32 + dx;
                if next_y >= 0 && next_x >= 0 && next_y < rows && next_x < cols {
                    Some((next_y as usize, next_x as usize))
                } else {
                    None
                }
            })
            .filter(|&(next_y, next_x)| self.grid[next_y][next_x])
            .count()
    }

    pub fn is_running(&self) -> bool {
        matches!(self.state, MOD::RUNNING(_))
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => false,
            KeyCode::Char(' ') => {
                self.toggle_running();
                true
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.move_cursor(-1, 0);
                true
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.move_cursor(1, 0);
                true
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_cursor(0, -1);
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_cursor(0, 1);
                true
            }
            KeyCode::Enter => {
                if !self.is_running() {
                    self.toggle_selected_cell();
                }
                true
            }
            KeyCode::Char('n') => {
                if !self.is_running() {
                    self.tick();
                }
                true
            }
            KeyCode::Char('c') => {
                self.clear();
                true
            }
            KeyCode::Char('r') => {
                self.reset();
                true
            }
            _ => true,
        }
    }

    pub fn mode_label(&self) -> &'static str {
        match self.state {
            MOD::RUNNING(_) => "Running",
            MOD::EDITING(_) => "Editing",
        }
    }

    pub fn cursor(&self) -> (usize, usize) {
        if self.config.col == 0 || self.config.row == 0 {
            return (0, 0);
        }
        let idx = self.cursor_index().min(self.board_len().saturating_sub(1));
        (idx % self.config.col, idx / self.config.col)
    }

    pub fn alive_cells(&self) -> usize {
        self.grid
            .iter()
            .map(|row| row.iter().filter(|cell| **cell).count())
            .sum()
    }

    pub fn status_line(&self) -> String {
        let (x, y) = self.cursor();
        format!(
            "mode={}  generation={}  cursor=({}, {})  alive={}",
            self.mode_label(),
            self.generation,
            x,
            y,
            self.alive_cells()
        )
    }

    pub fn help_line(&self) -> &'static str {
        "arrows/hjkl move  enter toggle  space run/pause  n step  c clear  r reset  q quit"
    }

    fn reset(&mut self) {
        self.clear();
        get_seeds().into_iter().for_each(|cell| {
            if cell.y < self.config.row && cell.x < self.config.col {
                self.grid[cell.y][cell.x] = true;
            }
        });
    }

    fn clear(&mut self) {
        self.grid.iter_mut().for_each(|row| row.fill(false));
        self.generation = 0;
    }

    fn toggle_running(&mut self) {
        let idx = self.cursor_index();
        self.state = match self.state {
            MOD::RUNNING(_) => MOD::EDITING(idx),
            MOD::EDITING(_) => MOD::RUNNING(idx),
        };
    }

    fn toggle_selected_cell(&mut self) {
        if self.config.col == 0 || self.config.row == 0 {
            return;
        }
        let (x, y) = self.cursor();
        self.grid[y][x] = !self.grid[y][x];
    }

    fn move_cursor(&mut self, dx: isize, dy: isize) {
        if self.config.col == 0 || self.config.row == 0 {
            return;
        }
        let (x, y) = self.cursor();
        let next_x = (x as isize + dx).clamp(0, self.config.col.saturating_sub(1) as isize);
        let next_y = (y as isize + dy).clamp(0, self.config.row.saturating_sub(1) as isize);
        self.set_cursor_index(next_y as usize * self.config.col + next_x as usize);
    }

    fn board_len(&self) -> usize {
        self.config.col.saturating_mul(self.config.row)
    }

    fn cursor_index(&self) -> usize {
        match self.state {
            MOD::RUNNING(idx) | MOD::EDITING(idx) => idx,
        }
    }

    fn set_cursor_index(&mut self, idx: usize) {
        let idx = idx.min(self.board_len().saturating_sub(1));
        self.state = match self.state {
            MOD::RUNNING(_) => MOD::RUNNING(idx),
            MOD::EDITING(_) => MOD::EDITING(idx),
        };
    }
}
