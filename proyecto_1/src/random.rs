// LCG algorithm constants, taken from glibc implementation
use std::ops::{Bound, RangeBounds};

const M: u32 = 2u32.pow(31);
const A: u32 = 1103515245;
const C: u32 = 12345;

pub(crate) struct UniformRng {
    state: u32,
}

impl UniformRng {
    pub fn from_seed(seed: u32) -> UniformRng { UniformRng { state: seed } }

    pub fn gen(&mut self) -> u32 {
        self.state =
            ((A as u64 * self.state as u64 + C as u64) % M as u64) as u32;

        // Return 30 LSB, as per glibc implementation
        self.state & 0x3FFF_FFFF
    }
}
