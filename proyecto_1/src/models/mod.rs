mod bus;
mod cache;
mod instructions;
mod main_memory;
mod processor;

pub use bus::Bus;
pub use cache::Cache;
pub use instructions::{Instruction, Operation};
pub use main_memory::Memory;
pub use processor::Processor;

pub struct SoC {
    processors: Vec<Processor>,
    bus: Bus,
    main_memory: Memory,
}
