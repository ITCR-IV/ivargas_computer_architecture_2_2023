mod bus;
mod cache;
mod instructions;
mod main_memory;
mod processor;

use std::sync::mpsc::{sync_channel, Sender};

pub use bus::{Bus, BusSignal};
pub use cache::Cache;
pub use instructions::{Instruction, Operation};
pub use main_memory::Memory;
pub use processor::Processor;

use crate::app::Event;

type Data = u16;

pub struct SoC {
    processors: Vec<Processor>,
    bus: Bus,
    main_memory: Memory,
}

impl SoC {
    pub fn init_system(
        num_processors: usize,
        gui_sender: Sender<Event>,
    ) -> SoC {
        let (bus_tx, bus_rx) = sync_channel(0);

        let mut processors = Vec::with_capacity(num_processors);
        let mut bus = Bus::new(bus_rx);
        let main_memory = Memory::new(gui_sender.clone());

        for _ in 0..num_processors {
            let cache = Cache::new(gui_sender.clone());
            let processor =
                Processor::init(bus_tx.clone(), cache, gui_sender.clone());
            bus.register_processor(&processor);
            processors.push(processor);
        }

        SoC {
            processors,
            bus,
            main_memory,
        }
    }
}
