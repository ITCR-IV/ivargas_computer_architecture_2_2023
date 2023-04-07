mod app;
mod constants;
mod models;
mod random;

use app::AppState;

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "Cache Sim",
        eframe::NativeOptions::default(),
        Box::new(|cc| Box::new(AppState::new(cc))),
    )
}
