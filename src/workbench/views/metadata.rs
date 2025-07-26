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
use std::cell::RefCell;
use std::rc::Rc;

pub struct Metadata {
    pub runtime: Rc<RefCell<Runtime>>,
}

impl Widget for &Metadata {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Runtime Info ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White));

        let runtime = self.runtime.borrow();

        let lines = vec![
            Line::from(vec![
                "Endpoint: ".into(),
                runtime.endpoint().to_string().bold(),
            ]),
            Line::from(vec!["Version: ".into(), runtime.version.clone().bold()]),
            Line::from(vec![
                "Allocated pointers: ".into(),
                runtime
                    .memory()
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
