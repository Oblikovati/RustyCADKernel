//! Standalone vector-angle regression checks from BRepMesh_Delaun_Test.cxx.
use super::super::{flag, real};
use rustycad_tkmath::gp::{Vector2, Xy};
pub fn angle_case(mode: i32) {
    let right = Vector2::new(1., 0.);
    let up = Vector2::new(0., 1.);
    if mode == 0 {
        real(up.angle(right).unwrap());
        real(right.angle(up).unwrap());
    } else {
        report_winding(mode == 1);
    }
    println!();
}
fn report_winding(counter_clockwise: bool) {
    let coordinates = winding_coordinates(counter_clockwise);
    let total = winding_angle(&coordinates);
    real(total);
    flag((2. * std::f64::consts::PI - total).abs() <= 1e-12);
    flag((total.abs() - 2. * std::f64::consts::PI).abs() <= 1e-12);
}
fn winding_coordinates(counter_clockwise: bool) -> [Xy; 5] {
    if counter_clockwise {
        [
            Xy::new(1., 0.),
            Xy::new(0., 1.),
            Xy::new(-1., 0.),
            Xy::new(0., -1.),
            Xy::new(1., 0.),
        ]
    } else {
        [
            Xy::new(1., 0.),
            Xy::new(0., -1.),
            Xy::new(-1., 0.),
            Xy::new(0., 1.),
            Xy::new(1., 0.),
        ]
    }
}
fn winding_angle(coordinates: &[Xy; 5]) -> f64 {
    let mut previous = Vector2::from(coordinates[0]);
    let mut total = 0.;
    for coordinate in &coordinates[1..] {
        let current = Vector2::from(*coordinate);
        total += current.angle(previous).unwrap();
        previous = current;
    }
    total
}
