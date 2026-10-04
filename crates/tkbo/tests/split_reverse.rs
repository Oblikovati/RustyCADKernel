//! Pinned native split edge orientation and tangent output cases.
use rustycad_tkbo::{
    bop_tools::{self, algo_tools2d},
    int_tools::Context,
};
use rustycad_tkbrep::{brep::BRepBuilder, topods::Edge};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::{
    geom::{GeomCurve, GeomLine},
    top_abs::Orientation,
};
use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Vector3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
#[test]
fn native_split_orientation_rows_match() {
    for line in include_str!("../../../tools/oracle/tkbo_split_reverse.expected").lines() {
        let expected: Vec<f64> = line
            .split_whitespace()
            .map(|token| token.parse().unwrap())
            .collect();
        let actual = observe(
            expected[0] as usize,
            expected[1] as usize,
            expected[2] as usize,
        );
        assert_eq!(actual.as_slice(), &expected[3..], "{line}");
    }
}
fn observe(kind: usize, first: usize, second: usize) -> [f64; 6] {
    let (mut split, mut original) = edges(kind);
    let orientations = [
        Orientation::Forward,
        Orientation::Reversed,
        Orientation::Internal,
        Orientation::External,
    ];
    split.set_orientation(orientations[first]);
    original.set_orientation(orientations[second]);
    let mut error = 99;
    let reverse =
        bop_tools::is_split_edge_to_reverse(&split, &original, &Context::default(), &mut error)
            .unwrap();
    let mut tangent = Vector3::new(9., 8., 7.);
    let found = kind != 6 && algo_tools2d::edge_tangent(&split, 0.5, &mut tangent).unwrap();
    [
        f64::from(reverse),
        f64::from(error),
        f64::from(found),
        tangent.x(),
        tangent.y(),
        tangent.z(),
    ]
}
fn edges(kind: usize) -> (Edge, Edge) {
    let curve = line(Direction3::new(1., 0., 0.).unwrap());
    let other = if kind == 0 {
        curve.clone()
    } else {
        line(
            Direction3::new(
                if kind == 2 {
                    -1.
                } else if kind == 3 {
                    0.
                } else {
                    1.
                },
                if kind == 3 { 1. } else { 0. },
                0.,
            )
            .unwrap(),
        )
    };
    let mut split = BRepBuilderAPIMakeEdge::from_curve_params(&curve, 0., 1.)
        .unwrap()
        .edge()
        .unwrap();
    let mut original = BRepBuilderAPIMakeEdge::from_curve_params(&other, -2., 2.)
        .unwrap()
        .edge()
        .unwrap();
    let builder = BRepBuilder::default();
    if kind == 4 {
        builder.degenerated(&split, true).unwrap();
    }
    if kind == 5 {
        builder.degenerated(&original, true).unwrap();
    }
    if kind == 6 {
        builder.make_edge(&mut split).unwrap();
        builder.make_edge(&mut original).unwrap();
    }
    (split, original)
}
fn line(direction: Direction3) -> Handle<dyn GeomCurve> {
    Handle::new(GeomLine::new(Axis1::new(Point3::default(), direction))).cast()
}
