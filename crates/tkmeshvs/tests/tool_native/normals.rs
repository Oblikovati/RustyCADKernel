//! Signed bounds, header counts, non-planarity and failure keep native output timing.
use super::T;
use super::snapshot::real;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkmath::gp::Vector3;
use std::fmt::Write;
pub(super) fn replay(rows: &mut String) {
    for (index, values) in NORMALS.iter().enumerate() {
        let lower = match index {
            8 => -3,
            7 | 10 | 11 => 0,
            _ => 1,
        };
        let mut nodes = Array1::with_bounds(lower, lower + values.len() as i32 - 1).unwrap();
        for (offset, value) in values.iter().enumerate() {
            *nodes.change_value(lower + offset as i32).unwrap() = *value;
        }
        for average in 0..2 {
            normal_row(rows, index, average, &nodes);
        }
    }
}
fn normal_row(rows: &mut String, index: usize, average: i32, nodes: &Array1<'_, f64>) {
    let mut normal = Vector3::new(7., 8., 9.);
    write!(rows, "normal {index} {average}").unwrap();
    let result = if average == 0 {
        T::get_normal(nodes, &mut normal)
    } else {
        T::get_average_normal(nodes, &mut normal)
    };
    match result {
        Ok(planar) => write!(rows, " {}", planar as i32).unwrap(),
        Err(cause) => write!(rows, " failure {}", cause.exception_type()).unwrap(),
    }
    real(rows, normal.x());
    real(rows, normal.y());
    real(rows, normal.z());
    rows.push('\n');
}
const NORMALS: &[&[f64]] = &[
    &[0., 0., 0., 1., 0., 0., 0., 1., 0.],
    &[0., 0., 0., 0., 1., 0., 1., 0., 0.],
    &[0., 0., 0., 1., 0., 0., 1., 1., 1., 0., 1., 0.],
    &[0., 0., 0., 1., 0., 0., 2., 0., 0.],
    &[0., 0., 0., 1., 0., 0.],
    &[0., 0., 0., 1., 0., 0., 0., 1., 0., 0., -1., 0.],
    &[0., 0., 0., 1e-4, 0., 0., 0., 1e-4, 0.],
    &[3., 0., 0., 0., 1., 0., 0., 0., 1., 0.],
    &[0., 0., 0., 1., 0., 0., 0., 1., 0.],
    &[0., 0., 0., 1., 0., 0., 0., 1., 0., 8., 9.],
    &[4., 0., 0., 0., 1., 0., 0., 0., 1., 0.],
    &[-1., 0., 0., 0.],
    &[0., 0., 0., 0., 0., 0., 1., 1., 1.],
];
