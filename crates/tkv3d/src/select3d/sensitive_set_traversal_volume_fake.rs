//! A named selector fake exercises callbacks that mutate selection policy.
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select_mgr::{SelectMgrSelectingVolumeManager, SelectMgrSelectionType};
use rustycad_tkernel::ncollection::{Array1, DynamicArray, Vec3, Vec4};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Direction3, Point2, Point3, Transform3};
use std::cell::{Cell, RefCell};

pub(super) struct ChangingPolicyVolume {
    inner: SelectMgrSelectingVolumeManager,
    overlap: Cell<bool>,
    switch_policy: bool,
    pub(super) calls: RefCell<Vec<&'static str>>,
    pub(super) incoming_depths: RefCell<Vec<f64>>,
}
impl ChangingPolicyVolume {
    pub(super) fn new(switch_policy: bool) -> Self {
        Self {
            inner: super::super::super::primitive_array_matching_tests::matching_volume(1, false),
            overlap: Cell::new(true),
            switch_policy,
            calls: RefCell::new(Vec::new()),
            incoming_depths: RefCell::new(Vec::new()),
        }
    }
}
use super::super::super::selector_volume_delegation::delegate_volume_operation;
impl SelectBasicsSelectingVolumeManager for ChangingPolicyVolume {
    fn is_overlap_allowed(&self) -> bool {
        self.calls.borrow_mut().push("policy");
        self.overlap.get()
    }
    fn overlaps_point(
        &self,
        _point: &Point3,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.calls.borrow_mut().push("overlap");
        self.incoming_depths.borrow_mut().push(pick.depth());
        if self.incoming_depths.borrow().len() == 1 {
            pick.set_depth(5.0);
        }
        if self.switch_policy {
            self.overlap.set(false);
        }
        Ok(true)
    }
    fn overlaps_point_only(&self, _point: &Point3) -> Result<bool, Failure> {
        self.calls.borrow_mut().push("inside");
        Ok(true)
    }
    delegate_volume_operation!(get_active_selection_type()-> SelectMgrSelectionType);
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
    delegate_volume_operation!(get_near_picked_pnt()-> Result<Point3, Failure>);
    delegate_volume_operation!(get_far_picked_pnt()-> Result<Point3, Failure>);
    delegate_volume_operation!(get_view_ray_direction()-> Result<Direction3, Failure>);
    delegate_volume_operation!(is_scalable_active_volume()-> bool);
    delegate_volume_operation!(get_mouse_position()-> Point2);
    delegate_volume_operation!(get_planes(plane_equations: &mut DynamicArray<Vec4<f64>>));
}
