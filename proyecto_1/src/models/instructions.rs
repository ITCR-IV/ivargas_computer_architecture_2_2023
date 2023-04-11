use std::fmt;

use crate::models::Data;

#[derive(Debug, Clone)]
pub enum Instruction {
    Calc,
    Read { address: usize },
    Write { address: usize, data: Data },
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Calc => write!(f, "calc"),
            Instruction::Read { address } => {
                write!(f, "read {address:#04b}")
            }
            Instruction::Write { address, data } => {
                write!(f, "write {address:#04b}; {data:#04X}")
            }
        }
    }
}
