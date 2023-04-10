#![feature(mutex_unlock)]
#![feature(variant_count)]
mod app;

mod models;
mod random;

use std::sync::mpsc::channel;

use app::AppState;
use models::system::{self, SocProperties};

const SYSTEM_PROPS: SocProperties = SocProperties {
    num_processors: 4,
    cache_associativity: 2,
    cache_sets: 2,
    main_memory_blocks: 8,
};

fn main() -> Result<(), eframe::Error> {
    let (gui_events_tx, gui_events_rx) = channel();

    let instruction_inputs = system::init_system(SYSTEM_PROPS, gui_events_tx);

    eframe::run_native(
        "Cache Sim",
        eframe::NativeOptions::default(),
        Box::new(|cc| {
            Box::new(AppState::new(
                cc,
                gui_events_rx,
                instruction_inputs,
                SYSTEM_PROPS,
            ))
        }),
    )
}
