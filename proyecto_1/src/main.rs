#![feature(mutex_unlock)]
mod app;

mod constants;
mod models;
mod random;

use std::sync::mpsc::channel;

use app::AppState;

fn main() -> Result<(), eframe::Error> {
    let (gui_events_tx, gui_events_rx) = channel();

    eframe::run_native(
        "Cache Sim",
        eframe::NativeOptions::default(),
        Box::new(|cc| Box::new(AppState::new(cc, gui_events_rx))),
    )
}
