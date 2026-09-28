//! Standard_Integer: truncated remainders, std::max/min and the documented overflow choices.
use rustycad_tkernel::standard::integer;

#[test]
fn parity_uses_the_truncated_remainder() {
    assert!(integer::is_even(i32::MIN) && !integer::is_odd(i32::MIN));
    assert!(integer::is_odd(i32::MAX));
    // -3 % 2 == -1: a negative odd value is neither odd nor even, as natively.
    assert!(!integer::is_odd(-3) && !integer::is_even(-3));
}

#[test]
fn abs_max_min_modulus_and_square() {
    assert_eq!(integer::abs(-i32::MAX), i32::MAX);
    assert_eq!((integer::max(3, -4), integer::min(3, -4)), (3, -4));
    assert_eq!(integer::modulus(-7, 3), -1);
    assert_eq!(integer::modulus(7, -3), 1);
    assert_eq!(integer::modulus(i32::MIN, 3), -2);
    assert_eq!(integer::square(46340), 2_147_395_600);
}

#[test]
fn undefined_overflow_takes_the_x86_64_result() {
    assert_eq!(integer::abs(i32::MIN), i32::MIN);
    assert_eq!(integer::square(65536), 0);
    assert_eq!(integer::square(46341), -2_147_479_015);
}

#[test]
#[should_panic(expected = "remainder with a divisor of zero")]
fn modulus_by_zero_traps() {
    let divisor = std::hint::black_box(0);
    integer::modulus(1, divisor);
}

#[test]
#[should_panic(expected = "overflow")]
fn modulus_of_int_min_by_minus_one_traps() {
    let divisor = std::hint::black_box(-1);
    integer::modulus(i32::MIN, divisor);
}

#[test]
fn limits_match_limits_h() {
    assert_eq!(integer::integer_first(), -2_147_483_648);
    assert_eq!(integer::integer_last(), 2_147_483_647);
    assert_eq!(integer::integer_size(), 32);
}
