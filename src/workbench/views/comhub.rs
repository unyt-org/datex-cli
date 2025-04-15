use std::cell::RefCell;
use std::rc::Rc;
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

pub struct ComHub {
    pub runtime: Rc<RefCell<Runtime>>
}

impl Widget for &ComHub {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let runtime = self.runtime.borrow();
        let metadata = runtime.com_hub.lock().unwrap().get_metadata();

        let block = Block::default()
            .title(" ComHub ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White));

        let mut lines = vec![
            Line::from(vec![
                "Registered Interfaces: ".into(),
                metadata.interfaces.len().to_string().into(),
            ]),
            Line::from(vec![
                "Connected Sockets: ".into(),
                metadata.endpoint_sockets.len().to_string().into(),
            ]),
        ];

        // iterate interfaces
        for (i, interface) in metadata.interfaces.iter().enumerate() {
            lines.push(Line::from(vec![
                format!("Interface {}: ", i).into(),
                interface.properties.name.clone()
                    .map_or_else(|| "None".into(), |i| i.to_string().into()),
            ]));
        }

        Paragraph::new(Text::from_iter(lines))
            .block(block)
            .render(area, buf);
    }
}
