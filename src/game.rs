use crate::save;
use crossterm::event::{KeyCode, KeyEvent};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MOD {
    RUNNING(usize),
    EDITING(usize),
}

#[derive(Clone, Debug, Default)]
pub(crate) struct LoadDialog {
    pub(crate) saves: Vec<Vec<save::Cell>>,
    pub(crate) selected: usize,
}

impl LoadDialog {
    fn new(saves: Vec<Vec<save::Cell>>) -> Self {
        Self { saves, selected: 0 }
    }

    pub(crate) fn len(&self) -> usize {
        self.saves.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.saves.is_empty()
    }

    pub(crate) fn selected_number(&self) -> usize {
        self.selected + 1
    }

    pub(crate) fn selected_seed(&self) -> Option<&[save::Cell]> {
        self.saves.get(self.selected).map(Vec::as_slice)
    }

    fn move_selection(&mut self, delta: isize) {
        if self.is_empty() {
            return;
        }

        let last = self.saves.len().saturating_sub(1) as isize;
        self.selected = (self.selected as isize + delta).clamp(0, last) as usize;
    }

    fn select_first(&mut self) {
        self.selected = 0;
    }

    fn select_last(&mut self) {
        if !self.is_empty() {
            self.selected = self.saves.len() - 1;
        }
    }

    fn remove_selected(&mut self) -> Option<Vec<save::Cell>> {
        if self.is_empty() {
            return None;
        }

        let removed = self.saves.remove(self.selected);
        if self.selected >= self.saves.len() && !self.saves.is_empty() {
            self.selected = self.saves.len() - 1;
        } else if self.saves.is_empty() {
            self.selected = 0;
        }
        Some(removed)
    }
}

pub struct World {
    pub grid: Vec<Vec<bool>>,
    pub config: Config,
    pub state: MOD,
    pub generation: usize,
    pub(crate) load_dialog: Option<LoadDialog>,
    status_message: String,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub frames: usize,
    pub col: usize,
    pub row: usize,
}

fn get_seeds() -> Vec<save::Cell> {
    vec![(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)]
}

impl World {
    pub fn init_world(config: Config) -> World {
        let mut world = World {
            grid: vec![vec![false; config.col]; config.row],
            config,
            state: MOD::EDITING(0),
            generation: 0,
            load_dialog: None,
            status_message: format!(
                "press s to save current seed, Shift+L to load from {}",
                save::DEFAULT_SAVE_PATH
            ),
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
        self.load_dialog.is_none() && matches!(self.state, MOD::RUNNING(_))
    }

    pub fn is_load_dialog_open(&self) -> bool {
        self.load_dialog.is_some()
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if self.is_load_dialog_open() {
            return self.handle_load_dialog_key(key);
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => false,
            KeyCode::Char(' ') => {
                if !self.is_running() {
                    self.toggle_selected_cell();
                }
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
                self.toggle_running();
                true
            }
            KeyCode::Char('n') => {
                if !self.is_running() {
                    self.tick();
                }
                true
            }
            KeyCode::Char('r') => {
                self.clear();
                self.set_status_message("cleared board".to_string());
                true
            }
            KeyCode::Char('s') => {
                self.save_current_seed();
                true
            }
            KeyCode::Char('L') => {
                self.open_load_dialog();
                true
            }
            _ => true,
        }
    }

    pub fn mode_label(&self) -> &'static str {
        if self.is_load_dialog_open() {
            return "Load";
        }

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
        if let Some(dialog) = self.load_dialog.as_ref() {
            return match dialog.selected_seed() {
                Some(seed) => format!(
                    "mode=Load  file={}  saves={}  selected={}  alive={}",
                    save::DEFAULT_SAVE_PATH,
                    dialog.len(),
                    dialog.selected_number(),
                    seed.len()
                ),
                None => format!("mode=Load  file={}  saves=0", save::DEFAULT_SAVE_PATH),
            };
        }

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
        if self.is_load_dialog_open() {
            "j/k or arrows select  g/G top/bottom  enter load  d delete  esc close"
        } else {
            "arrows/hjkl move  space toggle  enter run/pause  n step  r clear  s save  L load  q quit"
        }
    }

