//! Shared builder state and native overrides for every frustum (OCCT 3d097a0).
use super::boundary_sphere::select_mgr_boundary_intersect_sphere;
use super::select_mgr_base_intersector_dump_json;
use super::{SelectMgrBaseIntersector, SelectMgrBaseIntersectorBase, SelectMgrFrustumBuilder};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, SharedState};
use rustycad_tkmath::gp::{Direction3, Point3};
use rustycad_tkservice::graphic3d::Graphic3dCamera;
use rustycad_tkservice::json_dump::{base_class, class_begin, field_integer, field_pointer};
use std::io;

/// Embedded native state; concrete intersectors delegate their virtual setters here.
pub struct SelectMgrBaseFrustumBase {
    /// Inherited camera and selection type.
    pub intersector: SelectMgrBaseIntersectorBase,
    /// `myPixelTolerance`, shared with derived picking volumes.
    pub pixel_tolerance: SharedState<i32>,
    /// `myBuilder`, retained by handle.
    pub builder: SharedState<Handle<SelectMgrFrustumBuilder>>,
}

impl Default for SelectMgrBaseFrustumBase {
    /// Start with tolerance two and a fresh builder; `SelectMgrBaseFrustumBase::default()`.
    fn default() -> Self {
        Self {
            intersector: SelectMgrBaseIntersectorBase::default(),
            pixel_tolerance: SharedState::new(2),
            builder: SharedState::new(Handle::new(SelectMgrFrustumBuilder::new())),
        }
    }
}

impl SelectMgrBaseFrustumBase {
    /// Replace the builder, adopting its camera only if present; `base.set_builder(&builder)`.
    pub fn set_builder(&self, builder: &Handle<SelectMgrFrustumBuilder>) {
        *self.builder.write() = builder.clone();
        if let Some(builder) = builder.get() {
            *self.intersector.camera.write() = builder.camera();
        }
    }
    /// Propagate a camera and invalidate the old viewport; `base.set_camera(&camera)`.
    pub fn set_camera(&self, camera: &Handle<Graphic3dCamera>) {
        *self.intersector.camera.write() = camera.clone();
        let builder = self.builder.read().clone();
        if let Some(builder) = builder.get() {
            builder.set_camera(camera);
            builder.invalidate_viewport();
        }
    }
    /// Keep even non-positive tolerances, as natively; `base.set_pixel_tolerance(-1)`.
    pub fn set_pixel_tolerance(&self, tolerance: i32) {
        *self.pixel_tolerance.write() = tolerance;
    }
    /// Pass the window dimensions to the current builder; `base.set_window_size(800, 600)`.
    pub fn set_window_size(&self, width: i32, height: i32) {
        let builder = self.builder.read().clone();
        if let Some(builder) = builder.get() {
            builder.set_window_size(width, height);
        }
    }
    /// Read builder dimensions; a null builder has none; `base.window_size()`.
    pub fn window_size(&self) -> Option<(i32, i32)> {
        self.builder
            .read()
            .get()
            .map(SelectMgrFrustumBuilder::window_size)
    }
    /// Pass a viewport to the current builder; `base.set_viewport(0., 0., 1., 1.)`.
    pub fn set_viewport(&self, x: f64, y: f64, width: f64, height: f64) {
        let builder = self.builder.read().clone();
        if let Some(builder) = builder.get() {
            builder.set_viewport(x, y, width, height);
        }
    }
    /// Dump state with the inherited base at the requested depth; `base.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "SelectMgr_BaseFrustum")?;
        base_class(stream, depth, &|s, _| {
            select_mgr_base_intersector_dump_json(&self.intersector, s)
        })?;
        field_integer(stream, "myPixelTolerance", *self.pixel_tolerance.read())?;
        field_pointer(
            stream,
            "myBuilder",
            self.builder.read().object_address() as usize,
        )
    }
}

/// A frustum supplies this embedded state and the required intersector geometry.
pub trait SelectMgrBaseFrustum: SelectMgrBaseIntersector {
    /// Expose shared frustum state; `frustum.frustum_base()`.
    fn frustum_base(&self) -> &SelectMgrBaseFrustumBase;
    /// Replace the builder and adopt its camera; `frustum.set_builder(&builder)`.
    fn set_builder(&self, builder: &Handle<SelectMgrFrustumBuilder>) {
        self.frustum_base().set_builder(builder);
    }
    /// Test a closed boundary against a sphere, preserving the output on a miss.
    /// Example: `frustum.is_boundary_intersect_sphere(&center, 1., &normal, &points, &mut inside)?`.
    fn is_boundary_intersect_sphere(
        &self,
        center: &Point3,
        radius: f64,
        normal: &Direction3,
        points: &[Point3],
        inside: &mut bool,
    ) -> Result<bool, Failure> {
        select_mgr_boundary_intersect_sphere(self, center, radius, normal, points, inside)
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn SelectMgrBaseFrustum,
    "SelectMgr_BaseFrustum",
    dyn SelectMgrBaseIntersector
);
