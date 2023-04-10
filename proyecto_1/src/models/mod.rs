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

fn box_err<'a, E: std::error::Error + 'a>(
    res: Result<(), E>,
) -> Result<(), Box<dyn std::error::Error + 'a>> {
    match res {
        Ok(()) => Ok(()),
        Err(err) => Err(Box::new(err)),
    }
}
