use std::mem::variant_count;

use crate::{models::Data, random::UniformRng};

pub enum Instruction {
    Calc,
    Read { address: usize },
    Write { address: usize, data: Data },
}

impl Instruction {
    pub fn gen_random_instruction(rng: &mut UniformRng) -> Instruction {
        match rng.gen() % variant_count::<Instruction>() as u32 {
            0 => Instruction::Calc,
            1 => Instruction::Read {
                address: rng.gen() as usize,
            },
            2 => Instruction::Write {
                address: rng.gen() as usize,
                data: rng.gen() as u16,
            },
            _ => panic!("Unaccounted for instruction"),
        }
    }
}
