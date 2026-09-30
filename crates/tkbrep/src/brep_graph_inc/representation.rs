// Rust translation of ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_Representation.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The geometry and mesh use records: each is owned by one edge, coedge or face, and a
//! curve use carries the parameter range of that use.
use crate::brep_graph::{CoEdgeId, EdgeId, FaceId};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkmath::poly::{Polygon2D, Polygon3D, PolygonOnTriangulation, Triangulation};

/// `BRepGraphInc::EdgeCurve3DRep`: the 3D curve use of an edge.
#[derive(Clone, Default)]
pub struct EdgeCurve3DRep {
    /// `ParentEdgeId`.
    pub parent_edge_id: EdgeId,
    /// `Curve`.
    pub curve: Handle<dyn GeomCurve>,
    /// `ParamFirst`.
    pub param_first: f64,
    /// `ParamLast`.
    pub param_last: f64,
}

/// `BRepGraphInc::EdgePolygon3DRep`: the 3D polygon use of an edge.
#[derive(Clone, Default)]
pub struct EdgePolygon3DRep {
    /// `ParentEdgeId`.
    pub parent_edge_id: EdgeId,
    /// `Polygon`.
    pub polygon: Handle<Polygon3D>,
}

/// `BRepGraphInc::CoEdgeCurve2DRep`: the p-curve use of a coedge.
#[derive(Clone, Default)]
pub struct CoEdgeCurve2DRep {
    /// `ParentCoEdgeId`.
    pub parent_co_edge_id: CoEdgeId,
    /// `Curve`.
    pub curve: Handle<dyn Geom2dCurve>,
    /// `ParamFirst`.
    pub param_first: f64,
    /// `ParamLast`.
    pub param_last: f64,
}

/// `BRepGraphInc::CoEdgePolygon2DRep`: the polygon-on-surface use of a coedge.
#[derive(Clone, Default)]
pub struct CoEdgePolygon2DRep {
    /// `ParentCoEdgeId`.
    pub parent_co_edge_id: CoEdgeId,
    /// `Polygon`.
    pub polygon: Handle<Polygon2D>,
}

/// `BRepGraphInc::CoEdgePolygonOnTriRep`: the polygon-on-triangulation use of a coedge.
#[derive(Clone, Default)]
pub struct CoEdgePolygonOnTriRep {
    /// `ParentCoEdgeId`.
    pub parent_co_edge_id: CoEdgeId,
    /// `Polygon`.
    pub polygon: Handle<PolygonOnTriangulation>,
}

/// `BRepGraphInc::FaceSurfaceRep`: the surface use of a face.
#[derive(Clone, Default)]
pub struct FaceSurfaceRep {
    /// `ParentFaceId`.
    pub parent_face_id: FaceId,
    /// `Surface`.
    pub surface: Handle<dyn GeomSurface>,
}

/// `BRepGraphInc::FaceTriangulationRep`: the triangulation use of a face.
#[derive(Clone, Default)]
pub struct FaceTriangulationRep {
    /// `ParentFaceId`.
    pub parent_face_id: FaceId,
    /// `Triangulation`.
    pub triangulation: Handle<Triangulation>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_default_to_null_handles() {
        assert!(EdgeCurve3DRep::default().curve.is_null());
        assert!(FaceSurfaceRep::default().surface.is_null());
        assert!(!CoEdgeCurve2DRep::default().parent_co_edge_id.is_valid());
        assert!(FaceTriangulationRep::default().triangulation.is_null());
        assert!(EdgePolygon3DRep::default().polygon.is_null());
        assert!(CoEdgePolygon2DRep::default().polygon.is_null());
        assert!(CoEdgePolygonOnTriRep::default().polygon.is_null());
    }
}
