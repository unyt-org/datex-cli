
use datex_core::runtime::Runtime;
use ratatui::style::{Color, Style};
use ratatui::widgets::Borders;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};

pub struct Metadata<'a> {
    pub runtime: &'a Runtime,
}

impl<'a> Widget for &Metadata<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Runtime Info ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White));

        let lines = vec![
            Line::from(vec![
                "Version: ".into(),
                self.runtime.version.clone().bold(),
            ]),
            Line::from(vec![
                "Allocated pointers: ".into(),
                self.runtime
                    .memory
                    .borrow()
                    .get_pointer_ids()
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
