//! Active selection volume ownership and reuse (OCCT 3d097a0).
use super::*;
use crate::select_basics::SelectBasicsSelectingVolumeManager;
use rustycad_tkernel::{
    ncollection::Array1,
    standard::{Failure, Handle},
};
use rustycad_tkmath::gp::{Axis1, GeneralTransform3, Point2, Point3};
use rustycad_tkservice::graphic3d::{Graphic3dCamera, Graphic3dSequenceOfHClipPlane};

/// A value manager sharing its active volume and clip planes when cloned.
/// Example: `let mut manager = SelectMgrSelectingVolumeManager::new(); manager.init_axis_selecting_volume(Axis1::default());`.
#[derive(Clone, Default)]
pub struct SelectMgrSelectingVolumeManager {
    pub(super) active: Handle<dyn SelectMgrBaseIntersector>,
    pub(super) view_planes: Handle<Graphic3dSequenceOfHClipPlane>,
    pub(super) object_planes: Handle<Graphic3dSequenceOfHClipPlane>,
    pub(super) clip_range: SelectMgrViewClipRange,
    pub(super) allow_overlap: bool,
}
impl SelectMgrSelectingVolumeManager {
    /// Start without an active volume; `SelectMgrSelectingVolumeManager::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Read the shared selector; `manager.active_volume()`.
    pub fn active_volume(&self) -> &Handle<dyn SelectMgrBaseIntersector> {
        &self.active
    }
    /// Activate a custom selector, including null; `manager.init_selecting_volume(&volume)`.
    pub fn init_selecting_volume(&mut self, volume: &Handle<dyn SelectMgrBaseIntersector>) {
        self.active = volume.clone();
    }
    fn rectangular_volume(&self) -> Handle<SelectMgrRectangularFrustum> {
        let existing = self.active.cast::<SelectMgrRectangularFrustum>();
        if existing.is_null() {
            return Handle::new(SelectMgrRectangularFrustum::new());
        }
        existing
    }
    /// Reuse a rectangle for point selection; `manager.init_point_selecting_volume(pixel)`.
    pub fn init_point_selecting_volume(&mut self, point: Point2) {
        let volume = self.rectangular_volume();
        volume.init(point);
        self.active = volume.cast();
    }
    /// Reuse a rectangle for box selection; `manager.init_box_selecting_volume(min, max)`.
    pub fn init_box_selecting_volume(&mut self, minimum: Point2, maximum: Point2) {
        let volume = self.rectangular_volume();
        volume.init_box(minimum, maximum);
        self.active = volume.cast();
    }
    /// Reuse a polyline set and refresh its overlap flag; `manager.init_polyline_selecting_volume(&points)?`.
    pub fn init_polyline_selecting_volume(
        &mut self,
        points: &Array1<'_, Point2>,
    ) -> Result<(), Failure> {
        let mut volume = self.active.cast::<SelectMgrTriangularFrustumSet>();
        if volume.is_null() {
            volume = Handle::new(SelectMgrTriangularFrustumSet::new());
        }
        volume.init(points)?;
        self.active = volume.cast();
        volume.set_allow_overlap_detection(self.is_overlap_allowed());
        Ok(())
    }
    /// Reuse an axis selector; `manager.init_axis_selecting_volume(axis)`.
    pub fn init_axis_selecting_volume(&mut self, axis: Axis1) {
        let mut volume = self.active.cast::<SelectMgrAxisIntersector>();
        if volume.is_null() {
            volume = Handle::new(SelectMgrAxisIntersector::new());
        }
        volume.init(axis);
        self.active = volume.cast();
    }
    pub(super) fn required_volume(
        &self,
        operation: &str,
    ) -> Result<&dyn SelectMgrBaseIntersector, Failure> {
        self.active.get().ok_or_else(|| Failure::with_message(
            "Standard_ProgramError",
            &format!("SelectingVolumeManager::{operation}: active=null; expected initialized selection volume"),
        ))
    }
    /// Construct the initialized volume; `manager.build_selecting_volume()?`.
    pub fn build_selecting_volume(&self) -> Result<(), Failure> {
        self.required_volume("BuildSelectingVolume")?.build()
    }
    /// Deprecated point Init+Build alias; `manager.build_point_selecting_volume(pixel)?`.
    pub fn build_point_selecting_volume(&mut self, point: Point2) -> Result<(), Failure> {
        self.init_point_selecting_volume(point);
        self.build_selecting_volume()
    }
    /// Deprecated box Init+Build alias; `manager.build_box_selecting_volume(min, max)?`.
    pub fn build_box_selecting_volume(
        &mut self,
        minimum: Point2,
        maximum: Point2,
    ) -> Result<(), Failure> {
        self.init_box_selecting_volume(minimum, maximum);
        self.build_selecting_volume()
    }
    /// Deprecated polyline Init+Build alias; `manager.build_polyline_selecting_volume(&points)?`.
    pub fn build_polyline_selecting_volume(
        &mut self,
        points: &Array1<'_, Point2>,
    ) -> Result<(), Failure> {
        self.init_polyline_selecting_volume(points)?;
        self.build_selecting_volume()
    }
    /// Copy transformed geometry; an empty manager yields a fresh default; `manager.scale_and_transform(2, &trsf, &builder)?`.
    pub fn scale_and_transform(
        &self,
        scale: i32,
        trsf: &GeneralTransform3,
        builder: &Handle<SelectMgrFrustumBuilder>,
    ) -> Result<Self, Failure> {
        let Some(volume) = self.active.get() else {
            return Ok(Self::new());
        };
        let active = volume.scale_and_transform(scale, trsf, builder)?;
        Ok(Self {
            active,
            ..self.clone()
        })
    }
    /// Copy and immediately rebuild with another builder; `manager.copy_with_builder(&builder)?`.
    pub fn copy_with_builder(
        &self,
        builder: &Handle<SelectMgrFrustumBuilder>,
    ) -> Result<Self, Failure> {
        let mut copy = self.clone();
        if let Some(volume) = self.active.get() {
            copy.active = volume.copy_with_builder(builder)?;
            copy.build_selecting_volume()?;
        }
        Ok(copy)
    }
    /// Read the active camera or null; `manager.camera()`.
    pub fn camera(&self) -> Handle<Graphic3dCamera> {
        self.active
            .get()
            .map_or_else(Handle::default, |volume| volume.camera())
    }
    /// Set the initialized volume's camera; `manager.set_camera(&camera)?`.
    pub fn set_camera(&self, camera: &Handle<Graphic3dCamera>) -> Result<(), Failure> {
        self.required_volume("SetCamera")?.set_camera(camera);
        Ok(())
    }
    /// Read window dimensions, preserving absent native outputs; `manager.window_size()`.
    pub fn window_size(&self) -> Option<(i32, i32)> {
        self.active.get().and_then(|volume| volume.window_size())
    }
    /// Set initialized volume dimensions; `manager.set_window_size(800, 600)?`.
    pub fn set_window_size(&self, width: i32, height: i32) -> Result<(), Failure> {
        self.required_volume("SetWindowSize")?
            .set_window_size(width, height);
        Ok(())
    }
    /// Set the initialized viewport; `manager.set_viewport(0., 0., 1., 1.)?`.
    pub fn set_viewport(&self, x: f64, y: f64, width: f64, height: f64) -> Result<(), Failure> {
        self.required_volume("SetViewport")?
            .set_viewport(x, y, width, height);
        Ok(())
    }
    /// Set initialized volume tolerance; `manager.set_pixel_tolerance(2)?`.
    pub fn set_pixel_tolerance(&self, tolerance: i32) -> Result<(), Failure> {
        self.required_volume("SetPixelTolerance")?
            .set_pixel_tolerance(tolerance);
        Ok(())
    }
    /// Change manager permission; existing polyline geometry retains its flag until Init.
    /// Example: `manager.allow_overlap_detection(true)`.
    pub fn allow_overlap_detection(&mut self, allow: bool) {
        self.allow_overlap = allow;
    }
    /// Copy rectangular vertices, or None for null/nonrectangular volumes.
    /// Native OCCT 3d097a0 statically casts other volumes with undefined behavior.
    /// Example: `manager.get_vertices()`.
    pub fn get_vertices(&self) -> Option<[Point3; 8]> {
        self.active
            .cast::<SelectMgrRectangularFrustum>()
            .get()
            .map(|volume| volume.get_vertices())
    }
}
