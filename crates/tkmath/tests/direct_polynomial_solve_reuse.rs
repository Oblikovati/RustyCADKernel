//! Replays protected OCCT Solve calls through public state-preserving Rust methods.
use rustycad_tkmath::math::DirectPolynomialRoots;

#[test]
fn quartic_overload_preserves_flags_and_replaces_solution_count() {
    let mut roots = DirectPolynomialRoots::quartic(1.0, 0.0, -5.0, 0.0, 4.0);
    assert_eq!(
        root_bits::<4>(&roots),
        [
            0xc000_0000_0000_0000,
            0xbff0_0000_0000_0000,
            0x4000_0000_0000_0000,
            0x3ff0_0000_0000_0000
        ]
    );
    roots.solve_quartic(1.0, 0.0, 1.0, 0.0, 1.0);
    assert!(roots.is_done());
    assert_eq!(roots.solutions().unwrap(), 0);
}

#[test]
fn cubic_overflow_keeps_prior_count_roots_and_failed_status() {
    let mut roots = DirectPolynomialRoots::cubic(1.0, -6.0, 11.0, -6.0);
    roots.solve_cubic(1.0, 1.0e60, 1.0, 1.0);
    assert!(!roots.is_done());
    assert_eq!(roots.solutions().unwrap(), 3);
    assert_eq!(
        root_bits::<3>(&roots),
        [
            0x4008_0000_0000_0000,
            0x4000_0000_0000_0000,
            0x3ff0_0000_0000_0000
        ]
    );
}

#[test]
fn cubic_success_does_not_reset_previous_failure_status() {
    let mut roots = DirectPolynomialRoots::cubic(1.0, 1.0e60, 1.0, 1.0);
    roots.solve_cubic(1.0, -6.0, 11.0, -6.0);
    assert!(!roots.is_done());
    assert_eq!(roots.solutions().unwrap(), 3);
    assert_eq!(
        root_bits::<3>(&roots),
        [
            0x4008_0000_0000_0000,
            0x4000_0000_0000_0000,
            0x3ff0_0000_0000_0000
        ]
    );
}

#[test]
fn quadratic_no_real_roots_replaces_count_without_changing_done() {
    let mut roots = DirectPolynomialRoots::quadratic(1.0, -5.0, 6.0);
    assert_eq!(
        root_bits::<2>(&roots),
        [0x4008_0000_0000_0000, 0x4000_0000_0000_0000]
    );
    roots.solve_quadratic(1.0, 1.0, 1.0);
    assert!(roots.is_done());
    assert_eq!(roots.solutions().unwrap(), 0);
}

#[test]
fn linear_infinite_flag_blocks_later_overloads_without_being_cleared() {
    let mut roots = DirectPolynomialRoots::linear(1.0, -2.0);
    roots.solve_linear(0.0, 0.0);
    roots.solve_linear(1.0, -3.0);
    roots.solve_quadratic(1.0, -5.0, 6.0);
    assert!(roots.is_done());
    assert!(roots.infinite_roots());
    assert_eq!(
        roots.solutions().unwrap_err().exception_type(),
        "StdFail_InfiniteSolutions"
    );
}

#[test]
fn linear_success_updates_root_and_preserves_done_state() {
    let mut roots = DirectPolynomialRoots::linear(1.0, -2.0);
    roots.solve_linear(1.0, -3.0);
    assert!(roots.is_done());
    assert_eq!(roots.value(1).unwrap().to_bits(), 0x4008_0000_0000_0000);
}

fn root_bits<const N: usize>(roots: &DirectPolynomialRoots) -> [u64; N] {
    std::array::from_fn(|index| roots.value(index as i32 + 1).unwrap().to_bits())
}
