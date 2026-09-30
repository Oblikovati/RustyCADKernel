// Rust translation of the six BRepGraph_Tool::Face::SurfaceAdaptor overloads
// (ModelingData/TKBRep/BRepGraph/BRepGraph_Tool.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The transformed surface adaptors of `BRepGraph_Tool::Face`, re-exported by
//! [`face`](super::face). The bounded overloads take `(u_first, u_last, v_first,
//! v_last)`.
use super::FaceUsage;
use super::face::{UvBounds, surface, usage};
use crate::brep_graph::{BRepGraph, FaceId, FaceRefId};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorTransformedSurface;
use rustycad_tkmath::gp::Transform3;

// The placement of a face usage: identity for an identity location.
fn usage_trsf(face_usage: &FaceUsage) -> Transform3 {
    if face_usage.location.is_identity() {
        Transform3::default()
    } else {
        face_usage.location.transformation()
    }
}

// The adaptor of a face's surface with `trsf`, over its bounds or `range`; an empty
// adaptor without a surface.
fn adaptor_of(
    graph: &BRepGraph,
    face: FaceId,
    range: Option<UvBounds>,
    trsf: impl FnOnce() -> Transform3,
) -> Result<GeomAdaptorTransformedSurface, Failure> {
    let surface = surface(graph, face);
    if surface.is_null() {
        return Ok(GeomAdaptorTransformedSurface::default());
    }
    match range {
        None => GeomAdaptorTransformedSurface::with_surface(&surface, trsf()),
        Some(range) => {
            GeomAdaptorTransformedSurface::with_range(&surface, (range, (0.0, 0.0)), trsf())
        }
    }
}

/// `SurfaceAdaptor(theGraph, theFace)`: the definition-frame adaptor.
///
/// ```
/// use rustycad_tkbrep::brep_graph::{BRepGraph, FaceRefId, tool};
/// let adaptor = tool::face::surface_adaptor_ref(&BRepGraph::new(), FaceRefId::new(0)).unwrap();
/// assert!(adaptor.geom_surface().is_null());
/// ```
pub fn surface_adaptor(
    graph: &BRepGraph,
    face: FaceId,
) -> Result<GeomAdaptorTransformedSurface, Failure> {
    adaptor_of(graph, face, None, Transform3::default)
}

/// `SurfaceAdaptor(theGraph, FaceUsage)`: with the usage placement.
pub fn surface_adaptor_usage(
    graph: &BRepGraph,
    face_usage: &FaceUsage,
) -> Result<GeomAdaptorTransformedSurface, Failure> {
    adaptor_of(graph, face_usage.def_id, None, || usage_trsf(face_usage))
}

/// `SurfaceAdaptor(theGraph, theFaceRef)`: with the reference's usage; empty for an
/// inactive reference or face.
pub fn surface_adaptor_ref(
    graph: &BRepGraph,
    face_ref: FaceRefId,
) -> Result<GeomAdaptorTransformedSurface, Failure> {
    let face_usage = usage(graph, face_ref);
    if !face_usage.is_valid() {
        return Ok(GeomAdaptorTransformedSurface::default());
    }
    surface_adaptor_usage(graph, &face_usage)
}

/// `SurfaceAdaptor(theGraph, theFace, UFirst, ULast, VFirst, VLast)`.
pub fn surface_adaptor_bounded(
    graph: &BRepGraph,
    face: FaceId,
    range: UvBounds,
) -> Result<GeomAdaptorTransformedSurface, Failure> {
    adaptor_of(graph, face, Some(range), Transform3::default)
}

/// `SurfaceAdaptor(theGraph, FaceUsage, UFirst, ULast, VFirst, VLast)`.
pub fn surface_adaptor_usage_bounded(
    graph: &BRepGraph,
    face_usage: &FaceUsage,
    range: UvBounds,
) -> Result<GeomAdaptorTransformedSurface, Failure> {
    adaptor_of(graph, face_usage.def_id, Some(range), || {
        usage_trsf(face_usage)
    })
}

/// `SurfaceAdaptor(theGraph, theFaceRef, UFirst, ULast, VFirst, VLast)`.
pub fn surface_adaptor_ref_bounded(
    graph: &BRepGraph,
    face_ref: FaceRefId,
    range: UvBounds,
) -> Result<GeomAdaptorTransformedSurface, Failure> {
    let face_usage = usage(graph, face_ref);
    if !face_usage.is_valid() {
        return Ok(GeomAdaptorTransformedSurface::default());
    }
    surface_adaptor_usage_bounded(graph, &face_usage, range)
}
