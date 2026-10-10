//! AxisIntersector virtual bodies, retaining native assertions and output mutations.
use super::axis_quadric::{axis_inside_entry, axis_nearest_depth};
use super::*;
use crate::select_basics::SelectBasicsPickResult;
use rustycad_tkernel::{
    ncollection::Vec3,
    standard::{Failure, Handle, dump::DumpStream},
};
use rustycad_tkmath::gp::{Direction3, GeneralTransform3, Point3, Transform3};
use rustycad_tkservice::{
    graphic3d::Graphic3dCamera,
    json_dump::{base_class, class_begin, field_dumped},
};
use std::io;
#[allow(clippy::too_many_arguments)]
impl SelectMgrBaseIntersector for SelectMgrAxisIntersector {
    fn intersector_base(&self) -> &SelectMgrBaseIntersectorBase {
        &self.base
    }
    fn build(&self) -> Result<(), Failure> {
        Ok(())
    }
    fn set_camera(&self, _camera: &Handle<Graphic3dCamera>) {}
    fn is_scalable(&self) -> bool {
        false
    }
    fn scale_and_transform(
        &self,
        _scale: i32,
        transform: &GeneralTransform3,
        _builder: &Handle<SelectMgrFrustumBuilder>,
    ) -> Result<Handle<dyn SelectMgrBaseIntersector>, Failure> {
        Ok(Handle::new(self.transformed_axis(transform)?).cast())
    }
    fn copy_with_builder(
        &self,
        _builder: &Handle<SelectMgrFrustumBuilder>,
    ) -> Result<Handle<dyn SelectMgrBaseIntersector>, Failure> {
        let axis = self.initialized_axis("CopyWithBuilder")?;
        let result = Self::new();
        result.init(axis);
        Ok(Handle::new(result).cast())
    }
    fn get_near_pnt(&self) -> Result<Point3, Failure> {
        Ok(self.initialized_axis("GetNearPnt")?.location())
    }
    fn get_far_pnt(&self) -> Result<Point3, Failure> {
        self.initialized_axis("GetFarPnt")?;
        Ok(Point3::new(f64::MAX, f64::MAX, f64::MAX))
    }
    fn get_view_ray_direction(&self) -> Result<Direction3, Failure> {
        Ok(self.initialized_axis("GetViewRayDirection")?.direction())
    }
    fn dist_to_geometry_center(&self, point: &Point3) -> Result<f64, Failure> {
        Ok(point.distance(&self.initialized_axis("DistToGeometryCenter")?.location()))
    }
    fn detected_point(&self, depth: f64) -> Result<Point3, Failure> {
        let axis = self.initialized_axis("DetectedPoint")?;
        Ok(Point3::from(
            axis.location().xyz() + axis.direction().xyz() * depth,
        ))
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        // The dump class name is independent of the inherited native runtime descriptor.
        class_begin(stream, "SelectMgr_AxisIntersector")?;
        base_class(stream, depth, &|stream, _| {
            select_mgr_base_intersector_dump_json(&self.base, stream)
        })?;
        field_dumped(stream, depth, "&myAxis", &|stream, _| {
            self.axis.read().dump_json(stream)
        })
    }
    fn overlaps_box(
        &self,
        min: &Vec3<f64>,
        max: &Vec3<f64>,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        self.initialized_axis("OverlapsBox")?;
        Ok(axis_inside_entry(
            self.has_intersection_box(min, max),
            inside,
        ))
    }
    fn overlaps_box_pick(
        &self,
        min: &Vec3<f64>,
        max: &Vec3<f64>,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.initialized_axis("OverlapsBox")?;
        let Some((entry, leave)) = self.has_intersection_box(min, max) else {
            return Ok(false);
        };
        let Some(depth) = axis_nearest_depth(entry, leave, clip)? else {
            return Ok(false);
        };
        pick.set_depth(depth);
        Ok(true)
    }
    fn overlaps_point(&self, point: &Point3) -> Result<bool, Failure> {
        self.initialized_axis("OverlapsPoint")?;
        Ok(self.has_intersection_point(point)?.is_some())
    }
    fn overlaps_point_pick(
        &self,
        point: &Point3,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.initialized_axis("OverlapsPoint")?;
        let Some(depth) = self.has_intersection_point(point)? else {
            return Ok(false);
        };
        pick.set_depth(depth);
        pick.set_picked_point(*point);
        Ok(!clip.is_clipped(depth))
    }
    fn overlaps_segment(
        &self,
        first: &Point3,
        last: &Point3,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.initialized_axis("OverlapsSegment")?;
        Ok(self.ray_segment_distance(first, last, pick) && !clip.is_clipped(pick.depth()))
    }
    fn overlaps_polygon(
        &self,
        points: &[Point3],
        sensitivity: i32,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.axis_polygon_pick(points, sensitivity, clip, pick)
    }
    fn overlaps_triangle(
        &self,
        first: &Point3,
        second: &Point3,
        third: &Point3,
        sensitivity: i32,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.axis_triangle_pick([*first, *second, *third], sensitivity, clip, pick)
    }
    fn overlaps_sphere(
        &self,
        center: &Point3,
        radius: f64,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        let axis = self.initialized_axis("OverlapsSphere")?;
        Ok(axis_inside_entry(
            self.ray_sphere_intersection(center, radius, &axis.location(), &axis.direction()),
            inside,
        ))
    }
    fn overlaps_sphere_pick(
        &self,
        center: &Point3,
        radius: f64,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.axis_sphere_pick(center, radius, clip, pick)
    }
    fn overlaps_cylinder(
        &self,
        bottom: f64,
        top: f64,
        height: f64,
        transform: &Transform3,
        hollow: bool,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        let (_, hit) = self.axis_cylinder_times((bottom, top), height, *transform, hollow)?;
        Ok(axis_inside_entry(hit, inside))
    }
    fn overlaps_cylinder_pick(
        &self,
        bottom: f64,
        top: f64,
        height: f64,
        transform: &Transform3,
        hollow: bool,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.axis_cylinder_pick((bottom, top), height, *transform, hollow, clip, pick)
    }
    fn overlaps_circle(
        &self,
        radius: f64,
        transform: &Transform3,
        filled: bool,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        let (_, time) = self.axis_circle_time(radius, *transform, filled)?;
        Ok(axis_inside_entry(time.map(|value| (value, value)), inside))
    }
    fn overlaps_circle_pick(
        &self,
        radius: f64,
        transform: &Transform3,
        filled: bool,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.axis_circle_pick(radius, *transform, filled, clip, pick)
    }
}
