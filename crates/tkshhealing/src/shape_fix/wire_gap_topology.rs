//! Shared native vertex and nonmanifold topology copying for 2D/3D gap repair.
use super::{FixRoot, ShapeFixWire};
use crate::{
    shape_analysis::ShapeAnalysisTransferParametersProj, shape_fix::ShapeFixShapeTolerance,
};
use rustycad_tkbrep::{
    brep::BRepBuilder,
    brep_tools::ReShape,
    topods::{self, Edge, ShapeIterator, Vertex},
};
use rustycad_tkernel::{precision, standard::Failure};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
pub(super) fn copy_gap_vertex(tool: &ShapeFixWire, vertex: &Vertex) -> Result<Vertex, Failure> {
    let copy = topods::vertex(&vertex.empty_copied()?)?;
    ShapeFixShapeTolerance.set_tolerance(&copy, precision::confusion(), ShapeEnum::Shape)?;
    ReShape::replace(&*tool.context(), vertex, &copy)?;
    Ok(copy)
}
pub(super) fn preserve_gap_nonmanifold(
    tool: &ShapeFixWire,
    source: &Edge,
    target: &mut Edge,
) -> Result<(), Failure> {
    for shape in ShapeIterator::new(source, false, true) {
        if !matches!(
            shape.orientation(),
            Orientation::Internal | Orientation::External
        ) {
            continue;
        }
        let vertex = topods::vertex(&shape)?;
        let copy =
            ShapeAnalysisTransferParametersProj::copy_nm_vertex_edge(&vertex, target, source)?;
        BRepBuilder::default().add(target, &copy)?;
        ReShape::replace(&*tool.context(), &vertex, &copy)?;
    }
    Ok(())
}
