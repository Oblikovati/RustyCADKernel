//! Native cleaning guard must preserve IEEE unordered comparisons, not algebraic De Morgan rewrites.
#[path = "support/textured_face_fixture.rs"]
mod face_fixture;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::poly::mesh_purpose;
use rustycad_tkv3d::{prs3d::Prs3dDrawer, std_prs::StdPrsToolTriangulatedShape};

#[test]
fn textured_compute_deflection_cleaning_preserves_native_infinite_previous_values() {
    for scenario in 0..5 {
        let face = face_fixture::textured_face_fixture(1).unwrap();
        let alias = face.clone();
        let drawer = deviation_drawer(scenario);
        StdPrsToolTriangulatedShape::clear_on_own_deflection_change(&face, &drawer, true).unwrap();
        let present = !BRepTool::triangulation(&alias, mesh_purpose::NONE)
            .unwrap()
            .0
            .is_null();
        let actual = format!(
            "MARTENSITE_DEVIATION_CLEAR {scenario} {}",
            i32::from(present)
        );
        assert!(
            include_str!("fixtures/ais_textured_shape/native-deviation-clear.txt")
                .contains(&actual)
        );
    }
}
fn deviation_drawer(scenario: i32) -> Handle<Prs3dDrawer> {
    let drawer = Handle::new(Prs3dDrawer::new());
    drawer.setup_own_defaults().unwrap();
    let value = if scenario < 2 { 0.75 } else { f64::INFINITY };
    if scenario & 1 != 0 {
        drawer.set_deviation_coefficient(value);
    } else {
        drawer.set_deviation_angle(value);
    }
    if scenario >= 2 {
        drawer.update_previous_deviation_angle();
        drawer.update_previous_deviation_coefficient();
    }
    drawer.set_auto_triangulation(scenario != 4);
    drawer
}
