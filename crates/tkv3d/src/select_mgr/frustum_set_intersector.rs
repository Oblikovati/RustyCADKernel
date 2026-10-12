//! Polyline set virtual queries and shared-base configuration at OCCT 3d097a0.
use super::*;
use crate::select_basics::SelectBasicsPickResult;
use rustycad_tkernel::{
    ncollection::{DynamicArray, Vec3, Vec4},
    standard::{Failure, Handle, dump::DumpStream},
};
use rustycad_tkmath::gp::{GeneralTransform3, Point3, Transform3};
use rustycad_tkservice::graphic3d::Graphic3dCamera;
use rustycad_tkservice::json_dump::{base_class, class_begin, field_dumped};
use std::io;

#[allow(clippy::too_many_arguments)]
impl SelectMgrBaseIntersector for SelectMgrTriangularFrustumSet {
    fn intersector_base(&self) -> &SelectMgrBaseIntersectorBase {
        &self.base.intersector
    }
    fn build(&self) -> Result<(), Failure> {
        self.frustum_set_build()
    }
    fn is_scalable(&self) -> bool {
        false
    }
    fn scale_and_transform(
        &self,
        scale: i32,
        transform: &GeneralTransform3,
        builder: &Handle<SelectMgrFrustumBuilder>,
    ) -> Result<Handle<dyn SelectMgrBaseIntersector>, Failure> {
        Ok(Handle::new(self.frustum_set_transformed(scale, transform, builder)?).cast())
    }
    fn copy_with_builder(
        &self,
        builder: &Handle<SelectMgrFrustumBuilder>,
    ) -> Result<Handle<dyn SelectMgrBaseIntersector>, Failure> {
        Ok(Handle::new(self.frustum_set_copied(builder)?).cast())
    }
    fn set_camera(
        &self,
        camera: &dyn rustycad_tkernel::standard::HandleReference<Graphic3dCamera>,
    ) {
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
    fn get_planes(&self, planes: &mut DynamicArray<Vec4<f64>>) {
        planes.clear(false);
        for child in self.frustums.read().iter() {
            child.get_planes(planes);
        }
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "SelectMgr_TriangularFrustumSet")?;
        base_class(stream, depth, &|stream, depth| {
            self.base.dump_json(stream, depth)
        })?;
        for child in self.frustums.read().iter() {
            field_dumped(stream, depth, "aFrustum.get()", &|stream, depth| {
                child.dump_json(stream, depth)
            })?;
        }
        Ok(())
    }
    fn overlaps_box_pick(
        &self,
        min: &Vec3<f64>,
        max: &Vec3<f64>,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.frustum_set_any(|child| child.overlaps_box_pick(min, max, clip, pick))
    }
    fn overlaps_box(
        &self,
        min: &Vec3<f64>,
        max: &Vec3<f64>,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        if !self.frustum_set_any(|child| child.overlaps_box(min, max, None))? {
            return Ok(false);
        }
        if *self.allow_overlap.read() {
            return Ok(true);
        }
        if let Some(inside) = inside {
            self.frustum_set_refine_box(min, max, inside);
        }
        Ok(true)
    }
    fn overlaps_point_pick(
        &self,
        point: &Point3,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.frustum_set_any(|child| child.overlaps_point_pick(point, clip, pick))
    }
    fn overlaps_point(&self, point: &Point3) -> Result<bool, Failure> {
        self.frustum_set_initialized("OverlapsPoint")?;
        Ok(self.frustum_set_contains_point(point))
    }
    fn overlaps_polygon(
        &self,
        points: &[Point3],
        sensitivity: i32,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        if !self.frustum_set_any(|child| child.overlaps_polygon(points, sensitivity, clip, pick))? {
            return Ok(false);
        }
        self.frustum_set_accepts_edges(points)
    }

    fn overlaps_segment(
        &self,
        first: &Point3,
        second: &Point3,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        if !self.frustum_set_any(|child| child.overlaps_segment(first, second, clip, pick))? {
            return Ok(false);
        }
        Ok(*self.allow_overlap.read() || !self.frustum_set_crosses_segment(*first, *second)?)
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
        if !self.frustum_set_any(|child| {
            child.overlaps_triangle(first, second, third, sensitivity, clip, pick)
        })? {
            return Ok(false);
        }
        self.frustum_set_accepts_edges(&[*first, *second, *third])
    }
    fn overlaps_sphere(
        &self,
        center: &Point3,
        radius: f64,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        let _ = inside;
        self.frustum_set_sphere_overlap(*center, radius)
    }
    fn overlaps_sphere_pick(
        &self,
        center: &Point3,
        radius: f64,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.frustum_set_any(|child| child.overlaps_sphere_pick(center, radius, clip, pick))
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
        self.frustum_set_any(|child| {
            child.overlaps_cylinder_pick(bottom, top, height, transform, hollow, clip, pick)
        })
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
        self.frustum_set_cylinder_overlap((bottom, top), height, *transform, hollow, inside)
    }
    fn overlaps_circle_pick(
        &self,
        radius: f64,
        transform: &Transform3,
        filled: bool,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.frustum_set_any(|child| {
            child.overlaps_circle_pick(radius, transform, filled, clip, pick)
        })
    }
    fn overlaps_circle(
        &self,
        radius: f64,
        transform: &Transform3,
        filled: bool,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        self.frustum_set_circle_overlap(radius, *transform, filled, inside)
    }

    fn detected_point(&self, depth: f64) -> Result<Point3, Failure> {
        Err(Failure::with_message(
            "Standard_ProgramError",
            &format!(
                "TriangularFrustumSet::DetectedPoint depth={depth}; expected a Point selecting volume"
            ),
        ))
    }
}

impl SelectMgrTriangularFrustumSet {
    fn frustum_set_refine_box(&self, min: &Vec3<f64>, max: &Vec3<f64>, inside: &mut bool) {
        for index in 0..8 {
            let point = Point3::new(
                if index & 1 == 0 { min.x } else { max.x },
                if index & 2 == 0 { min.y } else { max.y },
                if index & 4 == 0 { min.z } else { max.z },
            );
            if !self.frustum_set_contains_point(&point) {
                *inside &= false;
                break;
            }
        }
    }
}
