pub mod bus;
pub mod cache;
pub mod instructions;
pub mod main_memory;
pub mod processor;
pub mod system;

pub type Data = u16;

pub enum MemOp {
    Write,
    Read,
}
