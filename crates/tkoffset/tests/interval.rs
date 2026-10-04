//! Native BRepOffset interval storage, including reversed and nonfinite bounds.
use rustycad_tkfillet::chfi_ds::TypeOfConcavity;
use rustycad_tkoffset::brep_offset::BRepOffsetInterval;
#[test]
fn native_construction_and_independent_setters_match() {
    let variants = [
        TypeOfConcavity::Concave,
        TypeOfConcavity::Convex,
        TypeOfConcavity::Tangential,
        TypeOfConcavity::FreeBound,
        TypeOfConcavity::Other,
        TypeOfConcavity::Mixed,
    ];
    let expected: Vec<Vec<f64>> = include_str!("../../../tools/oracle/tkoffset_interval.expected")
        .lines()
        .map(|line| {
            line.split_whitespace()
                .map(|part| part.parse().unwrap())
                .collect()
        })
        .collect();
    for (index, variant) in variants.into_iter().enumerate() {
        let mut interval = BRepOffsetInterval::new(6., -2., variant);
        assert_eq!(observations(&interval).as_slice(), expected[index * 2]);
        interval.set_first(-7.);
        assert_eq!(interval.last().unwrap(), -2.);
        interval.set_last(9.);
        assert_eq!(interval.get_type().unwrap(), variant);
        interval.set_type(TypeOfConcavity::Other);
        assert_eq!(observations(&interval).as_slice(), expected[index * 2 + 1]);
    }
}
fn observations(interval: &BRepOffsetInterval) -> [f64; 3] {
    [
        interval.first().unwrap(),
        interval.last().unwrap(),
        f64::from(interval.get_type().unwrap() as i32),
    ]
}
#[test]
fn default_fields_remain_unset_until_individually_initialized() {
    let mut interval = BRepOffsetInterval::default();
    assert!(interval.first().is_err());
    assert!(interval.last().is_err());
    assert!(interval.get_type().is_err());
    interval.set_first(-0.);
    assert!(interval.first().unwrap().is_sign_negative());
    assert!(interval.last().is_err());
    assert!(interval.get_type().is_err());
    interval.set_last(f64::INFINITY);
    interval.set_type(TypeOfConcavity::Mixed);
    assert!(interval.last().unwrap().is_infinite());
    assert_eq!(interval.get_type().unwrap(), TypeOfConcavity::Mixed);
    interval.set_first(f64::NAN);
    assert!(interval.first().unwrap().is_nan());
}
