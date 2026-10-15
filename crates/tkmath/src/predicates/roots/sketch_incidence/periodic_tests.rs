// SPDX-License-Identifier: GPL-2.0-only
//! The periodic unwrap equals the ported `BSplCLib::Unperiodize` form.
use super::input::{SketchIncidenceSpline, SketchPeriodicSpline};
use super::periodic::unwrap;
use crate::bsplclib;
use rustycad_tkernel::ncollection::Array1;

fn hexagon(multiplicities: Vec<usize>, knots: Vec<f64>, poles: usize) -> SketchPeriodicSpline {
    SketchPeriodicSpline {
        degree: 3,
        poles: (0..poles).map(|i| [i as f64, (i * i % 5) as f64]).collect(),
        weights: Some((0..poles).map(|i| 1. + i as f64 / 4.).collect()),
        knots,
        multiplicities,
        bounds: [0., 3.],
    }
}
type Unperiodized = (
    Array1<'static, i32>,
    Array1<'static, f64>,
    Array1<'static, f64>,
);

// Unperiodize on interleaved (x, y, w) values: poles are only copied cyclically.
fn unperiodize(periodic: &SketchPeriodicSpline) -> Unperiodized {
    let mults: Vec<i32> = periodic.multiplicities.iter().map(|&m| m as i32).collect();
    let mults = Array1::from_shared(&mults, 1);
    let degree = periodic.degree as i32;
    let (nb_knots, nb_poles) = bsplclib::prepare_unperiodize(degree, &mults).unwrap();
    let weights = periodic.weights.as_ref().unwrap();
    let flat: Vec<f64> = (0..periodic.poles.len())
        .flat_map(|i| [periodic.poles[i][0], periodic.poles[i][1], weights[i]])
        .collect();
    let mut new_mults = Array1::with_bounds(1, nb_knots).unwrap();
    let mut new_knots = Array1::with_bounds(1, nb_knots).unwrap();
    let mut new_poles = Array1::with_bounds(1, 3 * nb_poles).unwrap();
    let (knots, flat) = (
        Array1::from_shared(&periodic.knots, 1),
        Array1::from_shared(&flat, 1),
    );
    let outputs = (&mut new_mults, &mut new_knots, &mut new_poles);
    bsplclib::unperiodize_flat(
        degree, 3, &mults, &knots, &flat, outputs.0, outputs.1, outputs.2,
    )
    .unwrap();
    (new_mults, new_knots, new_poles)
}
// The non-periodic flat knot sequence of the unperiodized knots.
fn unperiodized(periodic: &SketchPeriodicSpline) -> SketchIncidenceSpline {
    let (mults, knots, poles) = unperiodize(periodic);
    let degree = periodic.degree as i32;
    let length = bsplclib::knot_sequence_length(&mults, degree, false).unwrap();
    let mut sequence = Array1::with_bounds(1, length).unwrap();
    bsplclib::knot_sequence_with_degree(&knots, &mults, degree, false, &mut sequence).unwrap();
    let values = poles.as_slice();
    SketchIncidenceSpline {
        degree: periodic.degree,
        poles: values.chunks(3).map(|p| [p[0], p[1]]).collect(),
        weights: Some(values.chunks(3).map(|p| p[2]).collect()),
        flat_knots: sequence.as_slice().to_vec(),
        bounds: periodic.bounds,
    }
}
#[test]
fn unwrap_equals_unperiodize_for_simple_and_multiple_knots() {
    let cases = [
        hexagon(vec![1; 4], vec![0., 1., 2., 3.], 3),
        hexagon(vec![2, 1, 2, 2], vec![0., 1., 2., 3.], 5),
        hexagon(vec![3, 2, 3], vec![0., 1.5, 3.], 5),
    ];
    for periodic in cases {
        assert_eq!(
            unwrap(&periodic).unwrap(),
            unperiodized(&periodic),
            "{periodic:?}"
        );
    }
}
#[test]
fn unwrap_reproduces_the_hand_derived_flat_knots() {
    let periodic = hexagon(vec![2, 1, 2, 2], vec![0., 1., 2., 3.], 5);
    let spline = unwrap(&periodic).unwrap();
    assert_eq!(
        spline.flat_knots,
        [-1., -1., 0., 0., 1., 2., 2., 3., 3., 4., 5.]
    );
    let wrapped: Vec<[f64; 2]> = [0, 1, 2, 3, 4, 0, 1]
        .iter()
        .map(|&i| periodic.poles[i])
        .collect();
    assert_eq!(spline.poles, wrapped);
}
