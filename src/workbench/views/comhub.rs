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

pub struct ComHub<'a> {
    pub runtime: &'a Runtime,
}

impl<'a> Widget for &ComHub<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let metadata = self.runtime.com_hub.borrow().get_metadata();

        let block = Block::default()
            .title(" ComHub ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White));

        let lines = vec![
            Line::from(vec![
                "Registered Interfaces: ".into(),
                metadata.interfaces.len().to_string().into(),
            ]),
            Line::from(vec![
                "Connected Sockets: ".into(),
                metadata.endpoint_sockets.len().to_string().into(),
            ]),
        ];

        Paragraph::new(Text::from_iter(lines))
            .block(block)
            .render(area, buf);
    }
}
