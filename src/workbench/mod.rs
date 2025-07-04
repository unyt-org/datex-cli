use std::cell::RefCell;
use crate::workbench::workbench::Workbench;
use datex_core::runtime::Runtime;
use std::io;
use std::rc::Rc;

mod views;
mod workbench;

pub async fn start_workbench(runtime: Rc<RefCell<Runtime>>) -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = Workbench::new(runtime).run(&mut terminal).await;
    ratatui::restore();
    app_result
}
