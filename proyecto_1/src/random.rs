// LCG algorithm constants, taken from:
//      L'Ecuyer, Pierre (1999). "Tables of Linear Congruential Generators of
//      Different Sizes and Good Lattice Structure". Mathematics of
//      Computation. doi:10.1090/S0025-5718-99-00996-5

use std::ops::{Bound, RangeBounds};

const M: u64 = 2u64.pow(32) - 5;
const A: u64 = 1588635695;
const C: u64 = 12345;

pub(crate) struct UniformRng {
    state: u64,
}

impl UniformRng {
    pub fn from_seed(seed: u32) -> UniformRng {
        UniformRng { state: seed.into() }
    }

    pub fn gen(&mut self) -> u32 {
        self.state = (A * self.state + C) % M;

        // Safe to cast because of modulo
        self.state as u32
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
