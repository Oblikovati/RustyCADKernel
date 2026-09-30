//! Original OCC24923 declaration, same seeded inputs, retries and one-percent bound.
#![allow(non_snake_case)]
use rustycad_tkmath::gp::Xy;
use rustycad_tkmesh::brep_mesh::BRepMeshCircleTool;
#[path = "support/circle_random.rs"]
mod fixture;
#[test]
fn BRepMesh_CircleTool_Test__OCC24923_CircumCirclePassesThroughAllVertices() {
    let failures = count_original_failures();
    let failure_rate = f64::from(failures) / 100000.;
    assert!(
        failure_rate <= 0.01,
        "Too many bad circumscribed circles: {failures} / 100000"
    );
}

fn count_original_failures() -> i32 {
    let mut random = fixture::CircleRandom::seeded();
    let mut failures = 0;
    for _ in 0..100000 {
        let points = random.accepted_triangle();
        let mut center = Xy::default();
        let mut radius = 0.;
        if !BRepMeshCircleTool::make_circle(&points, &mut center, &mut radius) {
            continue;
        }
        if points
            .iter()
            .any(|point| !fixture::on_circle(*point, center, radius))
        {
            failures += 1;
        }
    }
    failures
}
