//! Real registered meshing runs once; already complete meshes bypass the owned factory boundary.
#[allow(dead_code)] // Shared genuine topology fixture also supplies volume grouping matrices.
#[path = "support/textured_volume_replay.rs"]
mod volume_fixture;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::aspect::AspectTypeOfDeflection;
use rustycad_tkv3d::{prs3d::Prs3dDrawer, std_prs::StdPrsToolTriangulatedShape};

#[test]
fn textured_shape_real_factory_and_deflection_match_native_complete_mesh_skip() {
    for scenario in 0..4 {
        let shape = tessellate_shape(scenario);
        let drawer = tessellate_drawer(scenario);
        let values = tessellate_observations(&shape, &drawer);
        let prefix = format!("MARTENSITE_TESSELLATE {scenario} ");
        let native = include_str!("fixtures/ais_textured_shape/native-tessellate.txt");
        let expected: Vec<f64> = native
            .lines()
            .find_map(|line| line.strip_prefix(&prefix))
            .unwrap()
            .split_whitespace()
            .map(|value| value.parse().unwrap())
            .collect();
        assert_eq!(values, expected, "real native meshing case {scenario}");
    }
}
fn tessellate_shape(scenario: i32) -> Shape {
    if scenario == 2 {
        return Shape::default();
    }
    volume_fixture::textured_box_volume(scenario == 3, false).unwrap()
}
fn tessellate_drawer(scenario: i32) -> Handle<Prs3dDrawer> {
    let drawer = Handle::new(Prs3dDrawer::new());
    drawer.setup_own_defaults().unwrap();
    let mode = if scenario == 1 {
        AspectTypeOfDeflection::Relative
    } else {
        AspectTypeOfDeflection::Absolute
    };
    drawer.set_type_of_deflection(mode);
    drawer.set_maximal_chordial_deviation(0.1);
    drawer
}
fn tessellate_observations(shape: &Shape, drawer: &Handle<Prs3dDrawer>) -> Vec<f64> {
    let deviation = StdPrsToolTriangulatedShape::get_deflection(shape, drawer).unwrap();
    let before = StdPrsToolTriangulatedShape::is_tessellated(shape, drawer).unwrap();
    let first = StdPrsToolTriangulatedShape::tessellate(shape, drawer).unwrap();
    let after = StdPrsToolTriangulatedShape::is_tessellated(shape, drawer).unwrap();
    let second = StdPrsToolTriangulatedShape::tessellate(shape, drawer).unwrap();
    vec![
        deviation,
        drawer.maximal_chordial_deviation(),
        f64::from(i32::from(before)),
        f64::from(i32::from(first)),
        f64::from(i32::from(after)),
        f64::from(i32::from(second)),
    ]
}
