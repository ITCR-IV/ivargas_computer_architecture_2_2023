// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use app::AppState;

pub mod random;
pub mod constants;
mod app;

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "Cache Sim",
        eframe::NativeOptions::default(),
        Box::new(|cc| Box::new(AppState::new(cc))),
    )
}
