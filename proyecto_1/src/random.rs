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

    pub fn gen_range<R: RangeBounds<u32>>(&mut self, range: R) -> u32 {
        let inclusive_min = match range.start_bound() {
            Bound::Included(&min) => min,
            Bound::Excluded(&min) => min + 1,
            Bound::Unbounded => 0,
        };

        let inclusive_max = match range.end_bound() {
            Bound::Included(&max) => max,
            Bound::Excluded(&max) => max - 1,
            Bound::Unbounded => 0,
        };

        (self.gen() % (inclusive_max - inclusive_min + 1)) + inclusive_min
    }
}
