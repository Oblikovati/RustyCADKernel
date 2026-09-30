//! Non-manifold vertex copies own new representation records and retain unrelated geometry.
use super::*;
use rustycad_tkbrep::{
    brep::{
        BRepPointOnCurve, BRepPointOnCurveOnSurface, BRepPointOnSurface, BRepPointRepresentation,
        access::{edge_tshape, vertex_tshape},
        update_curves::as_gcurve,
    },
    topods::{self, Vertex},
};
use rustycad_tkg3d::{geom::GeomSurface, top_abs::Orientation};
mod edge;
mod face;
impl ShapeAnalysisTransferParametersProj {
    /// Copy an internal/external vertex onto another edge; `ShapeAnalysisTransferParametersProj::copy_nm_vertex_edge(&vertex,&target,&source)?`.
    pub fn copy_nm_vertex_edge(
        vertex: &Vertex,
        target: &Edge,
        source: &Edge,
    ) -> Result<Vertex, Failure> {
        if !is_non_manifold(vertex) {
            return Ok(Vertex::default());
        }
        edge::copy_vertex(vertex, target, source)
    }
    /// Copy an internal/external vertex onto another face; `ShapeAnalysisTransferParametersProj::copy_nm_vertex_face(&vertex,&target,&source)?`.
    pub fn copy_nm_vertex_face(
        vertex: &Vertex,
        target: &Face,
        source: &Face,
    ) -> Result<Vertex, Failure> {
        if !is_non_manifold(vertex) {
            return Ok(Vertex::default());
        }
        face::copy_vertex(vertex, target, source)
    }
}
fn is_non_manifold(vertex: &Vertex) -> bool {
    matches!(
        vertex.orientation(),
        Orientation::Internal | Orientation::External
    )
}
fn append_copy(
    vertex: &Vertex,
    representation: &Handle<dyn BRepPointRepresentation>,
) -> Result<(), Failure> {
    let Some(copy) = copy_representation(representation)? else {
        return Ok(());
    };
    vertex_tshape(vertex, "TransferParametersProj::CopyNMVertex")?
        .change_points(|points| points.push(copy));
    Ok(())
}
fn copy_representation(
    point: &Handle<dyn BRepPointRepresentation>,
) -> Result<Option<Handle<dyn BRepPointRepresentation>>, Failure> {
    if point.is_point_on_curve() {
        let copy = BRepPointOnCurve::new(point.parameter(), &point.curve()?, &point.location());
        return Ok(Some(Handle::new(copy).cast()));
    }
    if point.is_point_on_surface() {
        return Ok(Some(copy_surface_point(point)?));
    }
    if point.is_point_on_curve_on_surface() {
        return Ok(Some(copy_pcurve_point(point)?));
    }
    Ok(None)
}
fn copy_surface_point(
    point: &Handle<dyn BRepPointRepresentation>,
) -> Result<Handle<dyn BRepPointRepresentation>, Failure> {
    let copy = BRepPointOnSurface::new(
        point.parameter(),
        point.parameter2()?,
        &point.surface()?,
        &point.location(),
    );
    Ok(Handle::new(copy).cast())
}
fn copy_pcurve_point(
    point: &Handle<dyn BRepPointRepresentation>,
) -> Result<Handle<dyn BRepPointRepresentation>, Failure> {
    let copy = BRepPointOnCurveOnSurface::new(
        point.parameter(),
        &point.pcurve()?,
        &point.surface()?,
        &point.location(),
    );
    Ok(Handle::new(copy).cast())
}
