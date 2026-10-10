//! Triangular selecting volume and its shared native state (OCCT 3d097a0).
use super::{
    SelectMgrBaseFrustum, SelectMgrBaseFrustumBase, SelectMgrBaseIntersector, SelectMgrFrustumCore,
};
use rustycad_tkernel::standard::{SharedState, StandardTransient};
use rustycad_tkmath::gp::Point2;

/// The three input pixels used when rebuilding a triangular selecting volume.
#[derive(Clone, Copy, Debug, Default)]
pub struct SelectMgrSelectionTriangle {
    /// Native `SelectionTriangle::Points`; `triangle.points[0] = pixel`.
    pub points: [Point2; 3],
}

/// One triangular part of a polyline selector, composed with the native frustum state.
pub struct SelectMgrTriangularFrustum {
    transient: StandardTransient,
    /// Inherited camera, builder and tolerance; `volume.base.window_size()`.
    pub base: SelectMgrBaseFrustumBase,
    /// Protected template geometry; `volume.frustum_core.read().vertices()`.
    pub frustum_core: SharedState<SelectMgrFrustumCore<3>>,
    /// Native `mySelTriangle`; `volume.selection_triangle.read().points`.
    pub selection_triangle: SharedState<SelectMgrSelectionTriangle>,
}
rustycad_tkernel::impl_transient!(SelectMgrTriangularFrustum, "SelectMgr_TriangularFrustum",
    dyn SelectMgrBaseFrustum, transient, interfaces: [dyn SelectMgrBaseFrustum, dyn SelectMgrBaseIntersector]);

impl Default for SelectMgrTriangularFrustum {
    fn default() -> Self {
        Self::new()
    }
}
impl Drop for SelectMgrTriangularFrustum {
    fn drop(&mut self) {
        self.clear();
    }
}
impl SelectMgrBaseFrustum for SelectMgrTriangularFrustum {
    fn frustum_base(&self) -> &SelectMgrBaseFrustumBase {
        &self.base
    }
}
impl SelectMgrTriangularFrustum {
    /// Create the empty volume used by a frustum set; `SelectMgrTriangularFrustum::new()`.
    pub fn new() -> Self {
        Self {
            transient: StandardTransient::default(),
            base: SelectMgrBaseFrustumBase::default(),
            frustum_core: SharedState::new(SelectMgrFrustumCore::new()),
            selection_triangle: SharedState::new(SelectMgrSelectionTriangle::default()),
        }
    }
    /// Store input pixels without rebuilding geometry; `volume.init(a, b, c)`.
    pub fn init(&self, first: Point2, second: Point2, third: Point2) {
        self.selection_triangle.write().points = [first, second, third];
    }
    /// Release only the builder, retaining the camera and geometry; `volume.clear()`.
    pub fn clear(&self) {
        self.base.builder.write().nullify();
    }
}
