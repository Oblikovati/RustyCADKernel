use super::super::fixture;
use rustycad_tkbo::{bop_tools::algo_tools3d, int_tools::Context};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2, Point3};
#[test]
fn native_hatching_points_match() {
    for line in include_str!("../../../../tools/oracle/tkbo_point_in_face.expected").lines() {
        let expected: Vec<f64> = line
            .split_whitespace()
            .map(|token| token.parse().unwrap())
            .collect();
        let actual = observe(
            expected[0] as usize,
            expected[1] as usize,
            expected[2] as usize,
        );
        for (index, (actual, expected)) in actual.iter().zip(&expected[3..]).enumerate() {
            assert!(
                (actual - expected).abs() < 1e-12,
                "column {index}, {line}: actual={actual}"
            );
        }
    }
}
fn observe(kind: usize, orientation: usize, mode: usize) -> [f64; 6] {
    let mut face = fixture(kind).face;
    face.set_orientation(
        [
            Orientation::Forward,
            Orientation::Reversed,
            Orientation::Internal,
            Orientation::External,
        ][orientation],
    );
    let context = Context::default();
    let mut uv = Point2::new(9., 8.);
    let mut point = Point3::new(7., 6., 5.);
    let status = if mode == 0 {
        algo_tools3d::point_in_face(&face, &mut point, &mut uv, &context).unwrap()
    } else {
        let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::new(
            Point2::new(if mode == 3 { 100. } else { 0.5 }, 0.),
            Direction2::Y,
        )))
        .cast();
        algo_tools3d::point_in_face_curve(
            &face,
            &line,
            &mut point,
            &mut uv,
            &context,
            if mode == 2 { 0.1 } else { 0. },
        )
        .unwrap()
    };
    [
        f64::from(status),
        uv.x(),
        uv.y(),
        point.x(),
        point.y(),
        point.z(),
    ]
}
