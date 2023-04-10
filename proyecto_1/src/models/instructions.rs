use crate::models::Data;

pub enum Instruction {
    Calc,
    Read { address: usize },
    Write { address: usize, data: Data },
}
