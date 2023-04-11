use crate::models::Data;

#[derive(Debug)]
pub enum Instruction {
    Calc,
    Read { address: usize },
    Write { address: usize, data: Data },
}
