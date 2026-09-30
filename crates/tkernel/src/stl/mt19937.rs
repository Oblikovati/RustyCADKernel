//! `std::mt19937`, the 32-bit Mersenne Twister the native filler draws its order from.
//!
//! The C++ standard fixes the algorithm and its parameters ([rand.predef]), so every
//! conforming library yields the same sequence; the NCollection_UBTreeFiller seeds it
//! with 5489, BRepLib_PointCloudShape with 0.
const STATE_WORDS: usize = 624;
const SHIFT_WORDS: usize = 397;
const MATRIX_A: u32 = 0x9908_b0df;
const UPPER_MASK: u32 = 0x8000_0000;
const LOWER_MASK: u32 = 0x7fff_ffff;

/// Mersenne Twister state (`std::mt19937`).
///
/// ```
/// use rustycad_tkernel::stl::MersenneTwister;
/// assert_eq!(MersenneTwister::new(5489).next_u32(), 3_499_211_612);
/// ```
pub struct MersenneTwister {
    state: [u32; STATE_WORDS],
    position: usize,
}
impl MersenneTwister {
    /// Seeds with the standard's linear initialization (multiplier 1812433253);
    /// `MersenneTwister::new(0)`.
    pub fn new(seed: u32) -> Self {
        let mut state: [u32; STATE_WORDS] = [0; STATE_WORDS];
        state[0] = seed;
        for index in 1..STATE_WORDS {
            let previous: u32 = state[index - 1];
            state[index] = 1_812_433_253_u32
                .wrapping_mul(previous ^ (previous >> 30))
                .wrapping_add(index as u32);
        }
        Self {
            state,
            position: STATE_WORDS,
        }
    }
    /// Next tempered 32-bit output, regenerating the state every 624 draws;
    /// `generator.next_u32()`.
    pub fn next_u32(&mut self) -> u32 {
        if self.position >= STATE_WORDS {
            self.twist();
        }
        let mut value: u32 = self.state[self.position];
        self.position += 1;
        value ^= value >> 11;
        value ^= (value << 7) & 0x9d2c_5680;
        value ^= (value << 15) & 0xefc6_0000;
        value ^ (value >> 18)
    }
    fn twist(&mut self) {
        for index in 0..STATE_WORDS {
            let next: usize = (index + 1) % STATE_WORDS;
            let mixed: u32 = (self.state[index] & UPPER_MASK) | (self.state[next] & LOWER_MASK);
            let matrix: u32 = if mixed & 1 == 0 { 0 } else { MATRIX_A };
            self.state[index] =
                self.state[(index + SHIFT_WORDS) % STATE_WORDS] ^ (mixed >> 1) ^ matrix;
        }
        self.position = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::MersenneTwister;

    #[test]
    fn standard_default_seed_sequence() {
        let mut generator: MersenneTwister = MersenneTwister::new(5489);
        assert_eq!(generator.next_u32(), 3_499_211_612);
        // [rand.predef] requires the 10000th draw of a default mt19937 to be 4123659995.
        let tenth_thousand: u32 = (1..10_000).fold(0, |_, _| generator.next_u32());
        assert_eq!(tenth_thousand, 4_123_659_995);
    }
}
