use super::Face;
use super::shift::shift_wire_pcurves;
use crate::shape_analysis::ShapeAnalysisSurface;
use crate::shape_build::ShapeBuildReShape;
use crate::shape_fix::ShapeFixFace;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Builder, Shape, ShapeIterator, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Vector2;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;

pub(super) fn build_with_natural_bounds(
    fixer: &ShapeFixFace,
    face: &Face,
    context: &Handle<ShapeBuildReShape>,
    surface_analysis: &ShapeAnalysisSurface,
    wires: Vec<Wire>,
    other_children: Vec<Shape>,
) -> Result<bool, Failure> {
    let (shift, translations) =
        super::selection::wire_translations(face, &wires, surface_analysis)?;
    shift_source_wires(face, &wires, &translations)?;
    let natural_face = make_natural_face(face)?;
    let natural_wires = face_wires(&natural_face)?;
    let mut result_wires = wires;
    result_wires.extend(natural_wires.iter().cloned());
    let shift = Vector2::new(shift.x(), shift.y());
    shift_natural_wires(face, &natural_wires, shift, surface_analysis)?;
    publish_natural_face(fixer, face, context, result_wires, other_children)
}

fn shift_source_wires(face: &Face, wires: &[Wire], shifts: &[Vector2]) -> Result<(), Failure> {
    for (wire, shift) in wires.iter().zip(shifts) {
        shift_wire_pcurves(face, wire, *shift, None)?;
    }
    Ok(())
}

fn make_natural_face(face: &Face) -> Result<Face, Failure> {
    let (surface, location) = BRepTool::surface_with_location(face)?;
    let mut natural =
        BRepBuilderAPIMakeFace::from_surface(&surface, precision::confusion())?.face()?;
    natural.set_location(&location, false)?;
    Ok(natural)
}

fn face_wires(face: &Face) -> Result<Vec<Wire>, Failure> {
    ShapeIterator::new(&face.clone().into_shape(), false, true)
        .filter(|child| child.shape_type().ok() == Some(rustycad_tkg3d::top_abs::ShapeEnum::Wire))
        .map(|child| topods::wire(&child))
        .collect()
}

fn shift_natural_wires(
    original_face: &Face,
    wires: &[Wire],
    shift: Vector2,
    surface: &ShapeAnalysisSurface,
) -> Result<(), Failure> {
    if shift.x().hypot(shift.y()) < precision::p_confusion() {
        return Ok(());
    }
    for wire in wires {
        shift_wire_pcurves(original_face, wire, shift, Some(surface))?;
    }
    Ok(())
}

fn publish_natural_face(
    fixer: &ShapeFixFace,
    old_face: &Face,
    context: &Handle<ShapeBuildReShape>,
    wires: Vec<Wire>,
    other_children: Vec<Shape>,
) -> Result<bool, Failure> {
    let mut result = old_face.empty_copied()?;
    result.set_orientation(Orientation::Forward);
    for wire in &wires {
        Builder.add(&mut result, &wire.clone().into_shape())?;
    }
    for child in &other_children {
        Builder.add(&mut result, child)?;
    }
    let mut result_face = topods::face(&result)?;
    orient_natural_result(fixer, &mut result_face);
    super::publish_rebuilt_face(fixer, old_face, context, &result_face)?;
    Ok(true)
}

fn orient_natural_result(fixer: &ShapeFixFace, result: &mut Face) {
    if !*fixer.forward.read() {
        result.set_orientation(Orientation::Reversed);
    }
}
