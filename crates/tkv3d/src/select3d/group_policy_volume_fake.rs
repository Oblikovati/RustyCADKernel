//! Named real-volume extension changes selection type and policies between native virtual calls.
use super::Select3DSensitiveGroup;
use super::group_fault_child_fake::GroupFaultContext;
use super::quadric_oracle_volume::quadric_volume;
use super::selector_volume_delegation::delegate_volume_operation;
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select_mgr::{SelectMgrSelectingVolumeManager, SelectMgrSelectionType};
use rustycad_tkernel::ncollection::{Array1, DynamicArray, Vec3, Vec4};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Direction3, Point2, Point3, Transform3};
use std::{cell::Cell, sync::Arc};
pub(super) struct GroupPolicyVolumeFake<'a> {
    inner: SelectMgrSelectingVolumeManager,
    group: &'a Select3DSensitiveGroup,
    context: Arc<GroupFaultContext>,
    mode: i32,
    calls: Cell<i32>,
}
impl<'a> GroupPolicyVolumeFake<'a> {
    pub(super) fn new(
        group: &'a Select3DSensitiveGroup,
        context: Arc<GroupFaultContext>,
        mode: i32,
    ) -> Self {
        // Real box/frustum behavior remains underneath the injected virtual policy changes.
        Self {
            inner: quadric_volume(8, false, false, false),
            group,
            context,
            mode,
            calls: Cell::new(0),
        }
    }
    fn mutate_policy(&self, calls: i32) {
        if self.mode == 3 && calls == 1 {
            self.group.set_match_type(true);
        }
        if self.mode == 3 && calls == 2 {
            self.group.set_check_overlap_all(true);
        }
        if self.mode == 4 && calls == 2 {
            self.group.set_check_overlap_all(false);
        }
    }
}
impl SelectBasicsSelectingVolumeManager for GroupPolicyVolumeFake<'_> {
    fn get_active_selection_type(&self) -> SelectMgrSelectionType {
        if self.mode == 99 {
            std::panic::panic_any(73_i32);
        }
        let calls = self.calls.get() + 1;
        self.calls.set(calls);
        self.context.trace.lock().unwrap().push(100 + calls);
        self.mutate_policy(calls);
        if self.mode == 1 && calls == 1 || self.mode == 2 && calls == 2 {
            return SelectMgrSelectionType::Point;
        }
        SelectMgrSelectionType::Box
    }
    delegate_volume_operation!(
        overlaps_box(
            box_min: &Vec3<f64>,
            box_max: &Vec3<f64>,
            pick_result: &mut SelectBasicsPickResult,
        ) -> Result<bool, Failure>
    );
    delegate_volume_operation!(
        overlaps_box_inside(
            box_min: &Vec3<f64>,
            box_max: &Vec3<f64>,
            inside: Option<&mut bool>,
        ) -> Result<bool, Failure>
    );
    delegate_volume_operation!(
        overlaps_point(
            point: &Point3,
            pick_result: &mut SelectBasicsPickResult,
        ) -> Result<bool, Failure>
    );
    delegate_volume_operation!(overlaps_point_only(point: &Point3) -> Result<bool, Failure>);
    delegate_volume_operation!(
        overlaps_polygon(
            points: &Array1<'_, Point3>,
            sens_type: i32,
            pick_result: &mut SelectBasicsPickResult,
        ) -> Result<bool, Failure>
    );
    delegate_volume_operation!(
        overlaps_segment(
            pt1: &Point3,
            pt2: &Point3,
            pick_result: &mut SelectBasicsPickResult,
        ) -> Result<bool, Failure>
    );
    delegate_volume_operation!(
        overlaps_triangle(
            pt1: &Point3,
            pt2: &Point3,
            pt3: &Point3,
            sens_type: i32,
            pick_result: &mut SelectBasicsPickResult,
        ) -> Result<bool, Failure>
    );
    delegate_volume_operation!(
        overlaps_sphere(
            center: &Point3,
            radius: f64,
            pick_result: &mut SelectBasicsPickResult,
        ) -> Result<bool, Failure>
    );
    delegate_volume_operation!(
        overlaps_sphere_inside(
            center: &Point3,
            radius: f64,
            inside: Option<&mut bool>,
        ) -> Result<bool, Failure>
    );
    delegate_volume_operation!(
        overlaps_cylinder(
            bottom_rad: f64,
            top_rad: f64,
            height: f64,
            trsf: &Transform3,
            is_hollow: bool,
            pick_result: &mut SelectBasicsPickResult,
        ) -> Result<bool, Failure>
    );
    delegate_volume_operation!(
        overlaps_cylinder_inside(
            bottom_rad: f64,
            top_rad: f64,
            height: f64,
            trsf: &Transform3,
            is_hollow: bool,
            inside: Option<&mut bool>,
        ) -> Result<bool, Failure>
    );
    delegate_volume_operation!(
        overlaps_circle(
            radius: f64,
            trsf: &Transform3,
            is_filled: bool,
            pick_result: &mut SelectBasicsPickResult,
        ) -> Result<bool, Failure>
    );
    delegate_volume_operation!(
        overlaps_circle_inside(
            radius: f64,
            trsf: &Transform3,
            is_filled: bool,
            inside: Option<&mut bool>,
        ) -> Result<bool, Failure>
    );
    delegate_volume_operation!(dist_to_geometry_center(cog: &Point3) -> Result<f64, Failure>);
    delegate_volume_operation!(detected_point(depth: f64) -> Result<Point3, Failure>);
    fn is_overlap_allowed(&self) -> bool {
        // Native base IsOverlapAllowed calls the derived GetActiveSelectionType when box overlap is disabled.
        self.get_active_selection_type() == SelectMgrSelectionType::Point
    }
    delegate_volume_operation!(get_near_picked_pnt() -> Result<Point3, Failure>);
    delegate_volume_operation!(get_far_picked_pnt() -> Result<Point3, Failure>);
    delegate_volume_operation!(get_view_ray_direction() -> Result<Direction3, Failure>);
    delegate_volume_operation!(is_scalable_active_volume() -> bool);
    delegate_volume_operation!(get_mouse_position() -> Point2);
    delegate_volume_operation!(get_planes(plane_equations: &mut DynamicArray<Vec4<f64>>));
}
