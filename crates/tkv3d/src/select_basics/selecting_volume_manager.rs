//! `SelectBasics_SelectingVolumeManager`: the abstract selecting volume that the sensitive
//! entities test themselves against (v3d-design.md section 2).
//!
//! The native overloads that differ by their last argument get one name each: the pick-result
//! form keeps the native name (`overlaps_box`), the `bool* theInside` form takes the suffix
//! `_inside` and an `Option<&mut bool>` (`None` is the native null pointer), and the
//! depth-less point test is `overlaps_point_only`.
use super::SelectBasicsPickResult;
use crate::select_mgr::SelectMgrSelectionType;
use rustycad_tkernel::ncollection::{Array1, DynamicArray, HArray1, Vec3, Vec4};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkmath::gp::{Direction3, Point2, Point3, Transform3};
use std::io;

/// A selecting volume (frustum, frustum set or axis) as seen by the sensitive entities.
///
/// Example: `if manager.overlaps_point(&p, &mut pick) { pick.depth(); }`.
pub trait SelectBasicsSelectingVolumeManager {
    /// `GetActiveSelectionType`; `manager.get_active_selection_type()`.
    fn get_active_selection_type(&self) -> SelectMgrSelectionType;
    /// `OverlapsBox` with depth: whether the box overlaps the volume; `manager.overlaps_box(&min, &max, &mut pick)`.
    fn overlaps_box(
        &self,
        box_min: &Vec3<f64>,
        box_max: &Vec3<f64>,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure>;
    /// `OverlapsBox` with the full-inside flag; `manager.overlaps_box_inside(&min, &max, Some(&mut inside))`.
    fn overlaps_box_inside(
        &self,
        box_min: &Vec3<f64>,
        box_max: &Vec3<f64>,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure>;
    /// `OverlapsPoint` with depth; `manager.overlaps_point(&p, &mut pick)`.
    fn overlaps_point(
        &self,
        point: &Point3,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure>;
    /// `OverlapsPoint` without depth (the inclusion helper); `manager.overlaps_point_only(&p)`.
    fn overlaps_point_only(&self, point: &Point3) -> Result<bool, Failure>;
    /// `OverlapsPolygon`; `sens_type` is a `Select3D_TypeOfSensitivity` value.
    /// Example: `manager.overlaps_polygon(&points, 0, &mut pick)`.
    fn overlaps_polygon(
        &self,
        points: &Array1<'_, Point3>,
        sens_type: i32,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure>;
    /// `OverlapsSegment`; `manager.overlaps_segment(&a, &b, &mut pick)`.
    fn overlaps_segment(
        &self,
        pt1: &Point3,
        pt2: &Point3,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure>;
    /// `OverlapsTriangle`; `manager.overlaps_triangle(&a, &b, &c, 0, &mut pick)`.
    fn overlaps_triangle(
        &self,
        pt1: &Point3,
        pt2: &Point3,
        pt3: &Point3,
        sens_type: i32,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure>;
    /// `OverlapsSphere` with depth; `manager.overlaps_sphere(&center, 1.0, &mut pick)`.
    fn overlaps_sphere(
        &self,
        center: &Point3,
        radius: f64,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure>;
    /// `OverlapsSphere` with the full-inside flag; `manager.overlaps_sphere_inside(&c, 1.0, None)`.
    fn overlaps_sphere_inside(
        &self,
        center: &Point3,
        radius: f64,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure>;
    /// `OverlapsCylinder` with depth (a cone when the radii differ).
    /// Example: `manager.overlaps_cylinder(1.0, 1.0, 2.0, &trsf, false, &mut pick)`.
    fn overlaps_cylinder(
        &self,
        bottom_rad: f64,
        top_rad: f64,
        height: f64,
        trsf: &Transform3,
        is_hollow: bool,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure>;
    /// `OverlapsCylinder` with the full-inside flag.
    /// Example: `manager.overlaps_cylinder_inside(1.0, 1.0, 2.0, &trsf, false, None)`.
    fn overlaps_cylinder_inside(
        &self,
        bottom_rad: f64,
        top_rad: f64,
        height: f64,
        trsf: &Transform3,
        is_hollow: bool,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure>;
    /// `OverlapsCircle` with depth: a circle of `gp::XOY()` placed by `trsf`.
    /// Example: `manager.overlaps_circle(1.0, &trsf, true, &mut pick)`.
    fn overlaps_circle(
        &self,
        radius: f64,
        trsf: &Transform3,
        is_filled: bool,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure>;
    /// `OverlapsCircle` with the full-inside flag; `manager.overlaps_circle_inside(1.0, &trsf, true, None)`.
    fn overlaps_circle_inside(
        &self,
        radius: f64,
        trsf: &Transform3,
        is_filled: bool,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure>;
    /// `DistToGeometryCenter`; `manager.dist_to_geometry_center(&cog)`.
    fn dist_to_geometry_center(&self, cog: &Point3) -> Result<f64, Failure>;
    /// `DetectedPoint`: the point at a depth of the picking ray; `manager.detected_point(1.0)`.
    fn detected_point(&self, depth: f64) -> Result<Point3, Failure>;
    /// `IsOverlapAllowed`: false for the inclusion test; `manager.is_overlap_allowed()`.
    fn is_overlap_allowed(&self) -> bool;
    /// `GetNearPickedPnt`; `manager.get_near_picked_pnt()`.
    fn get_near_picked_pnt(&self) -> Result<Point3, Failure>;
    /// `GetFarPickedPnt`; `manager.get_far_picked_pnt()`.
    fn get_far_picked_pnt(&self) -> Result<Point3, Failure>;
    /// `GetViewRayDirection`; `manager.get_view_ray_direction()`.
    fn get_view_ray_direction(&self) -> Result<Direction3, Failure>;
    /// `IsScalableActiveVolume`; `manager.is_scalable_active_volume()`.
    fn is_scalable_active_volume(&self) -> bool;
    /// `GetMousePosition` (an infinite point when not supported); `manager.get_mouse_position()`.
    fn get_mouse_position(&self) -> Point2;
    /// `GetPlanes`: the plane equations `Ax + By + Cz + D = 0`; `manager.get_planes(&mut planes)`.
    fn get_planes(&self, plane_equations: &mut DynamicArray<Vec4<f64>>);
    /// `DumpJson`: writes nothing by default; `manager.dump_json(&mut stream, -1)?`.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let _ = (stream, depth);
        Ok(())
    }
    /// The deprecated `Overlaps(theArrayOfPts handle, ...)` alias of `OverlapsPolygon`.
    /// Example: `manager.overlaps_harray_polygon(&points, 0, &mut pick)`.
    fn overlaps_harray_polygon(
        &self,
        points: &HArray1<Point3>,
        sens_type: i32,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.overlaps_polygon(&points.array1(), sens_type, pick_result)
    }
}

/// The deprecated `SelectBasics_SelectingVolumeManager::Point`, `Box`, `Polyline` and
/// `Unknown` aliases of [`SelectMgrSelectionType`].
pub struct SelectBasicsSelectionTypeAliases;

impl SelectBasicsSelectionTypeAliases {
    /// `SelectBasics_SelectingVolumeManager::Point`.
    pub const POINT: SelectMgrSelectionType = SelectMgrSelectionType::Point;
    /// `SelectBasics_SelectingVolumeManager::Box`.
    pub const BOX: SelectMgrSelectionType = SelectMgrSelectionType::Box;
    /// `SelectBasics_SelectingVolumeManager::Polyline`.
    pub const POLYLINE: SelectMgrSelectionType = SelectMgrSelectionType::Polyline;
    /// `SelectBasics_SelectingVolumeManager::Unknown`.
    pub const UNKNOWN: SelectMgrSelectionType = SelectMgrSelectionType::Unknown;
}
