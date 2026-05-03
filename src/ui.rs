use crate::game::*;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::{Color, Frame, Line, Span, Style, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

impl World {
    pub fn render(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(4),
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
                match self.is_running() {
                    true => Style::default().fg(Color::Green),
                    false => Style::default().fg(Color::Yellow),
                },
            ),
        ]))
        .block(Block::default().borders(Borders::ALL).title("UI"));

        let board = Paragraph::new(Text::from(self.board_lines()))
            .block(Block::default().borders(Borders::ALL).title("Board"))
            .wrap(Wrap { trim: false });

        let footer = Paragraph::new(Text::from(vec![
            Line::from(self.status_line()),
            Line::from(self.help_line()),
        ]))
        .block(Block::default().borders(Borders::ALL).title("Input"));

        frame.render_widget(title, chunks[0]);
        frame.render_widget(board, chunks[1]);
        frame.render_widget(footer, chunks[2]);
    }

    fn board_lines(&self) -> Vec<Line<'static>> {
        let editing = !self.is_running();
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
                                ("<>", Style::default().fg(Color::Black).bg(Color::Green))
                            }
                            (true, false) => ("##", Style::default().fg(Color::Green)),
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
}
