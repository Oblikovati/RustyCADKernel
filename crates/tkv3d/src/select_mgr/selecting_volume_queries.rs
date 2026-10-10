//! Sensitive-entity query interface; empty volumes preserve caller outputs.
use super::*;
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use rustycad_tkernel::{
    ncollection::{Array1, DynamicArray, Vec3, Vec4},
    standard::{Failure, dump::DumpStream},
};
use rustycad_tkmath::gp::{Direction3, Point2, Point3, Transform3, Xy};
use std::io;
impl SelectBasicsSelectingVolumeManager for SelectMgrSelectingVolumeManager {
    fn overlaps_box(
        &self,
        box_min: &Vec3<f64>,
        box_max: &Vec3<f64>,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.active.get().map_or(Ok(false), |volume| {
            volume.overlaps_box_pick(box_min, box_max, &self.clip_range, pick_result)
        })
    }
    fn overlaps_box_inside(
        &self,
        box_min: &Vec3<f64>,
        box_max: &Vec3<f64>,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        self.active.get().map_or(Ok(false), |volume| {
            volume.overlaps_box(box_min, box_max, inside)
        })
    }
    fn overlaps_point(
        &self,
        point: &Point3,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.active.get().map_or(Ok(false), |volume| {
            volume.overlaps_point_pick(point, &self.clip_range, pick_result)
        })
    }
    fn overlaps_point_only(&self, point: &Point3) -> Result<bool, Failure> {
        self.active
            .get()
            .map_or(Ok(false), |volume| volume.overlaps_point(point))
    }
    fn overlaps_polygon(
        &self,
        points: &Array1<'_, Point3>,
        sens_type: i32,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.active.get().map_or(Ok(false), |volume| {
            volume.overlaps_polygon(points.as_slice(), sens_type, &self.clip_range, pick_result)
        })
    }
    fn overlaps_segment(
        &self,
        pt1: &Point3,
        pt2: &Point3,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.active.get().map_or(Ok(false), |volume| {
            volume.overlaps_segment(pt1, pt2, &self.clip_range, pick_result)
        })
    }
    fn overlaps_triangle(
        &self,
        pt1: &Point3,
        pt2: &Point3,
        pt3: &Point3,
        sens_type: i32,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.active.get().map_or(Ok(false), |volume| {
            volume.overlaps_triangle(pt1, pt2, pt3, sens_type, &self.clip_range, pick_result)
        })
    }
    fn overlaps_sphere(
        &self,
        center: &Point3,
        radius: f64,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.active.get().map_or(Ok(false), |volume| {
            volume.overlaps_sphere_pick(center, radius, &self.clip_range, pick_result)
        })
    }
    fn overlaps_sphere_inside(
        &self,
        center: &Point3,
        radius: f64,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        self.active.get().map_or(Ok(false), |volume| {
            volume.overlaps_sphere(center, radius, inside)
        })
    }
    fn overlaps_cylinder(
        &self,
        bottom_rad: f64,
        top_rad: f64,
        height: f64,
        trsf: &Transform3,
        is_hollow: bool,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.active.get().map_or(Ok(false), |volume| {
            volume.overlaps_cylinder_pick(
                bottom_rad,
                top_rad,
                height,
                trsf,
                is_hollow,
                &self.clip_range,
                pick_result,
            )
        })
    }
    fn overlaps_cylinder_inside(
        &self,
        bottom_rad: f64,
        top_rad: f64,
        height: f64,
        trsf: &Transform3,
        is_hollow: bool,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        self.active.get().map_or(Ok(false), |volume| {
            volume.overlaps_cylinder(bottom_rad, top_rad, height, trsf, is_hollow, inside)
        })
    }
    fn overlaps_circle(
        &self,
        radius: f64,
        trsf: &Transform3,
        is_filled: bool,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.active.get().map_or(Ok(false), |volume| {
            volume.overlaps_circle_pick(radius, trsf, is_filled, &self.clip_range, pick_result)
        })
    }
    fn overlaps_circle_inside(
        &self,
        radius: f64,
        trsf: &Transform3,
        is_filled: bool,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        self.active.get().map_or(Ok(false), |volume| {
            volume.overlaps_circle(radius, trsf, is_filled, inside)
        })
    }
    fn get_active_selection_type(&self) -> SelectMgrSelectionType {
        self.active
            .get()
            .map_or(SelectMgrSelectionType::Unknown, |volume| {
                volume.get_selection_type()
            })
    }
    fn dist_to_geometry_center(&self, cog: &Point3) -> Result<f64, Failure> {
        self.active
            .get()
            .map_or(Ok(f64::MAX), |volume| volume.dist_to_geometry_center(cog))
    }
    fn detected_point(&self, depth: f64) -> Result<Point3, Failure> {
        self.required_volume("DetectedPoint")?.detected_point(depth)
    }
    fn is_overlap_allowed(&self) -> bool {
        self.allow_overlap || self.get_active_selection_type() == SelectMgrSelectionType::Point
    }
    fn get_near_picked_pnt(&self) -> Result<Point3, Failure> {
        self.active
            .get()
            .map_or(Ok(Point3::default()), |volume| volume.get_near_pnt())
    }
    fn get_far_picked_pnt(&self) -> Result<Point3, Failure> {
        self.active
            .get()
            .map_or(Ok(Point3::new(f64::MAX, f64::MAX, f64::MAX)), |volume| {
                volume.get_far_pnt()
            })
    }
    fn get_view_ray_direction(&self) -> Result<Direction3, Failure> {
        self.active
            .get()
            .map_or(Ok(Direction3::X), |volume| volume.get_view_ray_direction())
    }
    fn is_scalable_active_volume(&self) -> bool {
        self.active.get().is_some_and(|volume| volume.is_scalable())
    }
    fn get_mouse_position(&self) -> Point2 {
        self.active
            .get()
            .map_or(Point2::from(Xy::new(f64::MAX, f64::MAX)), |volume| {
                volume.get_mouse_position()
            })
    }
    fn get_planes(&self, planes: &mut DynamicArray<Vec4<f64>>) {
        let Some(volume) = self.active.get() else {
            planes.clear(false);
            return;
        };
        volume.get_planes(planes);
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        self.dump_selecting_volume(stream, depth)
    }
}
