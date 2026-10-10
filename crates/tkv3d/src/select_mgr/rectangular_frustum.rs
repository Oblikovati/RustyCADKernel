//! Rectangular point/box selection state, preserving OCCT 3d097a0 construction.
use super::*;
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient};
use rustycad_tkmath::gp::{Direction3, Point2, Point3, Xy};

/// Stored selection bounds, independent from the last built geometry.
#[derive(Clone, Copy, Debug)]
pub struct SelectMgrSelectionRectangle {
    minimum: Point2,
    maximum: Point2,
}
impl Default for SelectMgrSelectionRectangle {
    fn default() -> Self {
        Self::new()
    }
}
impl SelectMgrSelectionRectangle {
    /// Start with RealLast bounds; `SelectMgrSelectionRectangle::new()`.
    pub fn new() -> Self {
        let sentinel = Point2::from(Xy::new(f64::MAX, f64::MAX));
        Self {
            minimum: sentinel,
            maximum: sentinel,
        }
    }
    /// Read the shared minimum/mouse position; `rectangle.mouse_pos()`.
    pub fn mouse_pos(&self) -> Point2 {
        self.minimum
    }
    /// Set both bounds for point selection; `rectangle.set_mouse_pos(pixel)`.
    pub fn set_mouse_pos(&mut self, pixel: Point2) {
        self.minimum = pixel;
        self.maximum = pixel;
    }
    /// Read the minimum without reordering; `rectangle.min_pnt()`.
    pub fn min_pnt(&self) -> Point2 {
        self.minimum
    }
    /// Store the minimum without reordering; `rectangle.set_min_pnt(pixel)`.
    pub fn set_min_pnt(&mut self, pixel: Point2) {
        self.minimum = pixel;
    }
    /// Read the maximum without reordering; `rectangle.max_pnt()`.
    pub fn max_pnt(&self) -> Point2 {
        self.maximum
    }
    /// Store the maximum without reordering; `rectangle.set_max_pnt(pixel)`.
    pub fn set_max_pnt(&mut self, pixel: Point2) {
        self.maximum = pixel;
    }
}
#[derive(Clone, Copy)]
pub(super) struct RectangularRay {
    pub near: Point3,
    pub far: Point3,
    pub direction: Direction3,
    pub scale: f64,
}
impl Default for RectangularRay {
    fn default() -> Self {
        Self {
            near: Point3::default(),
            far: Point3::default(),
            direction: Direction3::X,
            scale: 1.,
        }
    }
}
/// A finite selecting frustum built from a pixel or rectangle; native RTTI is BaseFrustum.
pub struct SelectMgrRectangularFrustum {
    pub(super) transient: StandardTransient,
    /// Shared builder, tolerance and camera; `volume.base.window_size()`.
    pub base: SelectMgrBaseFrustumBase,
    /// Protected native geometry; `volume.frustum_core.read().vertices()`.
    pub frustum_core: SharedState<SelectMgrFrustumCore<4>>,
    pub(super) selection_rectangle: SharedState<SelectMgrSelectionRectangle>,
    pub(super) ray: SharedState<RectangularRay>,
}
impl Default for SelectMgrRectangularFrustum {
    fn default() -> Self {
        Self::new()
    }
}
impl SelectMgrBaseFrustum for SelectMgrRectangularFrustum {
    fn frustum_base(&self) -> &SelectMgrBaseFrustumBase {
        &self.base
    }
}
impl SelectMgrRectangularFrustum {
    /// Start with empty geometry and unknown selection; `SelectMgrRectangularFrustum::new()`.
    pub fn new() -> Self {
        Self {
            transient: StandardTransient::default(),
            base: SelectMgrBaseFrustumBase::default(),
            frustum_core: SharedState::new(SelectMgrFrustumCore::new()),
            selection_rectangle: SharedState::new(SelectMgrSelectionRectangle::new()),
            ray: SharedState::new(RectangularRay::default()),
        }
    }
    /// Store a point, retaining geometry until Build; `volume.init(pixel)`.
    pub fn init(&self, pixel: Point2) {
        *self.base.intersector.selection_type.write() = SelectMgrSelectionType::Point;
        self.selection_rectangle.write().set_mouse_pos(pixel);
    }
    /// Store box bounds without sorting them; `volume.init_box(minimum, maximum)`.
    pub fn init_box(&self, minimum: Point2, maximum: Point2) {
        *self.base.intersector.selection_type.write() = SelectMgrSelectionType::Box;
        let mut rectangle = self.selection_rectangle.write();
        rectangle.set_min_pnt(minimum);
        rectangle.set_max_pnt(maximum);
    }
    /// Copy the eight native ordered vertices; `volume.get_vertices()`.
    pub fn get_vertices(&self) -> [Point3; 8] {
        self.frustum_core
            .read()
            .vertices()
            .try_into()
            .expect("eight rectangular vertices")
    }
    pub(super) fn rectangular_initialized(&self, operation: &str) -> Result<(), Failure> {
        let selection = self.get_selection_type();
        if matches!(
            selection,
            SelectMgrSelectionType::Point | SelectMgrSelectionType::Box
        ) {
            return Ok(());
        }
        Err(Failure::with_message(
            "Standard_ProgramError",
            &format!(
                "RectangularFrustum::{operation} selection={selection:?}; expected Point or Box"
            ),
        ))
    }
}
