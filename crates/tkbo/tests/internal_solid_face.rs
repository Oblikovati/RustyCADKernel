//! Native internal-face classification with adjacency and solid fallback.
use rustycad_tkbo::{bop_tools, int_tools::Context};
use rustycad_tkbrep::{
    topexp::{self, Explorer},
    topods::{Face, Solid},
    toptools::IndexedDataMapOfShapeListOfShape,
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::{
    gp::{Transform3, Vector3},
    toploc::Location,
};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
fn orientation(index: usize) -> Orientation {
    [
        Orientation::Forward,
        Orientation::Reversed,
        Orientation::Internal,
        Orientation::External,
    ][index]
}
#[test]
fn native_internal_solid_face_classification_matches() {
    let fixture = include_str!("../../../tools/oracle/tkbo_internal_solid_face.expected");
    assert_eq!(fixture.lines().count(), 160);
    for line in fixture.lines() {
        let row: Vec<i32> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        assert_eq!(observe(&row), row[4..], "{line}");
    }
}
fn shapes(row: &[i32]) -> (Solid, Face) {
    let mut solid = BRepPrimAPIMakeBox::from_sizes(1., 2., 3.)
        .unwrap()
        .solid()
        .unwrap();
    solid.set_orientation(orientation(row[1] as usize));
    let mut face = Face::unchecked(
        Explorer::new(&solid, ShapeEnum::Face, ShapeEnum::Shape)
            .unwrap()
            .current()
            .clone(),
    );
    if row[0] != 0 {
        let mut transform = Transform3::default();
        transform.set_translation(Vector3::new(
            [0., 0.5, -0.5, 1., 2.][row[0] as usize],
            0.,
            0.,
        ));
        face.set_location(&Location::new(transform), true).unwrap();
    }
    face.set_orientation(orientation(row[2] as usize));
    (solid, face)
}
fn observe(row: &[i32]) -> Vec<i32> {
    let (solid, face) = shapes(row);
    let mut adjacent = IndexedDataMapOfShapeListOfShape::default();
    if row[3] != 0 {
        topexp::map_shapes_and_ancestors(&solid, ShapeEnum::Edge, ShapeEnum::Face, &mut adjacent)
            .unwrap();
    }
    let result =
        bop_tools::internal_face_in_solid(&face, &solid, &adjacent, 1e-7, &Context::default());
    vec![
        if result.is_ok() { 0 } else { -1 },
        i32::from(result.unwrap_or(false)),
        adjacent.extent() as i32,
    ]
}
