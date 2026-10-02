//! Direct Rust mirrors for the standalone angle and winding declarations.
#![allow(non_snake_case)]
use rustycad_tkmath::gp::{Vector2, Xy};
fn winding_angle(points: &[Xy; 5]) -> f64 {
    let mut previous = Vector2::from(points[0]);
    let mut total = 0.;
    for point in &points[1..] {
        let current = Vector2::from(*point);
        total += current.angle(previous).unwrap();
        previous = current;
    }
    total
}
#[test]
fn BRepMesh_DelaunTest__Vec2dAngleSignConvention() {
    let right = Vector2::new(1., 0.);
    let up = Vector2::new(0., 1.);
    let ccw = up.angle(right).unwrap();
    let cw = right.angle(up).unwrap();
    assert!(ccw < 0.);
    assert!((ccw + std::f64::consts::FRAC_PI_2).abs() <= 1e-12);
    assert!(cw > 0.);
    assert!((cw - std::f64::consts::FRAC_PI_2).abs() <= 1e-12);
}
#[test]
fn BRepMesh_DelaunTest__WindingAngleCCWPolygon() {
    let points = [
        Xy::new(1., 0.),
        Xy::new(0., 1.),
        Xy::new(-1., 0.),
        Xy::new(0., -1.),
        Xy::new(1., 0.),
    ];
    let total = winding_angle(&points);
    assert!((total.abs() - 2. * std::f64::consts::PI).abs() <= 1e-12);
    assert!((2. * std::f64::consts::PI - total).abs() > 1e-12);
    assert!((total.abs() - 2. * std::f64::consts::PI).abs() <= 1e-12);
}
#[test]
fn BRepMesh_DelaunTest__WindingAngleCWPolygon() {
    let points = [
        Xy::new(1., 0.),
        Xy::new(0., -1.),
        Xy::new(-1., 0.),
        Xy::new(0., 1.),
        Xy::new(1., 0.),
    ];
    let total = winding_angle(&points);
    assert!((total - 2. * std::f64::consts::PI).abs() <= 1e-12);
    assert!((2. * std::f64::consts::PI - total).abs() <= 1e-12);
    assert!((total.abs() - 2. * std::f64::consts::PI).abs() <= 1e-12);
}
