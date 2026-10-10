//! `SelectMgr_BaseIntersector`: the abstract selecting volume (frustum, frustum set or axis)
//! with its camera and selection type, and the ray intersections with the quadrics shared by
//! the volumes (v3d-design.md section 6).
use super::ray_intersections::*;
use super::{SelectMgrFrustumBuilder, SelectMgrSelectionType, SelectMgrViewClipRange};
use crate::select_basics::SelectBasicsPickResult;
use rustycad_tkernel::ncollection::{DynamicArray, Vec3, Vec4};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, Transient};
use rustycad_tkmath::gp::{
    Direction3, GeneralTransform3, Point2, Point3, StandardDirection3, Transform3, Xy,
};
use rustycad_tkservice::graphic3d::Graphic3dCamera;
use rustycad_tkservice::json_dump::{class_begin, field_integer, field_pointer};
use std::io;

/// The camera and selection type of every intersector.
pub struct SelectMgrBaseIntersectorBase {
    /// `myCamera`.
    pub camera: SharedState<Handle<Graphic3dCamera>>,
    /// `mySelectionType`.
    pub selection_type: SharedState<SelectMgrSelectionType>,
}

impl Default for SelectMgrBaseIntersectorBase {
    /// No camera, unknown selection type.
    fn default() -> Self {
        Self {
            camera: SharedState::new(Handle::default()),
            selection_type: SharedState::new(SelectMgrSelectionType::Unknown),
        }
    }
}

