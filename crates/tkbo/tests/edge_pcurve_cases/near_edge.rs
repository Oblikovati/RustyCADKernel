use super::{edge_fixture, store_pcurve};
use rustycad_tkbo::bop_tools::algo_tools3d;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Point2, Point3};
#[test]
fn native_explicit_step_near_edge_matches() {
    for line in include_str!("../../../../tools/oracle/tkbo_point_near_edge.expected").lines() {
        let expected: Vec<f64> = line
            .split_whitespace()
            .map(|value| value.parse().unwrap())
            .collect();
        let actual = observe(
            expected[0] as usize,
            expected[1] as usize,
            expected[2] as usize,
            expected[3] as usize,
            None,
        );
        for (index, (actual, expected)) in actual.iter().zip(&expected[4..]).enumerate() {
            assert!(
                (actual - expected).abs() < 1e-12,
                "column {index}, {line}: actual={actual}"
            );
        }
    }
}
fn observe(
    kind: usize,
    edge_orientation: usize,
    face_orientation: usize,
    tolerance: usize,
    mode: Option<usize>,
) -> [f64; 6] {
    let (mut fixture, mut edge) = edge_fixture(if kind == 5 { 1 } else { kind });
    if kind != 5 {
        store_pcurve(&fixture, &edge);
    }
    let tolerance = [1e-7, 1e-5, 2e-5, 0.2, 5.][tolerance];
    let builder = BRepBuilder::default();
    builder.update_edge_tolerance(&edge, tolerance).unwrap();
    builder
        .update_face_tolerance(&fixture.face, 2. * tolerance)
        .unwrap();
    let orientations = [
        Orientation::Forward,
        Orientation::Reversed,
        Orientation::Internal,
        Orientation::External,
    ];
    edge.set_orientation(orientations[edge_orientation]);
    fixture.face.set_orientation(orientations[face_orientation]);
    let mut uv = Point2::new(9., 8.);
    let mut point = Point3::new(7., 6., 5.);
    let context = rustycad_tkbo::int_tools::Context::default();
    let status = match mode {
        None => {
            algo_tools3d::point_near_edge_step(&edge, &fixture.face, 0.5, 1e-4, &mut uv, &mut point)
        }
        Some(0) => algo_tools3d::point_near_edge_with_step(
            &edge,
            &fixture.face,
            0.5,
            1e-4,
            &mut uv,
            &mut point,
            &context,
        ),
        Some(1) => algo_tools3d::point_near_edge_at(
            &edge,
            &fixture.face,
            0.5,
            &mut uv,
            &mut point,
            &context,
        ),
        Some(2) => algo_tools3d::point_in_face_from_edge(
            &fixture.face,
            &edge,
            0.5,
            1e-4,
            &mut point,
            &mut uv,
            &context,
        ),
        Some(_) => {
            algo_tools3d::point_near_edge(&edge, &fixture.face, &mut uv, &mut point, &context)
        }
    }
    .unwrap();
    [
        f64::from(status),
        uv.x(),
        uv.y(),
        point.x(),
        point.y(),
        point.z(),
    ]
}

#[test]
fn native_context_near_edge_matches() {
    for line in
        include_str!("../../../../tools/oracle/tkbo_point_near_edge_context.expected").lines()
    {
        let expected: Vec<f64> = line
            .split_whitespace()
            .map(|value| value.parse().unwrap())
            .collect();
        let actual = observe(
            expected[0] as usize,
            expected[1] as usize,
            expected[2] as usize,
            expected[3] as usize,
            Some(expected[4] as usize),
        );
        for (index, (actual, expected)) in actual.iter().zip(&expected[5..]).enumerate() {
            assert!(
                (actual - expected).abs() < 1e-12,
                "column {index}, {line}: actual={actual}"
            );
        }
    }
}
