mod bus;
mod cache;
mod instructions;
mod main_memory;
mod processor;

pub use bus::{Bus, BusSignal};
pub use cache::Cache;
pub use instructions::{Instruction, Operation};
pub use main_memory::Memory;
pub use processor::Processor;

type Data = u16;

pub struct SoC {
    processors: Vec<Processor>,
    bus: Bus,
    main_memory: Memory,
}

impl SoC {
    pub fn init_system(num_processors: usize) {}
}
