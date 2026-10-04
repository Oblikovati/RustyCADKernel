use super::near_edge::prepare_fixture;
use rustycad_tkbo::{bop_tools::algo_tools3d, int_tools::Context};
use rustycad_tkmath::gp::{Direction3, Point3};
#[test]
fn native_approximate_normal_overloads_match() {
    for line in include_str!("../../../../tools/oracle/tkbo_approx_normal.expected").lines() {
        let row: Vec<f64> = line
            .split_whitespace()
            .map(|token| token.parse().unwrap())
            .collect();
        let actual = observe(&row);
        for (index, (actual, expected)) in actual.iter().zip(&row[5..]).enumerate() {
            assert!(
                (actual - expected).abs() < 1e-12,
                "column {index}, {line}: actual={actual}"
            );
        }
    }
}
fn observe(row: &[f64]) -> [f64; 7] {
    let (fixture, edge) = prepare_fixture(
        row[0] as usize,
        row[1] as usize,
        row[2] as usize,
        row[3] as usize,
    );
    let mut point = Point3::new(7., 6., 5.);
    let mut normal = Direction3::default();
    let context = Context::default();
    let success = match row[4] as usize {
        0 => algo_tools3d::approx_normal_with_step(
            &edge,
            &fixture.face,
            0.5,
            &mut point,
            &mut normal,
            1e-4,
        ),
        1 => algo_tools3d::approx_normal_on_edge(
            &edge,
            &fixture.face,
            0.5,
            &mut point,
            &mut normal,
            &context,
        ),
        _ => algo_tools3d::approx_normal_with_context_step(
            &edge,
            &fixture.face,
            0.5,
            1e-4,
            &mut point,
            &mut normal,
            &context,
        ),
    }
    .unwrap();
    [
        f64::from(success),
        point.x(),
        point.y(),
        point.z(),
        normal.x(),
        normal.y(),
        normal.z(),
    ]
}
