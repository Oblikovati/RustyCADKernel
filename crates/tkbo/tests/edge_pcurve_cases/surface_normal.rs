use super::super::fixture;
use rustycad_tkbo::bop_tools::algo_tools3d;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkernel::{ncollection::Array2, standard::Handle};
use rustycad_tkg3d::geom::{GeomBezierSurface, GeomSurface};
use rustycad_tkmath::gp::{Direction3, Point3};
#[test]
fn native_surface_normals_and_sense_thresholds_match() {
    for line in include_str!("../../../../tools/oracle/tkbo_surface_normal.expected").lines() {
        let (label, values) = line.split_once(' ').unwrap();
        let values: Vec<f64> = values
            .split_whitespace()
            .map(|token| token.parse().unwrap())
            .collect();
        if label == "sense" {
            assert_eq!(
                algo_tools3d::sense_flag(
                    direction(values[0] as usize),
                    direction(values[1] as usize)
                ),
                values[2] as i32
            );
            continue;
        }
        let actual = observe_normal(values[0] as usize);
        for (actual, expected) in actual.iter().zip(&values[1..]) {
            assert!((actual - expected).abs() < 1e-12, "{line}: actual={actual}");
        }
    }
}
fn direction(index: usize) -> Direction3 {
    let pi = std::f64::consts::PI;
    let angle: f64 = [0., 0.000199, 0.000201, 1., pi, pi - 0.000199, pi - 0.000201][index];
    Direction3::new(angle.cos(), angle.sin(), 0.).unwrap()
}
fn observe_normal(kind: usize) -> [f64; 4] {
    let surface = surface(kind);
    let mut normal = Direction3::new(0., 1., 0.).unwrap();
    let status = match algo_tools3d::get_normal_to_surface(&surface, 0.3, 0.2, &mut normal) {
        Ok(success) => f64::from(success),
        Err(error) => {
            assert_eq!(error.exception_type(), "Standard_ConstructionError");
            -1.
        }
    };
    [status, normal.x(), normal.y(), normal.z()]
}
fn surface(kind: usize) -> Handle<dyn GeomSurface> {
    if kind < 5 {
        return BRepTool::surface(&fixture(kind).face).unwrap();
    }
    let mut poles = Array2::with_bounds(1, 2, 1, 2).unwrap();
    for u in 1..=2 {
        for v in 1..=2 {
            let point = match kind {
                5 => Point3::default(),
                6 => Point3::new(f64::from(u) * 1e-9, f64::from(v) * 1e-9, 0.),
                _ => Point3::new(f64::from(u + v), 0., 0.),
            };
            poles.set_value(u, v, point).unwrap();
        }
    }
    Handle::new(GeomBezierSurface::new(&poles).unwrap()).cast()
}
