//! Exact integer and floating-point stream oracle for MathUtils_Random.hxx at 3d097a0.
use rustycad_tkmath::math_utils::RandomGenerator;
#[test]
fn seeded_streams_match_every_native_bit() {
    let fixture: &str = include_str!("fixtures/mathutils_random.txt");
    let mut generator: RandomGenerator = RandomGenerator::default();
    let mut previous: Option<u64> = None;
    for line in fixture.lines() {
        let words: Vec<u64> = line
            .split_whitespace()
            .map(|w| w.parse().unwrap())
            .collect();
        if previous != Some(words[0]) {
            generator.set_seed(words[0]);
            previous = Some(words[0]);
        }
        assert_eq!(generator.next_int(), words[1]);
        assert_eq!(generator.next_real().to_bits(), words[2]);
    }
}
#[test]
fn default_and_reseed_restart_the_stream() {
    let mut default: RandomGenerator = RandomGenerator::default();
    let mut explicit: RandomGenerator = RandomGenerator::new(1);
    let initial: u64 = explicit.next_int();
    assert_eq!(default.next_int(), initial);
    default.set_seed(1);
    assert_eq!(default.next_int(), initial);
}
