use std::io;

use crate::workbench::views::comhub::ComHub;
use crate::workbench::views::metadata::Metadata;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use datex_core::crypto::random::random_bytes_slice;
use datex_core::datex_values::Pointer;
use datex_core::runtime::Runtime;
use ratatui::layout::{Constraint, Direction, Layout};
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

pub struct Workbench<'a> {
    runtime: &'a Runtime,
    metadata: Metadata<'a>,
    comhub: ComHub<'a>,
    exit: bool,
}

impl<'a> Workbench<'a> {
    pub fn new(runtime: &Runtime) -> Workbench {
        // init the views
        let metadata = Metadata { runtime: &runtime };
        let comhub = ComHub { runtime: &runtime };

        Workbench {
            runtime,
            metadata,
            comhub,
            exit: false,
        }
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;

            // add ptr to the runtime
            let id = random_bytes_slice::<26>();
            self.runtime
                .memory
                .borrow_mut()
                .store_pointer(id, Pointer::from_id(id.to_vec()));
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Percentage(20),
                Constraint::Min(10),
            ])
            .split(frame.area());

        // draw the title
        self.draw_title(frame, layout[0]);

        // draw views
        frame.render_widget(&self.metadata, layout[1]);
        frame.render_widget(&self.comhub, layout[2]);
    }

    fn draw_title(&self, frame: &mut Frame, area: Rect) {
        let title = Line::from(vec![
            " DATEX Workbench ".bold(),
            format!("v{} ", self.runtime.version).dim(),
        ])
        .black();

        frame.render_widget(Paragraph::new(title).on_white(), area);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}
