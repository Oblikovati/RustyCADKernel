use super::correct_point_on_curve;
use rustycad_tkbrep::{
    topexp::Explorer,
    topods::{Edge, Face, Shape},
    toptools::IndexedMapOfShape,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
mod edge;
mod plane;
mod reference;
mod wire_intersection;
mod wires;
/// Correct wire vertex discrepancies before validating edge pcurves; `correct_curve_on_surface(&shape,&avoided,0.1,false)?`.
pub fn correct_curve_on_surface(
    shape: &Shape,
    avoided: &IndexedMapOfShape,
    maximum: f64,
    _run_parallel: bool,
) -> Result<(), Failure> {
    let faces: Vec<Face> = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)?
        .map(Face::unchecked)
        .collect();
    let mut pairs = Vec::new();
    for face in &faces {
        for edge in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
            pairs.push((Edge::unchecked(edge), face.clone()));
        }
    }
    for face in &faces {
        wires::correct_wires(face, avoided)?;
    }
    for (edge, face) in pairs {
        edge::correct_edge(&edge, &face, maximum, avoided)?;
    }
    Ok(())
}
/// Apply native point-on-curve then curve-on-surface correction; `correct_tolerances(&shape,&avoided,0.1,false)?`.
pub fn correct_tolerances(
    shape: &Shape,
    avoided: &IndexedMapOfShape,
    maximum: f64,
    run_parallel: bool,
) -> Result<(), Failure> {
    correct_point_on_curve(shape, avoided, maximum, run_parallel)?;
    correct_curve_on_surface(shape, avoided, maximum, run_parallel)
}
