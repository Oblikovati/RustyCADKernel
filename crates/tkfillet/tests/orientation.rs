//! Exhaustive declared orientations and representative native choice signs/parities.
use rustycad_tkfillet::chfi3d::{next_side, next_side_on_face, same_side};
use rustycad_tkg3d::top_abs::Orientation;
#[test]
fn native_side_propagation_matches_all_declared_orientations() {
    for line in include_str!("../../../tools/oracle/tkfillet_orientation.expected").lines() {
        let mut fields = line.split_whitespace();
        let kind = fields.next().unwrap();
        let values: Vec<i32> = fields.map(|field| field.parse().unwrap()).collect();
        match kind {
            "single" => check_single(&values),
            "pair" => check_pair(&values),
            "same" => check_same(&values),
            _ => panic!("unexpected oracle row {line}"),
        }
    }
}
fn orientation(value: i32) -> Orientation {
    [
        Orientation::Forward,
        Orientation::Reversed,
        Orientation::Internal,
        Orientation::External,
    ][value as usize]
}
fn check_single(values: &[i32]) {
    let mut side = orientation(values[0]);
    next_side_on_face(&mut side, orientation(values[1]), orientation(values[2]));
    assert_eq!(side as i32, values[3]);
}
fn check_pair(values: &[i32]) {
    let mut first = orientation(values[0]);
    let mut second = orientation(values[1]);
    let choice = next_side(
        &mut first,
        &mut second,
        orientation(values[2]),
        orientation(values[3]),
        values[4],
    );
    assert_eq!([choice, first as i32, second as i32], values[5..8]);
}
fn check_same(values: &[i32]) {
    let actual = same_side(
        orientation(values[0]),
        orientation(values[1]),
        orientation(values[2]),
        orientation(values[3]),
        orientation(values[4]),
    );
    assert_eq!(i32::from(actual), values[5]);
}
