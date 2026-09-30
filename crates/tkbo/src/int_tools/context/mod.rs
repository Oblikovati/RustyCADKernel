// IntTools_Context.hxx/.cxx at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Shared geometric services cached by native shape identity (orientation ignored).
use super::{FClass2d, SurfaceRangeLocalizeData};
use cache::ShapeCache;
use rustycad_tkbrep::{
    brep::BRepTool,
    brep_adaptor::BRepAdaptorSurface,
    topods::{Edge, Face, Shape, Solid},
};
use rustycad_tkernel::{
    impl_transient,
    ncollection::BaseAllocator,
    standard::{Failure, Handle, SharedState, StandardTransient},
};
use rustycad_tkg3d::{
    adaptor3d::Adaptor3dSurface,
    geom::GeomCurve,
    top_abs::{Orientation, State},
};
use rustycad_tkgeomalgo::{
    geom_api::{ProjectPointOnCurve, ProjectPointOnSurf},
    geom2d_hatch::Hatcher,
};
use rustycad_tkmath::{
    bnd::{BoundingBox3, OrientedBoundingBox},
    gp::{Point2, Point3},
};
use rustycad_tktopalgo::{brep_bnd_lib::BRepBndLib, brep_class3d::BRepClass3dSolidClassifier};
mod cache;
mod faces;
mod hatching;
mod projectors;
mod services;
pub use cache::Cached;
/// `IntTools_Context`; `Context::default().state_point_face(&face, uv)?`.
pub struct Context {
    parent: StandardTransient,
    tolerance: SharedState<f64>,
    classifiers: ShapeCache<FClass2d>,
    surfaces: ShapeCache<BRepAdaptorSurface>,
    localization: ShapeCache<SurfaceRangeLocalizeData>,
    boxes: ShapeCache<BoundingBox3>,
    oriented_boxes: ShapeCache<OrientedBoundingBox>,
    solids: ShapeCache<BRepClass3dSolidClassifier>,
    hatchers: ShapeCache<Hatcher>,
    surface_projectors: ShapeCache<ProjectPointOnSurf>,
    edge_projectors: ShapeCache<ProjectPointOnCurve>,
    curve_projectors: projectors::CurveCache,
}
impl_transient!(Context, "IntTools_Context", StandardTransient, parent);
impl Default for Context {
    fn default() -> Self {
        Self::with_allocator(BaseAllocator::common())
    }
}
impl Context {
    /// Uses the injected allocator for cached maps and services; `Context::with_allocator(BaseAllocator::common())`.
    pub fn with_allocator(allocator: BaseAllocator) -> Self {
        Self {
            parent: StandardTransient::default(),
            tolerance: SharedState::new(1e-12),
            classifiers: ShapeCache::new(&allocator),
            surfaces: ShapeCache::new(&allocator),
            localization: ShapeCache::new(&allocator),
            boxes: ShapeCache::new(&allocator),
            oriented_boxes: ShapeCache::new(&allocator),
            solids: ShapeCache::new(&allocator),
            hatchers: ShapeCache::new(&allocator),
            surface_projectors: ShapeCache::new(&allocator),
            edge_projectors: ShapeCache::new(&allocator),
            curve_projectors: projectors::CurveCache::new(&allocator),
        }
    }
}
mod blocks;
mod compute;
mod vertex_line;
