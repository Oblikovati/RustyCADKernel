//! Mesh node storage and precision conversion oracle counterpart.
use super::{xy, xyz};
use rustycad_tkmath::{
    gp::{Point2, Point3},
    poly::{ArrayOfNodes, ArrayOfUvNodes},
};
pub(super) fn nodes_case(v: &[f64]) {
    spatial(v);
    planar(v);
    println!();
}
fn spatial(v: &[f64]) {
    let mut doubles = [[v[0], v[1], v[2]], [v[3], v[4], v[5]]];
    let mut source = ArrayOfNodes::from_double_slice(&mut doubles).unwrap();
    let mut target = ArrayOfNodes::default();
    target.set_double_precision(false).unwrap();
    target.resize(2, false).unwrap();
    target.assign(&source).unwrap();
    xyz(target.value(0).unwrap().xyz());
    xyz(target.value(1).unwrap().xyz());
    source.assign(&target).unwrap();
    xyz(source.value(0).unwrap().xyz());
    xyz(source.value(1).unwrap().xyz());
    target.set_value(0, Point3::new(v[3], v[1], v[5])).unwrap();
    copied_and_errors(&source, &mut target);
}
fn copied_and_errors(source: &ArrayOfNodes<'_>, target: &mut ArrayOfNodes<'_>) {
    let mut copy = target.clone();
    copy.resize(3, true).unwrap();
    xyz(copy.value(0).unwrap().xyz());
    xyz(copy.value(1).unwrap().xyz());
    print!(
        "{} {} {} {} ",
        i32::from(source.is_deletable()),
        i32::from(target.is_deletable()),
        i32::from(copy.is_double_precision()),
        copy.len()
    );
    print!(
        "{} ",
        target
            .set_double_precision(true)
            .unwrap_err()
            .exception_type()
    );
    let mut small = ArrayOfNodes::new(1).unwrap();
    print!("{} ", small.assign(target).unwrap_err().exception_type());
}
fn planar(v: &[f64]) {
    let mut uv = [[v[0], v[1]], [v[3], v[4]]];
    let mut source = ArrayOfUvNodes::from_double_slice(&mut uv).unwrap();
    let mut target = ArrayOfUvNodes::default();
    target.set_double_precision(false).unwrap();
    target.resize(2, false).unwrap();
    target.assign(&source).unwrap();
    xy(target.value(0).unwrap().xy());
    xy(target.value(1).unwrap().xy());
    target.set_value(1, Point2::new(v[2], v[5])).unwrap();
    source.assign(&target).unwrap();
    xy(source.value(0).unwrap().xy());
    xy(source.value(1).unwrap().xy());
}