    pub fn message_line(&self) -> &str {
        &self.status_message
    }

    fn reset(&mut self) {
        self.clear();
        get_seeds().into_iter().for_each(|cell| {
            if cell.1 < self.config.row && cell.0 < self.config.col {
                self.grid[cell.1][cell.0] = true;
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

    fn current_seed(&self) -> Vec<save::Cell> {
        self.grid
            .iter()
            .enumerate()
            .flat_map(|(y, row)| {
                row.iter()
                    .enumerate()
                    .filter_map(move |(x, &alive)| alive.then_some((x, y)))
            })
            .collect()
    }

    fn save_current_seed(&mut self) {
        let seed = self.current_seed();
        let live_cells = seed.len();

        match save::append_seed(save::DEFAULT_SAVE_PATH, seed) {
            Ok(slot) => self.set_status_message(format!(
                "saved {live_cells} live cells to {} as seed #{slot}",
                save::DEFAULT_SAVE_PATH
            )),
            Err(err) => self.set_status_message(format!(
                "failed to save current seed to {}: {err}",
                save::DEFAULT_SAVE_PATH
            )),
        }
    }

    fn open_load_dialog(&mut self) {
        match save::load(save::DEFAULT_SAVE_PATH) {
            Ok(saves) => {
                let save_count = saves.seeds.len();
                self.load_dialog = Some(LoadDialog::new(saves.seeds));
                self.set_status_message(match save_count {
                    0 => format!("no saved seeds found in {}", save::DEFAULT_SAVE_PATH),
                    1 => format!(
                        "opened load window for 1 saved seed in {}",
                        save::DEFAULT_SAVE_PATH
                    ),
                    count => format!(
                        "opened load window for {count} saved seeds in {}",
                        save::DEFAULT_SAVE_PATH
                    ),
                });
            }
            Err(err) => self
                .set_status_message(format!("failed to open {}: {err}", save::DEFAULT_SAVE_PATH)),
        }
    }

    fn handle_load_dialog_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.close_load_dialog("closed load window".to_string());
                true
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if let Some(dialog) = self.load_dialog.as_mut() {
                    dialog.move_selection(-1);
                }
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if let Some(dialog) = self.load_dialog.as_mut() {
                    dialog.move_selection(1);
                }
                true
            }
            KeyCode::Char('g') => {
                if let Some(dialog) = self.load_dialog.as_mut() {
                    dialog.select_first();
                }
                true
            }
            KeyCode::Char('G') => {
                if let Some(dialog) = self.load_dialog.as_mut() {
                    dialog.select_last();
                }
                true
            }
            KeyCode::Enter => {
                self.load_selected_seed();
                true
            }
            KeyCode::Char('d') => {
                self.delete_selected_seed();
                true
            }
            _ => true,
        }
    }

    fn close_load_dialog(&mut self, message: String) {
        self.load_dialog = None;
        self.set_status_message(message);
    }

    fn load_selected_seed(&mut self) {
        let (selected, seed) = match self.load_dialog.as_ref() {
            Some(dialog) => match dialog.selected_seed() {
                Some(seed) => (dialog.selected_number(), seed.to_vec()),
                None => {
                    self.set_status_message(format!(
                        "no saved seeds available in {}",
                        save::DEFAULT_SAVE_PATH
                    ));
                    return;
                }
            },
            None => return,
        };

        let loaded_cells = self.apply_seed(&seed);
        let skipped_cells = seed.len().saturating_sub(loaded_cells);
        self.load_dialog = None;

        if skipped_cells == 0 {
            self.set_status_message(format!(
                "loaded seed #{selected} from {} ({loaded_cells} live cells)",
                save::DEFAULT_SAVE_PATH
            ));
        } else {
            self.set_status_message(format!(
                "loaded seed #{selected} from {} ({loaded_cells} cells, skipped {skipped_cells} out of bounds)",
                save::DEFAULT_SAVE_PATH
            ));
        }
    }

    fn delete_selected_seed(&mut self) {
        let Some((selected, save_count)) = self
            .load_dialog
            .as_ref()
            .map(|dialog| (dialog.selected_number(), dialog.len()))
        else {
            return;
        };

        if save_count == 0 {
            self.set_status_message(format!(
                "no saved seeds available in {}",
                save::DEFAULT_SAVE_PATH
            ));
            return;
        }

        match save::delete_seed(save::DEFAULT_SAVE_PATH, selected - 1) {
            Ok(Some(removed)) => {
                if let Some(dialog) = self.load_dialog.as_mut() {
                    let _ = dialog.remove_selected();
                }

                let remaining = self.load_dialog.as_ref().map_or(0, LoadDialog::len);
                self.set_status_message(format!(
                    "deleted seed #{selected} from {} ({} live cells, {} remaining)",
                    save::DEFAULT_SAVE_PATH,
                    removed.len(),
                    remaining
                ));
            }
            Ok(None) => self.set_status_message(format!(
                "seed #{selected} no longer exists in {}",
                save::DEFAULT_SAVE_PATH
            )),
            Err(err) => self.set_status_message(format!(
                "failed to delete seed #{selected} from {}: {err}",
                save::DEFAULT_SAVE_PATH
            )),
        }
    }

    fn apply_seed(&mut self, seed: &[save::Cell]) -> usize {
        self.clear();
        self.state = MOD::EDITING(0);

        let mut loaded_cells = 0;
        let mut first_cell = None;

        for &(x, y) in seed {
            if y < self.config.row && x < self.config.col {
                self.grid[y][x] = true;
                loaded_cells += 1;
                first_cell.get_or_insert((x, y));
            }
        }

        if let Some((x, y)) = first_cell {
            self.set_cursor_index(y * self.config.col + x);
        } else {
            self.set_cursor_index(0);
        }

        loaded_cells
    }

    fn set_status_message(&mut self, message: String) {
        self.status_message = message;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_seed_resets_world_and_skips_out_of_bounds_cells() {
        let config = Config {
            frames: 30,
            col: 4,
            row: 3,
        };
        let mut world = World::init_world(config);
        world.generation = 9;
        world.state = MOD::RUNNING(3);

        let loaded = world.apply_seed(&[(1, 1), (3, 2), (9, 9)]);

        assert_eq!(loaded, 2);
        assert_eq!(world.generation, 0);
        assert_eq!(world.state, MOD::EDITING(5));
        assert!(world.grid[1][1]);
        assert!(world.grid[2][3]);
        assert!(!world.grid[0][0]);
    }

    #[test]
    fn load_dialog_navigation_clamps_to_available_seeds() {
        let mut dialog = LoadDialog::new(vec![vec![(0, 0)], vec![(1, 1)], vec![(2, 2)]]);

        dialog.move_selection(10);
        assert_eq!(dialog.selected, 2);
        assert_eq!(dialog.selected_number(), 3);

        dialog.move_selection(-10);
        assert_eq!(dialog.selected, 0);
        assert_eq!(dialog.selected_number(), 1);

        dialog.select_last();
        assert_eq!(dialog.selected, 2);

        dialog.select_first();
        assert_eq!(dialog.selected, 0);
    }

    #[test]
    fn removing_selected_seed_keeps_selection_in_bounds() {
        let mut dialog = LoadDialog::new(vec![vec![(0, 0)], vec![(1, 1)], vec![(2, 2)]]);
        dialog.select_last();

        let removed = dialog
            .remove_selected()
            .expect("selected save should exist");
        assert_eq!(removed, vec![(2, 2)]);
        assert_eq!(dialog.selected, 1);
        assert_eq!(dialog.len(), 2);

        let removed = dialog
            .remove_selected()
            .expect("selected save should exist");
        assert_eq!(removed, vec![(1, 1)]);
        assert_eq!(dialog.selected, 0);
        assert_eq!(dialog.len(), 1);

        let removed = dialog
            .remove_selected()
            .expect("selected save should exist");
        assert_eq!(removed, vec![(0, 0)]);
        assert_eq!(dialog.selected, 0);
        assert!(dialog.is_empty());
    }
}
