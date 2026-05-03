use crate::game::*;
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    prelude::{Color, Frame, Line, Span, Style, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

impl World {
    pub fn render(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(5),
            ])
            .split(frame.area());

        let title = Paragraph::new(Line::from(vec![
            Span::styled(
                "Conway's Game of Life",
                Style::default().fg(Color::Cyan).bold(),
            ),
            Span::raw("  "),
            Span::styled(
                format!("[{}]", self.mode_label()),
                match self.mode_label() {
                    "Running" => Style::default().fg(Color::Green),
                    "Load" => Style::default().fg(Color::Cyan),
                    _ => Style::default().fg(Color::Yellow),
                },
            ),
        ]))
        .block(Block::default().borders(Borders::ALL).title("UI"));

        let board = Paragraph::new(Text::from(self.board_lines()))
            .block(Block::default().borders(Borders::ALL).title("Board"))
            .wrap(Wrap { trim: false });

        let footer = Paragraph::new(Text::from(vec![
            Line::from(self.status_line()),
            Line::from(self.message_line()),
            Line::from(self.help_line()),
        ]))
        .block(Block::default().borders(Borders::ALL).title("Status"));

        frame.render_widget(title, chunks[0]);
        frame.render_widget(board, chunks[1]);
        frame.render_widget(footer, chunks[2]);

        if let Some(dialog) = self.load_dialog.as_ref() {
            self.render_load_dialog(frame, dialog);
        }
    }

    fn board_lines(&self) -> Vec<Line<'static>> {
        let editing = !self.is_running() && !self.is_load_dialog_open();
        let (cursor_x, cursor_y) = self.cursor();

        self.grid
            .iter()
            .enumerate()
            .map(|(y, row)| {
                let spans = row
                    .iter()
                    .enumerate()
                    .map(|(x, alive)| {
                        let is_cursor = editing && x == cursor_x && y == cursor_y;
                        let (symbol, style) = match (*alive, is_cursor) {
                            (true, true) => {
                                ("[]", Style::default().fg(Color::Black).bg(Color::Green))
                            }
                            (true, false) => ("██", Style::default().fg(Color::Green)),
                            (false, true) => ("[]", Style::default().fg(Color::Yellow)),
                            (false, false) => ("  ", Style::default()),
                        };
                        Span::styled(symbol.to_string(), style)
                    })
                    .collect::<Vec<_>>();
                Line::from(spans)
            })
            .collect()
    }

    fn render_load_dialog(&self, frame: &mut Frame, dialog: &LoadDialog) {
        let popup_area = centered_rect(frame.area(), 78, 72);
        frame.render_widget(Clear, popup_area);
        frame.render_widget(
            Block::default()
                .borders(Borders::ALL)
                .title("Load Seed")
                .border_style(Style::default().fg(Color::Cyan)),
            popup_area,
        );

        let inner = popup_area.inner(Margin {
            vertical: 1,
            horizontal: 1,
        });
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(8), Constraint::Length(2)])
            .split(inner);
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(32), Constraint::Min(20)])
            .split(sections[0]);

        let saves = Paragraph::new(Text::from(self.load_dialog_lines(dialog, body[0].height)))
            .block(Block::default().borders(Borders::ALL).title("Saves"))
            .wrap(Wrap { trim: false });
        let preview = Paragraph::new(Text::from(self.preview_lines(dialog, body[1].width)))
            .block(Block::default().borders(Borders::ALL).title("Preview"))
            .wrap(Wrap { trim: false });
        let help =
            Paragraph::new("j/k or arrows move  Enter load  d delete  Esc close  g/G first/last")
                .style(Style::default().fg(Color::DarkGray));

        frame.render_widget(saves, body[0]);
        frame.render_widget(preview, body[1]);
        frame.render_widget(help, sections[1]);
    }

    fn load_dialog_lines(&self, dialog: &LoadDialog, height: u16) -> Vec<Line<'static>> {
        if dialog.is_empty() {
            return vec![
                Line::from(""),
                Line::from(Span::styled(
                    "  no saved seeds",
                    Style::default().fg(Color::DarkGray),
                )),
            ];
        }

        let visible_rows = usize::from(height.saturating_sub(2)).max(1);
        let start = dialog
            .selected
            .saturating_sub(visible_rows.saturating_sub(1) / 2);
        let end = (start + visible_rows).min(dialog.len());
        let start = end.saturating_sub(visible_rows);

        let mut lines = Vec::new();
        if start > 0 {
            lines.push(Line::from(Span::styled(
                format!("  ... {} earlier save(s)", start),
                Style::default().fg(Color::DarkGray),
            )));
        }

        for index in start..end {
            let seed = &dialog.saves[index];
            let (width, height) = seed_bounds(seed).unwrap_or((0, 0));
            let selected = index == dialog.selected;
            let style = if selected {
                Style::default().fg(Color::Black).bg(Color::Yellow).bold()
            } else {
                Style::default()
            };
            let prefix = if selected { ">" } else { " " };
            let label = format!(
                "{prefix} {:>2}  {:>3} cells  {:>2}x{:<2}",
                index + 1,
                seed.len(),
                width,
                height
            );
            lines.push(Line::from(Span::styled(label, style)));
        }

        if end < dialog.len() {
            lines.push(Line::from(Span::styled(
                format!("  ... {} later save(s)", dialog.len() - end),
                Style::default().fg(Color::DarkGray),
            )));
        }

        lines
    }

    fn preview_lines(&self, dialog: &LoadDialog, width: u16) -> Vec<Line<'static>> {
        let Some(seed) = dialog.selected_seed() else {
            return vec![
                Line::from("no saved seeds"),
                Line::from(""),
                Line::from("press s in the main view to create one"),
            ];
        };

        let mut lines = Vec::new();
        let (min_x, min_y, max_x, max_y) = seed_extents(seed).unwrap_or((0, 0, 0, 0));
        let (bounds_w, bounds_h) = seed_bounds(seed).unwrap_or((0, 0));
        lines.push(Line::from(format!(
            "seed #{} from {}",
            dialog.selected_number(),
            crate::save::DEFAULT_SAVE_PATH
        )));
        lines.push(Line::from(format!("alive cells: {}", seed.len())));
        lines.push(Line::from(format!(
            "bounds: {}x{}  origin: ({}, {})",
            bounds_w, bounds_h, min_x, min_y
        )));
        lines.push(Line::from(""));

        let preview_width = usize::from(width.saturating_sub(6)).clamp(6, 18);
        let preview_height = 10usize;
        let display_width = bounds_w.min(preview_width);
        let display_height = bounds_h.min(preview_height);
        let cropped = bounds_w > preview_width || bounds_h > preview_height;

        if display_width == 0 || display_height == 0 {
            lines.push(Line::from("<empty seed>"));
            return lines;
        }

        let mut preview = vec![vec![false; display_width]; display_height];

        for &(x, y) in seed {
            let norm_x = x.saturating_sub(min_x);
            let norm_y = y.saturating_sub(min_y);
            if norm_x < display_width && norm_y < display_height {
                preview[norm_y][norm_x] = true;
            }
        }

        for row in preview {
            let line = row
                .into_iter()
                .map(|alive| if alive { "██" } else { "  " })
                .collect::<String>();
            lines.push(Line::from(line));
        }

        if cropped {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!(
                    "cropped preview from {}x{} to {}x{}",
                    max_x - min_x + 1,
                    max_y - min_y + 1,
                    display_width,
                    display_height
                ),
                Style::default().fg(Color::DarkGray),
            )));
        }

        lines
    }
}

fn centered_rect(area: Rect, width_percent: u16, height_percent: u16) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height_percent) / 2),
            Constraint::Percentage(height_percent),
            Constraint::Percentage((100 - height_percent) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_percent) / 2),
            Constraint::Percentage(width_percent),
            Constraint::Percentage((100 - width_percent) / 2),
        ])
        .split(vertical[1])[1]
}

fn seed_extents(seed: &[crate::save::Cell]) -> Option<(usize, usize, usize, usize)> {
    let mut cells = seed.iter();
    let &(first_x, first_y) = cells.next()?;
    let mut min_x = first_x;
    let mut min_y = first_y;
    let mut max_x = first_x;
    let mut max_y = first_y;

    for &(x, y) in cells {
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }

    Some((min_x, min_y, max_x, max_y))
}

fn seed_bounds(seed: &[crate::save::Cell]) -> Option<(usize, usize)> {
    seed_extents(seed).map(|(min_x, min_y, max_x, max_y)| (max_x - min_x + 1, max_y - min_y + 1))
}
