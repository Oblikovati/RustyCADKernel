//! Polyline selecting volume, with native shared input array identity (OCCT 3d097a0).
use super::*;
use rustycad_tkernel::{
    ncollection::{Array1, HArray1},
    standard::{Failure, Handle, SharedState, StandardTransient},
};
use rustycad_tkmath::gp::{Point2, Point3};

/// Input pixels shared by copied sets, as the native HArray1 handle.
#[derive(Clone, Default)]
pub struct SelectMgrSelectionPolyline {
    /// Native `Points`; `polyline.points.array1().lower()`.
    pub points: Handle<HArray1<Point2>>,
}
/// A Delaunay-decomposed polyline volume with optional boundary-crossing rejection.
pub struct SelectMgrTriangularFrustumSet {
    pub(super) transient: StandardTransient,
    /// Shared inherited camera and builder; `volume.base.window_size()`.
    pub base: SelectMgrBaseFrustumBase,
    pub(super) selection_polyline: SharedState<SelectMgrSelectionPolyline>,
    pub(super) frustums: SharedState<Vec<Handle<SelectMgrTriangularFrustum>>>,
    pub(super) boundary_points: SharedState<Array1<'static, Point3>>,
    pub(super) allow_overlap: SharedState<bool>,
}
impl Default for SelectMgrTriangularFrustumSet {
    fn default() -> Self {
        Self::new()
    }
}
impl SelectMgrBaseFrustum for SelectMgrTriangularFrustumSet {
    fn frustum_base(&self) -> &SelectMgrBaseFrustumBase {
        &self.base
    }
}
impl SelectMgrTriangularFrustumSet {
    /// Start empty, requiring full containment; `SelectMgrTriangularFrustumSet::new()`.
    pub fn new() -> Self {
        Self {
            transient: StandardTransient::default(),
            base: SelectMgrBaseFrustumBase::default(),
            selection_polyline: SharedState::new(SelectMgrSelectionPolyline::default()),
            frustums: SharedState::new(Vec::new()),
            boundary_points: SharedState::new(Array1::default()),
            allow_overlap: SharedState::new(false),
        }
    }
    /// Replace pixels in the existing shared array, retaining built geometry; `volume.init(&pixels)?`.
    pub fn init(&self, pixels: &Array1<'_, Point2>) -> Result<(), Failure> {
        let mut selection = self.selection_polyline.write();
        if selection.points.is_null() {
            selection.points = Handle::new(
                HArray1::with_bounds(pixels.lower(), pixels.upper())
                    .map_err(|e| e.into_failure("TriangularFrustumSet::Init"))?,
            );
        }
        let mut array = selection.points.change_array1();
        array
            .resize_bounds(pixels.lower(), pixels.upper(), false)
            .map_err(|e| e.into_failure("TriangularFrustumSet::Init"))?;
        array.assign(pixels);
        *self.base.intersector.selection_type.write() = SelectMgrSelectionType::Polyline;
        Ok(())
    }
    /// Permit partial overlap instead of boundary rejection; `volume.set_allow_overlap_detection(true)`.
    pub fn set_allow_overlap_detection(&self, allow: bool) {
        *self.allow_overlap.write() = allow;
    }
    pub(super) fn frustum_set_initialized(&self, operation: &str) -> Result<(), Failure> {
        let selection = self.get_selection_type();
        if selection == SelectMgrSelectionType::Polyline {
            return Ok(());
        }
        Err(Failure::with_message(
            "Standard_ProgramError",
            &format!(
                "TriangularFrustumSet::{operation} selection={selection:?}; expected Polyline"
            ),
        ))
    }
    pub(super) fn frustum_set_contains_point(&self, point: &Point3) -> bool {
        self.frustums.read().iter().any(|child| {
            child
                .frustum_core
                .read()
                .has_point_overlap(point, child.is_orthographic())
        })
    }
    pub(super) fn frustum_set_any(
        &self,
        query: impl FnMut(&SelectMgrTriangularFrustum) -> Result<bool, Failure>,
    ) -> Result<bool, Failure> {
        self.frustum_set_initialized("Overlaps")?;
        self.frustum_set_any_child(query)
    }
    pub(super) fn frustum_set_any_child(
        &self,
        mut query: impl FnMut(&SelectMgrTriangularFrustum) -> Result<bool, Failure>,
    ) -> Result<bool, Failure> {
        for child in self.frustums.read().iter() {
            if query(child)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
