use super::super::ShapeFixWireframe;
use crate::shape_analysis::ShapeAnalysisTransferParametersProj;
use crate::shape_fix::FixRoot;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    brep_tools::ReShape,
    topods::{self, Edge, ShapeIterator},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;

pub(super) fn copy_pair_vertices(
    fixer: &ShapeFixWireframe,
    pair: &super::CandidatePair,
    target: &mut Edge,
) -> Result<(), Failure> {
    copy_source_vertices(fixer, &pair.first, target)?;
    copy_source_vertices(fixer, &pair.second, target)
}

fn copy_source_vertices(
    fixer: &ShapeFixWireframe,
    source: &Edge,
    target: &mut Edge,
) -> Result<(), Failure> {
    for child in ShapeIterator::new(&source.clone().into_shape(), false, false) {
        copy_non_manifold_vertex(fixer, source, target, &child)?;
    }
    Ok(())
}

fn copy_non_manifold_vertex(
    fixer: &ShapeFixWireframe,
    source: &Edge,
    target: &mut Edge,
    child: &rustycad_tkbrep::topods::Shape,
) -> Result<(), Failure> {
    if !is_non_manifold(child.orientation()) {
        return Ok(());
    }
    let vertex = topods::vertex(child)?;
    let copy = ShapeAnalysisTransferParametersProj::copy_nm_vertex_edge(&vertex, target, source)?;
    BRepBuilder::default().add(target, &copy)?;
    ReShape::replace(&*fixer.context(), &vertex, &copy)
}

fn is_non_manifold(orientation: Orientation) -> bool {
    matches!(orientation, Orientation::Internal | Orientation::External)
}
