//! A named virtual-volume fake preserves checked failure and callback ordering.
use super::selector_volume_delegation::delegate_volume_operation;
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select_mgr::{SelectMgrSelectingVolumeManager, SelectMgrSelectionType};
use rustycad_tkernel::ncollection::{Array1, DynamicArray, Vec3, Vec4};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Direction3, Point2, Point3, Transform3};
use std::cell::Cell;

pub(super) struct QuadricFaultVolumeFake {
    inner: SelectMgrSelectingVolumeManager,
    pub(super) stage: Cell<i32>,
    pub(super) trace: Cell<i32>,
}
impl QuadricFaultVolumeFake {
    pub(super) fn new(stage: i32) -> Self {
        Self {
            inner: SelectMgrSelectingVolumeManager::default(),
            stage: Cell::new(stage),
            trace: Cell::new(0),
        }
    }
    fn record_callback(&self, value: i32) {
        // Decimal traces expose the exact virtual dispatch order in native/Rust replay.
        self.trace.set(self.trace.get() * 10 + value);
    }
    fn fail_quadric_stage(&self, stage: i32, expected: &str) -> Result<(), Failure> {
        if self.stage.get() == stage {
            return Err(Failure::with_message(
                "Standard_Failure",
                &format!("quadric callback stage {stage}: expected successful {expected}"),
            ));
        }
        Ok(())
    }
    fn pick_quadric_callback(&self, pick: &mut SelectBasicsPickResult) -> Result<bool, Failure> {
        self.record_callback(3);
        pick.set_depth(7.);
        pick.set_picked_point(Point3::new(8., 9., 10.));
        pick.set_surface_normal(Vec3::new(11., 12., 13.));
        self.fail_quadric_stage(1, "point overlap")?;
        Ok(true)
    }
    fn inside_quadric_callback(&self, inside: Option<&mut bool>) -> Result<bool, Failure> {
        self.record_callback(4);
        if let Some(inside) = inside {
            *inside = false;
        }
        self.fail_quadric_stage(3, "inside overlap")?;
        Ok(true)
    }
}
impl SelectBasicsSelectingVolumeManager for QuadricFaultVolumeFake {
    fn get_active_selection_type(&self) -> SelectMgrSelectionType {
        self.record_callback(1);
        if self.stage.get() <= 2 {
            return SelectMgrSelectionType::Point;
        }
        SelectMgrSelectionType::Box
    }
    fn is_overlap_allowed(&self) -> bool {
        self.record_callback(2);
        self.stage.get() == 5
    }
    fn dist_to_geometry_center(&self, _center: &Point3) -> Result<f64, Failure> {
        self.record_callback(5);
        self.fail_quadric_stage(2, "center distance")?;
        Ok(17.)
    }
    fn overlaps_sphere(
        &self,
        _center: &Point3,
        _radius: f64,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        // Each analytic entry follows the same native virtual callback protocol.
        self.pick_quadric_callback(pick)
    }
    fn overlaps_sphere_inside(
        &self,
        _center: &Point3,
        _radius: f64,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        // Inclusion writes precede checked failure, while nonpoint picks remain untouched.
        self.inside_quadric_callback(inside)
    }
    fn overlaps_circle(
        &self,
        _radius: f64,
        _transform: &Transform3,
        _filled: bool,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        // Each analytic entry follows the same native virtual callback protocol.
        self.pick_quadric_callback(pick)
    }
    fn overlaps_circle_inside(
        &self,
        _radius: f64,
        _transform: &Transform3,
        _filled: bool,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        // Inclusion writes precede checked failure, while nonpoint picks remain untouched.
        self.inside_quadric_callback(inside)
    }
    fn overlaps_cylinder(
        &self,
        _bottom: f64,
        _top: f64,
        _height: f64,
        _transform: &Transform3,
        _hollow: bool,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        // Each analytic entry follows the same native virtual callback protocol.
        self.pick_quadric_callback(pick)
    }
    fn overlaps_cylinder_inside(
        &self,
        _bottom: f64,
        _top: f64,
        _height: f64,
        _transform: &Transform3,
        _hollow: bool,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        // Inclusion writes precede checked failure, while nonpoint picks remain untouched.
        self.inside_quadric_callback(inside)
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
    delegate_volume_operation!(detected_point(depth: f64) -> Result<Point3, Failure>);
    delegate_volume_operation!(get_near_picked_pnt()-> Result<Point3, Failure>);
    delegate_volume_operation!(get_far_picked_pnt()-> Result<Point3, Failure>);
    delegate_volume_operation!(get_view_ray_direction()-> Result<Direction3, Failure>);
    delegate_volume_operation!(is_scalable_active_volume()-> bool);
    delegate_volume_operation!(get_mouse_position()-> Point2);
    delegate_volume_operation!(get_planes(plane_equations: &mut DynamicArray<Vec4<f64>>));
    delegate_volume_operation!(
        overlaps_point(point: &Point3, pick: &mut SelectBasicsPickResult) -> Result<bool, Failure>
    );
    delegate_volume_operation!(overlaps_point_only(point: &Point3) -> Result<bool, Failure>);
}
