// Rust translation of `ensureTriangulationRep`, `ensurePolygon3DRep`,
// `ensurePolygon2DRep` and `ensurePolygonOnTriRep` of
// ModelingData/TKBRep/BRepGraph/BRepGraph_Copy.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The persistent mesh representations of a copied node, written through the editor's
//! `SetPersistent*` setters under the mesh policy.
use super::context::CopyContext;
use super::policy::{MeshPolicy, copy_mesh};
use crate::brep_graph::{
    CoEdgeId, CoEdgePolygon2DRepId, CoEdgePolygonOnTriRepId, EdgeId, EdgePolygon3DRepId, FaceId,
    FaceTriangulationRepId, RepTag, TypedId,
};
use rustycad_tkernel::standard::Failure;

// The shared guard of the four `ensure*Rep`: no copy under `MeshPolicy::Drop`, for an
// id past the source store or for a removed source record.
fn is_copyable<T: RepTag>(context: &CopyContext<'_>, rep: TypedId<T>) -> bool {
    let storage = context.src_storage();
    context.mesh != MeshPolicy::Drop
        && rep.is_valid_for(storage.nb::<T>())
        && !storage.is_removed(rep)
}

/// `ensureTriangulationRep(theCtx, theSrcRepId, theDstFaceId)`.
pub(crate) fn ensure_triangulation_rep(
    context: &mut CopyContext<'_>,
    rep: FaceTriangulationRepId,
    face: FaceId,
) -> Result<(), Failure> {
    if !is_copyable(context, rep) {
        return Ok(());
    }
    let source = &context
        .src_storage()
        .face_triangulation_rep(rep)
        .triangulation;
    let copied = copy_mesh(source, context.mesh)?;
    context
        .target_mut()
        .editor()
        .faces()
        .set_persistent_triangulation(face, &copied)
}

/// `ensurePolygon3DRep(theCtx, theSrcRepId, theDstEdgeId)`.
pub(crate) fn ensure_polygon3d_rep(
    context: &mut CopyContext<'_>,
    rep: EdgePolygon3DRepId,
    edge: EdgeId,
) -> Result<(), Failure> {
    if !is_copyable(context, rep) {
        return Ok(());
    }
    let copied = copy_mesh(
        &context.src_storage().edge_polygon3d_rep(rep).polygon,
        context.mesh,
    )?;
    context
        .target_mut()
        .editor()
        .edges()
        .set_persistent_polygon3d(edge, &copied)
}

/// `ensurePolygon2DRep(theCtx, theSrcRepId, theDstCoEdgeId)`.
pub(crate) fn ensure_polygon2d_rep(
    context: &mut CopyContext<'_>,
    rep: CoEdgePolygon2DRepId,
    co_edge: CoEdgeId,
) -> Result<(), Failure> {
    if !is_copyable(context, rep) {
        return Ok(());
    }
    let copied = copy_mesh(
        &context.src_storage().co_edge_polygon2d_rep(rep).polygon,
        context.mesh,
    )?;
    context
        .target_mut()
        .editor()
        .co_edges()
        .set_persistent_polygon2d(co_edge, &copied)
}

/// `ensurePolygonOnTriRep(theCtx, theSrcRepId, theDstCoEdgeId)`.
pub(crate) fn ensure_polygon_on_tri_rep(
    context: &mut CopyContext<'_>,
    rep: CoEdgePolygonOnTriRepId,
    co_edge: CoEdgeId,
) -> Result<(), Failure> {
    if !is_copyable(context, rep) {
        return Ok(());
    }
    let copied = copy_mesh(
        &context
            .src_storage()
            .co_edge_polygon_on_tri_rep(rep)
            .polygon,
        context.mesh,
    )?;
    context
        .target_mut()
        .editor()
        .co_edges()
        .set_persistent_polygon_on_tri(co_edge, &copied)
}
