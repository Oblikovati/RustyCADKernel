//! Supplemental aliasing cases preserve the pinned native row publication order.
use rustycad_tkmath::{geom_abs::Shape, plib::HermitJacobi};

const STARTS: [[usize; 4]; 4] = [[0, 0, 0, 0], [1, 3, 2, 0], [9, 6, 3, 0], [1, 12, 23, 34]];

fn check_native_row(line: &str) {
    let fields: Vec<u64> = line
        .split_whitespace()
        .map(|s| s.parse().unwrap())
        .collect();
    let shapes: [Shape; 3] = [Shape::C0, Shape::C1, Shape::C2];
    let basis: HermitJacobi = HermitJacobi::new(8, shapes[fields[0] as usize]).unwrap();
    let mut values: [f64; 48] = std::array::from_fn(|i| 700.0 + i as f64);
    let order: i32 = fields[1] as i32;
    let starts: [usize; 4] = STARTS[fields[2] as usize];
    basis
        .d0123_overlapping(order, 0.37, &mut values, starts)
        .unwrap();
    let bits: Vec<u64> = values.iter().map(|v| v.to_bits()).collect();
    assert_eq!(bits, fields[3..], "level/order/pattern {:?}", &fields[..3]);
}

#[test]
fn overlapping_outputs_match_native_bits_and_untouched_canaries() {
    let cases: Vec<&str> = include_str!("../../../tools/oracle/tkmath_hermit_overlapping.expected")
        .lines()
        .collect();
    assert_eq!(cases.len(), 48);
    cases.into_iter().for_each(check_native_row);
}

#[test]
fn invalid_requested_spans_and_orders_leave_output_unchanged() {
    for (order, starts) in [
        (3, [0, 0, 0, 1]),
        (1, [0, usize::MAX, 0, 0]),
        (-1, [0; 4]),
        (4, [0; 4]),
    ] {
        assert_invalid_request_preserves_output(order, starts);
    }
}

fn assert_invalid_request_preserves_output(order: i32, starts: [usize; 4]) {
    let basis: HermitJacobi = HermitJacobi::new(8, Shape::C1).unwrap();
    let mut values: [f64; 9] = [777.0; 9];
    assert!(
        basis
            .d0123_overlapping(order, 0.37, &mut values, starts)
            .is_err()
    );
    assert_eq!(values, [777.0; 9]);
}

#[test]
fn unrequested_row_starts_are_not_dereferenced() {
    let basis: HermitJacobi = HermitJacobi::new(8, Shape::C1).unwrap();
    let mut values: [f64; 9] = [777.0; 9];
    basis
        .d0123_overlapping(
            0,
            0.37,
            &mut values,
            [0, usize::MAX, usize::MAX, usize::MAX],
        )
        .unwrap();
    assert!(values.iter().all(|v| v.is_finite() && *v != 777.0));
}
