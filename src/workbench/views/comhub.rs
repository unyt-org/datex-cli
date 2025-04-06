use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use datex_core::runtime::Runtime;
use ratatui::style::{Color, Style};
use ratatui::widgets::Borders;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};

pub struct ComHub<'a> {
    pub runtime: &'a Runtime,
}

impl<'a> Widget for &ComHub<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" ComHub ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White));

        let lines = vec![
            Line::from(vec![
                "Registered Interfaces: ".into(),
                self.runtime
                    .com_hub
                    .borrow()
                    .interfaces
                    .len()
                    .to_string()
                    .bold(),
            ]),
            Line::from(vec![
                "Connected Sockets: ".into(),
                self.runtime
                    .com_hub
                    .borrow()
                    .sockets
                    .len()
                    .to_string()
                    .bold(),
            ]),
        ];

        Paragraph::new(Text::from_iter(lines))
            .block(block)
            .render(area, buf);
    }
}
