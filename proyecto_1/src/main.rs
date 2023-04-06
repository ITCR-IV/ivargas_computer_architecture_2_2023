mod random;
mod constants;
mod app;

use app::AppState;

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "Cache Sim",
        eframe::NativeOptions::default(),
        Box::new(|cc| Box::new(AppState::new(cc))),
    )
}
