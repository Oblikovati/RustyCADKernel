//! Standard_Real Epsilon, Standard_Integer parity and the nested-field dump helpers.
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{integer, real};

#[test]
fn epsilon_measures_the_gap_away_from_zero() {
    assert_eq!(real::epsilon(1.0), f64::EPSILON);
    assert_eq!(real::epsilon(-1.0), f64::EPSILON);
    assert_eq!(real::epsilon(0.0), f64::from_bits(1));
    assert_eq!(real::epsilon(-0.0), f64::from_bits(1));
    assert_eq!(real::epsilon(f64::MAX), 0.0);
    assert_eq!(real::epsilon(f64::INFINITY), f64::NEG_INFINITY);
    assert_eq!(real::epsilon(f64::NEG_INFINITY), f64::NEG_INFINITY);
    assert!(real::epsilon(f64::NAN).is_nan());
}

#[test]
fn parity_uses_truncated_remainders() {
    assert!(integer::is_even(0) && integer::is_even(-4) && !integer::is_even(-3));
    assert!(integer::is_odd(3) && !integer::is_odd(-3) && !integer::is_odd(2));
}

#[test]
fn class_and_nested_fields_follow_separator_rules() {
    let mut stream = DumpStream::default();
    dump::dump_class_begin(&mut stream, "gp_Ax2d").unwrap();
    dump::dump_key_to_class(&mut stream, "loc", "\"gp_Pnt2d\": [1, 2]").unwrap();
    assert_eq!(
        stream.text(),
        "\"className\": \"gp_Ax2d\", \"loc\": {\"gp_Pnt2d\": [1, 2]}"
    );
}
