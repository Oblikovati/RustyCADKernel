//! Real owned rectangular-volume override receives and mutates the actual caller camera field.
use super::*;
use crate::select_basics::SelectBasicsPickResult;
use crate::v3d::V3dViewSource;
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::standard::{
    Failure, Handle, HandleReference, SharedState, WeakHandle, raise_native_failure,
};
use rustycad_tkmath::gp::*;
use rustycad_tkservice::graphic3d::Graphic3dCamera;
use std::sync::atomic::{AtomicBool, Ordering};
pub(super) struct CameraArgumentFrustum {
    pub(super) volume: SelectMgrRectangularFrustum,
    pub(super) getter_probe:
        SharedState<Option<std::sync::Arc<super::camera_getter_fixture::CameraGetterSetterProbe>>>,
    view: WeakHandle<dyn V3dViewSource>,
    replacement: std::sync::Weak<SharedState<Handle<Graphic3dCamera>>>,
    pub(crate) armed: AtomicBool,
    mode: i32,
    pub records: SharedState<[i32; 5]>,
}
rustycad_tkernel::impl_transient!(CameraArgumentFrustum,"CameraArgumentFrustum",SelectMgrRectangularFrustum,volume,
 interfaces:[dyn SelectMgrBaseIntersector]);
impl CameraArgumentFrustum {
    pub(super) fn new(
        view: &Handle<dyn V3dViewSource>,
        replacement: &std::sync::Arc<SharedState<Handle<Graphic3dCamera>>>,
        mode: i32,
    ) -> Handle<Self> {
        Handle::new(Self {
            volume: SelectMgrRectangularFrustum::new(),
            getter_probe: SharedState::new(None),
            view: view.downgrade(),
            replacement: std::sync::Arc::downgrade(replacement),
            armed: AtomicBool::new(true),
            mode,
            records: SharedState::new([0; 5]),
        })
    }
    fn replace_caller_camera(&self, camera: &dyn HandleReference<Graphic3dCamera>) {
        let view = self.view.observe();
        self.records.write()[..3].copy_from_slice(&[
            i32::from(camera.reference_address() == view.camera_ref().reference_address()),
            camera.observe().ref_count(),
            self.volume.transient.ref_count(),
        ]);
        let replacement = self.replacement.upgrade().unwrap();
        let reference =
            rustycad_tkernel::standard::HandleFieldRef::new(replacement.as_ref(), |slot| slot);
        view.set_camera(&reference).unwrap();
        self.records.write()[3..].copy_from_slice(&[
            i32::from(camera.object_address() == reference.object_address()),
            camera.observe().ref_count(),
        ]);
        self.raise_camera_argument_failure();
    }
    fn raise_camera_argument_failure(&self) {
        match self.mode {
            1 => raise_native_failure(Err(Failure::new(
                "selector Camera argument fault=1; expected success",
            ))),
            2 => std::panic::panic_any(93_u32),
            _ => {}
        }
    }
}
impl SelectMgrBaseIntersector for CameraArgumentFrustum {
    fn intersector_base(&self) -> &SelectMgrBaseIntersectorBase {
        self.volume.intersector_base()
    }
    fn build(&self) -> Result<(), Failure> {
        self.volume.build()
    }
    fn is_scalable(&self) -> bool {
        self.volume.is_scalable()
    }
    fn copy_with_builder(
        &self,
        builder: &Handle<SelectMgrFrustumBuilder>,
    ) -> Result<Handle<dyn SelectMgrBaseIntersector>, Failure> {
        self.volume.copy_with_builder(builder)
    }
    fn scale_and_transform(
        &self,
        scale: i32,
        transform: &GeneralTransform3,
        builder: &Handle<SelectMgrFrustumBuilder>,
    ) -> Result<Handle<dyn SelectMgrBaseIntersector>, Failure> {
        self.volume.scale_and_transform(scale, transform, builder)
    }
    fn set_camera(&self, camera: &dyn HandleReference<Graphic3dCamera>) {
        let probe = self.getter_probe.read().clone();
        if let Some(probe) = probe {
            probe.modify_captured_field(self);
        }
        if self.armed.swap(false, Ordering::Relaxed) {
            self.replace_caller_camera(camera);
        }
        self.volume.set_camera(camera);
    }
    fn overlaps_box_pick(
        &self,
        min: &Vec3<f64>,
        max: &Vec3<f64>,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.volume.overlaps_box_pick(min, max, clip, pick)
    }

    fn overlaps_box(
        &self,
        min: &Vec3<f64>,
        max: &Vec3<f64>,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        self.volume.overlaps_box(min, max, inside)
    }

    fn overlaps_point_pick(
        &self,
        point: &Point3,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.volume.overlaps_point_pick(point, clip, pick)
    }

    fn overlaps_point(&self, point: &Point3) -> Result<bool, Failure> {
        self.volume.overlaps_point(point)
    }

    fn overlaps_polygon(
        &self,
        points: &[Point3],
        sens_type: i32,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.volume.overlaps_polygon(points, sens_type, clip, pick)
    }

    fn overlaps_segment(
        &self,
        p1: &Point3,
        p2: &Point3,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.volume.overlaps_segment(p1, p2, clip, pick)
    }

    fn overlaps_triangle(
        &self,
        p1: &Point3,
        p2: &Point3,
        p3: &Point3,
        sens_type: i32,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.volume
            .overlaps_triangle(p1, p2, p3, sens_type, clip, pick)
    }

    fn overlaps_sphere(
        &self,
        center: &Point3,
        radius: f64,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        self.volume.overlaps_sphere(center, radius, inside)
    }

    fn overlaps_sphere_pick(
        &self,
        center: &Point3,
        radius: f64,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.volume.overlaps_sphere_pick(center, radius, clip, pick)
    }

    fn overlaps_cylinder_pick(
        &self,
        bottom_rad: f64,
        top_rad: f64,
        height: f64,
        trsf: &Transform3,
        is_hollow: bool,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.volume
            .overlaps_cylinder_pick(bottom_rad, top_rad, height, trsf, is_hollow, clip, pick)
    }

    fn overlaps_cylinder(
        &self,
        bottom_rad: f64,
        top_rad: f64,
        height: f64,
        trsf: &Transform3,
        is_hollow: bool,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        self.volume
            .overlaps_cylinder(bottom_rad, top_rad, height, trsf, is_hollow, inside)
    }

    fn overlaps_circle_pick(
        &self,
        radius: f64,
        trsf: &Transform3,
        is_filled: bool,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.volume
            .overlaps_circle_pick(radius, trsf, is_filled, clip, pick)
    }

    fn overlaps_circle(
        &self,
        radius: f64,
        trsf: &Transform3,
        is_filled: bool,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        self.volume.overlaps_circle(radius, trsf, is_filled, inside)
    }
}
