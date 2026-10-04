use rustycad_tkbrep::{
    brep::{BRepCurveRepresentation, BRepTEdge, BRepTFace},
    topods::{Shape, ShapeIterator},
    toptools::MapOfShape,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
/// Search topology for geometric or polygonal support; `is_empty_shape(&shape)?`.
pub fn is_empty_shape(shape: &Shape) -> Result<bool, Failure> {
    Ok(!contains_geometry(shape, &mut MapOfShape::default())?)
}
fn contains_geometry(shape: &Shape, visited: &mut MapOfShape) -> Result<bool, Failure> {
    if shape.is_null() || visited.contains(shape) {
        return Ok(false);
    }
    if shape_geometry(shape)? {
        return Ok(true);
    }
    for child in ShapeIterator::new(shape, false, false) {
        if contains_geometry(&child, visited)? {
            return Ok(true);
        }
        // Native records the parent only after visiting each child.
        visited.add(shape.clone());
    }
    Ok(false)
}
fn shape_geometry(shape: &Shape) -> Result<bool, Failure> {
    match shape.shape_type()? {
        ShapeEnum::Vertex => Ok(true),
        ShapeEnum::Edge => edge_geometry(shape),
        ShapeEnum::Face => face_geometry(shape),
        _ => Ok(false),
    }
}
fn edge_geometry(shape: &Shape) -> Result<bool, Failure> {
    let edge: Handle<BRepTEdge> = shape.tshape().cast();
    let edge = edge.get().ok_or_else(|| missing_representation("edge"))?;
    for curve in edge.curves() {
        if curve_geometry(&*curve)? {
            return Ok(true);
        }
    }
    Ok(false)
}
fn curve_geometry(curve: &dyn BRepCurveRepresentation) -> Result<bool, Failure> {
    if curve.is_curve3d() {
        return Ok(!curve.curve3d()?.is_null());
    }
    if curve.is_curve_on_surface() || curve.is_regularity() {
        return Ok(true);
    }
    // Preserve native Polygon3D dispatch before the other polygon predicates, including errors.
    if !curve.polygon3d()?.is_null() {
        return Ok(true);
    }
    Ok(curve.is_polygon_on_triangulation() || curve.is_polygon_on_surface())
}
fn face_geometry(shape: &Shape) -> Result<bool, Failure> {
    let face: Handle<BRepTFace> = shape.tshape().cast();
    let face = face.get().ok_or_else(|| missing_representation("face"))?;
    Ok(!face.surface().is_null() || !face.active_triangulation().is_null())
}
fn missing_representation(kind: &str) -> Failure {
    Failure::with_message(
        "Standard_TypeMismatch",
        &format!("{kind} TShape has incompatible type; expected a BRep representation"),
    )
}