/// A selecting volume; the default bodies are the native ones.
pub trait SelectMgrBaseIntersector: Transient {
    /// The embedded base; `volume.intersector_base()`.
    fn intersector_base(&self) -> &SelectMgrBaseIntersectorBase;
    /// `Build` (pure virtual): builds the volume after its initialization; `volume.build()?`.
    fn build(&self) -> Result<(), Failure>;
    /// `IsScalable` (pure virtual); `volume.is_scalable()`.
    fn is_scalable(&self) -> bool;
    /// `SetPixelTolerance`: nothing by default; `volume.set_pixel_tolerance(2)`.
    fn set_pixel_tolerance(&self, tol: i32) {
        let _ = tol;
    }
    /// `ScaleAndTransform` (pure virtual): a volume scaled by a pixel tolerance and transformed.
    /// Example: `volume.scale_and_transform(4, &trsf, &builder)?`.
    fn scale_and_transform(
        &self,
        scale_factor: i32,
        trsf: &GeneralTransform3,
        builder: &Handle<SelectMgrFrustumBuilder>,
    ) -> Result<Handle<dyn SelectMgrBaseIntersector>, Failure>;
    /// `CopyWithBuilder` (pure virtual); `volume.copy_with_builder(&builder)?`.
    fn copy_with_builder(
        &self,
        builder: &Handle<SelectMgrFrustumBuilder>,
    ) -> Result<Handle<dyn SelectMgrBaseIntersector>, Failure>;
    /// `SetCamera`; `volume.set_camera(&camera)`.
    fn set_camera(&self, camera: &Handle<Graphic3dCamera>) {
        *self.intersector_base().camera.write() = camera.clone();
    }
    /// `WindowSize`: None by default (natively the out arguments are untouched).
    /// Example: `volume.window_size()`.
    fn window_size(&self) -> Option<(i32, i32)> {
        None
    }
    /// `SetWindowSize`: nothing by default; `volume.set_window_size(400, 300)`.
    fn set_window_size(&self, width: i32, height: i32) {
        let _ = (width, height);
    }
    /// `SetViewport`: nothing by default; `volume.set_viewport(0.0, 0.0, 1.0, 1.0)`.
    fn set_viewport(&self, x: f64, y: f64, width: f64, height: f64) {
        let _ = (x, y, width, height);
    }
    /// `GetNearPnt`: the origin by default; `volume.get_near_pnt()?`.
    fn get_near_pnt(&self) -> Result<Point3, Failure> {
        Ok(Point3::default())
    }
    /// `GetFarPnt`: RealLast by default; `volume.get_far_pnt()?`.
    fn get_far_pnt(&self) -> Result<Point3, Failure> {
        Ok(Point3::new(real_last(), real_last(), real_last()))
    }
    /// `GetViewRayDirection`: `gp_Dir()` (+X) by default; `volume.get_view_ray_direction()?`.
    fn get_view_ray_direction(&self) -> Result<Direction3, Failure> {
        // OCCT 3d097a0 gp_Dir() uses +X; the base does not assume a camera ray.
        Ok(Direction3::from_standard(StandardDirection3::X))
    }
    /// `GetMousePosition`: RealLast by default; `volume.get_mouse_position()`.
    fn get_mouse_position(&self) -> Point2 {
        Point2::from(Xy::new(real_last(), real_last()))
    }
    /// `GetPlanes`: none by default; `volume.get_planes(&mut planes)`.
    fn get_planes(&self, planes: &mut DynamicArray<Vec4<f64>>) {
        planes.clear(false);
    }
    /// `OverlapsBox` with depth (pure virtual).
    /// Example: `volume.overlaps_box_pick(&min, &max, &clip, &mut pick)?`.
    fn overlaps_box_pick(
        &self,
        min: &Vec3<f64>,
        max: &Vec3<f64>,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure>;
    /// `OverlapsBox` with the full-inside flag (pure virtual); `volume.overlaps_box(&min, &max, None)?`.
    fn overlaps_box(
        &self,
        min: &Vec3<f64>,
        max: &Vec3<f64>,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure>;
    /// `OverlapsPoint` with depth (pure virtual); `volume.overlaps_point_pick(&p, &clip, &mut pick)?`.
    fn overlaps_point_pick(
        &self,
        point: &Point3,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure>;
    /// `OverlapsPoint` (pure virtual); `volume.overlaps_point(&p)?`.
    fn overlaps_point(&self, point: &Point3) -> Result<bool, Failure>;
    /// `OverlapsPolygon` (pure virtual); `volume.overlaps_polygon(&points, 0, &clip, &mut pick)?`.
    fn overlaps_polygon(
        &self,
        points: &[Point3],
        sens_type: i32,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure>;
    /// `OverlapsSegment` (pure virtual); `volume.overlaps_segment(&a, &b, &clip, &mut pick)?`.
    fn overlaps_segment(
        &self,
        p1: &Point3,
        p2: &Point3,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure>;
    /// `OverlapsTriangle` (pure virtual); `volume.overlaps_triangle(&a, &b, &c, 0, &clip, &mut pick)?`.
    fn overlaps_triangle(
        &self,
        p1: &Point3,
        p2: &Point3,
        p3: &Point3,
        sens_type: i32,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure>;
    /// `OverlapsSphere` with the full-inside flag (pure virtual); `volume.overlaps_sphere(&c, 1.0, None)?`.
    fn overlaps_sphere(
        &self,
        center: &Point3,
        radius: f64,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure>;
    /// `OverlapsSphere` with depth (pure virtual); `volume.overlaps_sphere_pick(&c, 1.0, &clip, &mut pick)?`.
    fn overlaps_sphere_pick(
        &self,
        center: &Point3,
        radius: f64,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure>;
    /// `OverlapsCylinder` with depth (pure virtual).
    /// Example: `volume.overlaps_cylinder_pick(1.0, 1.0, 2.0, &trsf, false, &clip, &mut pick)?`.
    #[allow(clippy::too_many_arguments)]
    fn overlaps_cylinder_pick(
        &self,
        bottom_rad: f64,
        top_rad: f64,
        height: f64,
        trsf: &Transform3,
        is_hollow: bool,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure>;
    /// `OverlapsCylinder` with the full-inside flag (pure virtual).
    /// Example: `volume.overlaps_cylinder(1.0, 1.0, 2.0, &trsf, false, None)?`.
    fn overlaps_cylinder(
        &self,
        bottom_rad: f64,
        top_rad: f64,
        height: f64,
        trsf: &Transform3,
        is_hollow: bool,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure>;
    /// `OverlapsCircle` with depth (pure virtual).
    /// Example: `volume.overlaps_circle_pick(1.0, &trsf, true, &clip, &mut pick)?`.
    fn overlaps_circle_pick(
        &self,
        radius: f64,
        trsf: &Transform3,
        is_filled: bool,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure>;
    /// `OverlapsCircle` with the full-inside flag (pure virtual).
    /// Example: `volume.overlaps_circle(1.0, &trsf, true, None)?`.
    fn overlaps_circle(
        &self,
        radius: f64,
        trsf: &Transform3,
        is_filled: bool,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure>;
    /// `DistToGeometryCenter`: RealLast by default; `volume.dist_to_geometry_center(&cog)?`.
    fn dist_to_geometry_center(&self, cog: &Point3) -> Result<f64, Failure> {
        let _ = cog;
        Ok(real_last())
    }
    /// `DetectedPoint`: RealLast by default; `volume.detected_point(1.0)?`.
    fn detected_point(&self, depth: f64) -> Result<Point3, Failure> {
        let _ = depth;
        Ok(Point3::new(real_last(), real_last(), real_last()))
    }
    /// `DumpJson`; `volume.dump_json(&mut stream, -1)?`.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let _ = depth;
        select_mgr_base_intersector_dump_json(self.intersector_base(), stream)
    }
    /// `RaySphereIntersection`; `volume.ray_sphere_intersection(&c, 1.0, &loc, &dir)`.
    fn ray_sphere_intersection(
        &self,
        center: &Point3,
        radius: f64,
        loc: &Point3,
        ray_dir: &Direction3,
    ) -> Option<SelectMgrRayTimes> {
        select_mgr_ray_sphere_intersection(center, radius, loc, ray_dir)
    }
    /// `RayCylinderIntersection`; `volume.ray_cylinder_intersection(1.0, 1.0, 2.0, &loc, &dir, false)?`.
    fn ray_cylinder_intersection(
        &self,
        bottom_radius: f64,
        top_radius: f64,
        height: f64,
        loc: &Point3,
        ray_dir: &Direction3,
        is_hollow: bool,
    ) -> Result<Option<(f64, Option<f64>)>, Failure> {
        select_mgr_ray_cylinder_intersection(
            bottom_radius,
            top_radius,
            height,
            loc,
            ray_dir,
            is_hollow,
        )
    }
    /// `RayCircleIntersection`; `volume.ray_circle_intersection(1.0, &loc, &dir, true)`.
    fn ray_circle_intersection(
        &self,
        radius: f64,
        loc: &Point3,
        ray_dir: &Direction3,
        is_filled: bool,
    ) -> Option<f64> {
        select_mgr_ray_circle_intersection(radius, loc, ray_dir, is_filled)
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn SelectMgrBaseIntersector,
    "SelectMgr_BaseIntersector",
    StandardTransient
);

/// The members of every intersector.
pub trait SelectMgrBaseIntersectorMembers: SelectMgrBaseIntersector {
    /// `GetSelectionType`; `volume.get_selection_type()`.
    fn get_selection_type(&self) -> SelectMgrSelectionType {
        *self.intersector_base().selection_type.read()
    }
    /// `Camera`; `volume.camera()`.
    fn camera(&self) -> Handle<Graphic3dCamera> {
        self.intersector_base().camera.read().clone()
    }
    /// Whether the camera is orthographic (false without camera, natively a null dereference).
    fn is_orthographic(&self) -> bool {
        self.camera()
            .get()
            .is_some_and(Graphic3dCamera::is_orthographic)
    }
}
impl<T: SelectMgrBaseIntersector + ?Sized> SelectMgrBaseIntersectorMembers for T {}

/// The native `SelectMgr_BaseIntersector::DumpJson` body.
/// Example: `select_mgr_base_intersector_dump_json(volume.intersector_base(), &mut stream)?`.
pub fn select_mgr_base_intersector_dump_json(
    base: &SelectMgrBaseIntersectorBase,
    stream: &mut DumpStream,
) -> io::Result<()> {
    class_begin(stream, "SelectMgr_BaseIntersector")?;
    field_integer(
        stream,
        "mySelectionType",
        *base.selection_type.read() as i32,
    )?;
    field_pointer(
        stream,
        "myCamera",
        base.camera.read().object_address() as usize,
    )
}
