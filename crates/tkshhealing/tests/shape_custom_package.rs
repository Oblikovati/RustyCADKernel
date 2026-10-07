//! Behavior tests of the ShapeCustom package functions (the native comparison is the
//! sc_modif op 12/13 requests of tools/tkshhealing_corpus_shapecustom.py).
mod healing_support;

use healing_support::{compound, square_face};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::{BRepToolsModification, Modifier, ReShape};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkshhealing::shape_build::ShapeBuildReShape;
use rustycad_tkshhealing::shape_custom::{
    ShapeCustom, ShapeCustomContext, ShapeCustomDirectModification,
};

fn vertex_points(shape: &Shape) -> Vec<(f64, f64, f64)> {
    let mut explorer = Explorer::new(shape, ShapeEnum::Vertex, ShapeEnum::Shape).unwrap();
    let mut points = Vec::new();
    while explorer.more() {
        let p = BRepTool::pnt(&topods::vertex(explorer.current()).unwrap()).unwrap();
        points.push((p.x(), p.y(), p.z()));
        explorer.next().unwrap();
    }
    points
}

#[test]
fn scale_shape_scales_about_the_origin() {
    let face = square_face(1.).into_shape();
    let scaled = ShapeCustom::scale_shape(&face, 2.).unwrap();
    assert!(
        vertex_points(&scaled)
            .iter()
            .all(|&(_, _, z)| (z - 2.).abs() < 1e-12)
    );
}

#[test]
fn unchanged_compound_is_returned_as_is() {
    let face = square_face(0.).into_shape();
    let assembly = compound(&[face.clone(), face]).into_shape();
    let result = ShapeCustom::direct_faces(&assembly).unwrap();
    assert!(result.is_same(&assembly));
}

#[test]
fn apply_modifier_records_scaled_compound_in_context_and_reshape() {
    let face = square_face(0.).into_shape();
    let assembly = compound(&[face.clone(), face]).into_shape();
    let mut transform = rustycad_tkmath::gp::Transform3::default();
    transform
        .set_scale(rustycad_tkmath::gp::Point3::new(0., 0., 0.), 3.)
        .unwrap();
    let modification: Handle<dyn BRepToolsModification> = Handle::new(
        rustycad_tkshhealing::shape_custom::ShapeCustomTrsfModification::new(transform),
    )
    .cast();
    let mut context = ShapeCustomContext::default();
    let mut modifier = Modifier::new(false);
    let reshape = Handle::new(ShapeBuildReShape::new());
    let result = ShapeCustom::apply_modifier(
        &assembly,
        &modification,
        &mut context,
        &mut modifier,
        &ProgressRange::default(),
        &reshape,
    )
    .unwrap();
    assert!(!result.is_same(&assembly));
    assert_eq!(
        context.extent(),
        2,
        "the shared face once, the compound once"
    );
    let image = ReShape::apply(&*reshape, &assembly, ShapeEnum::Shape).unwrap();
    assert!(image.is_same(&result));
}

#[test]
fn null_shape_raises() {
    assert!(ShapeCustom::direct_faces(&Shape::default()).is_err());
    let _ = ShapeCustomDirectModification::new();
}
