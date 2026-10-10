//! Named frustum fixture: inherited state with abstract geometry guarded against calls.
use crate::abstract_intersector;
use rustycad_tkernel::standard::{Handle, StandardTransient, dump::DumpStream};
use rustycad_tkmath::gp::{Direction3, Point3};
use rustycad_tkservice::graphic3d::Graphic3dCamera;
use rustycad_tkv3d::select_mgr::*;
use std::io;

#[derive(Default)]
pub struct FrustumDefaultsIntersector {
    pub(crate) transient: StandardTransient,
    pub base: SelectMgrBaseFrustumBase,
    pub reject_sphere_rays: bool,
}
rustycad_tkernel::impl_transient!(FrustumDefaultsIntersector, "FrustumDefaultsIntersector", dyn SelectMgrBaseFrustum, transient,
    interfaces: [dyn SelectMgrBaseFrustum, dyn SelectMgrBaseIntersector]);
impl SelectMgrBaseFrustum for FrustumDefaultsIntersector {
    fn frustum_base(&self) -> &SelectMgrBaseFrustumBase {
        &self.base
    }
}
impl SelectMgrBaseIntersector for FrustumDefaultsIntersector {
    fn intersector_base(&self) -> &SelectMgrBaseIntersectorBase {
        &self.base.intersector
    }
    abstract_intersector::reject_abstract_geometry!();
    fn set_camera(&self, camera: &Handle<Graphic3dCamera>) {
        self.base.set_camera(camera);
    }
    fn set_pixel_tolerance(&self, tolerance: i32) {
        self.base.set_pixel_tolerance(tolerance);
    }
    fn set_window_size(&self, width: i32, height: i32) {
        self.base.set_window_size(width, height);
    }
    fn window_size(&self) -> Option<(i32, i32)> {
        self.base.window_size()
    }
    fn set_viewport(&self, x: f64, y: f64, width: f64, height: f64) {
        self.base.set_viewport(x, y, width, height);
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        self.base.dump_json(stream, depth)
    }
    fn ray_sphere_intersection(
        &self,
        center: &Point3,
        radius: f64,
        origin: &Point3,
        direction: &Direction3,
    ) -> Option<SelectMgrRayTimes> {
        if self.reject_sphere_rays {
            return None;
        }
        select_mgr_ray_sphere_intersection(center, radius, origin, direction)
    }
}
