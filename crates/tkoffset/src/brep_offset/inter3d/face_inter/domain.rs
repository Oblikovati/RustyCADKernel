//! Validates that all bounded planar domains have straight p-curve boundaries.
use super::not_implemented;
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp::Explorer,
    topods::{self, Face, Shape},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkg3d::top_abs::ShapeEnum;

pub(super) fn validate_face_boundaries(face: &Face) -> Result<(), Failure> {
    let face_shape: Shape = face.clone().into();
    let edges = descendants(&face_shape, ShapeEnum::Edge)?;
    for shape in &edges {
        let edge = topods::edge(shape)?;
        let pcurve = BRepTool::curve_on_face(&edge, face)?;
        if !is_line_pcurve(&pcurve.pcurve) {
            return Err(not_implemented(
                "curved face boundary; expected line pcurves",
            ));
        }
    }
    Ok(())
}

fn is_line_pcurve(curve: &rustycad_tkernel::standard::Handle<dyn Geom2dCurve>) -> bool {
    if !curve.cast::<Geom2dLine>().is_null() {
        return true;
    }
    let trimmed = curve.cast::<Geom2dTrimmedCurve>();
    trimmed
        .get()
        .is_some_and(|trimmed| !trimmed.basis_curve().cast::<Geom2dLine>().is_null())
}

fn descendants(shape: &Shape, kind: ShapeEnum) -> Result<Vec<Shape>, Failure> {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape)?;
    let mut shapes = Vec::new();
    while explorer.more() {
        shapes.push(explorer.current().clone());
        explorer.next()?;
    }
    Ok(shapes)
}
