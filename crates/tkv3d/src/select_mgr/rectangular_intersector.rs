//! Rectangular selecting-volume virtual methods at OCCT 3d097a0.
use super::*;
use crate::select_basics::SelectBasicsPickResult;
use rustycad_tkernel::{
    ncollection::{DynamicArray, Vec3, Vec4},
    standard::{Failure, Handle, dump::DumpStream},
};
use rustycad_tkmath::gp::{Direction3, GeneralTransform3, Point2, Point3, Transform3, Xy};
use rustycad_tkservice::graphic3d::Graphic3dCamera;
use std::io;

#[allow(clippy::too_many_arguments)]
impl SelectMgrBaseIntersector for SelectMgrRectangularFrustum {
    fn intersector_base(&self) -> &SelectMgrBaseIntersectorBase {
        &self.base.intersector
    }
    fn build(&self) -> Result<(), Failure> {
        self.rectangular_build()
    }
    fn is_scalable(&self) -> bool {
        self.get_selection_type() == SelectMgrSelectionType::Point
    }
    fn scale_and_transform(
        &self,
        scale: i32,
        transform: &GeneralTransform3,
        builder: &Handle<SelectMgrFrustumBuilder>,
    ) -> Result<Handle<dyn SelectMgrBaseIntersector>, Failure> {
        Ok(Handle::new(self.rectangular_transformed(scale, transform, builder)?).cast())
    }
    fn copy_with_builder(
        &self,
        builder: &Handle<SelectMgrFrustumBuilder>,
    ) -> Result<Handle<dyn SelectMgrBaseIntersector>, Failure> {
        Ok(Handle::new(self.rectangular_copied(builder)?).cast())
    }
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
    fn get_planes(&self, planes: &mut DynamicArray<Vec4<f64>>) {
        planes.clear(false);
        let core = self.frustum_core.read();
        for index in 0..6 {
            let normal = if self.is_orthographic() && index % 2 == 1 {
                core.planes[index - 1].reversed()
            } else {
                core.planes[index]
            };
            let vertex = core.vertices[if index % 2 == 0 { index } else { index + 2 }];
            planes.append(Vec4::new(
                normal.x(),
                normal.y(),
                normal.z(),
                -normal.dot(vertex.xyz()),
            ));
        }
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        self.rectangular_dump(stream, depth)
    }
    fn overlaps_box_pick(
        &self,
        min: &Vec3<f64>,
        max: &Vec3<f64>,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.rectangular_box_pick(min, max, clip, pick)
    }
    fn overlaps_box(
        &self,
        min: &Vec3<f64>,
        max: &Vec3<f64>,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        self.rectangular_initialized("OverlapsBox")?;
        self.frustum_core
            .read()
            .has_box_overlap(min, max, self.is_orthographic(), inside)
    }
    fn overlaps_point_pick(
        &self,
        point: &Point3,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        if !self.overlaps_point(point)? {
            return Ok(false);
        }
        let ray = *self.ray.read();
        let depth = (point.xyz() - ray.near.xyz()).dot(ray.direction.xyz());
        pick.set_depth(depth.abs() * ray.scale);
        pick.set_picked_point(*point);
        Ok(!clip.is_clipped(pick.depth()))
    }
    fn overlaps_point(&self, point: &Point3) -> Result<bool, Failure> {
        self.rectangular_initialized("OverlapsPoint")?;
        Ok(self
            .frustum_core
            .read()
            .has_point_overlap(point, self.is_orthographic()))
    }
    fn overlaps_polygon(
        &self,
        points: &[Point3],
        sensitivity: i32,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.rectangular_polygon_pick(points, sensitivity, clip, pick)
    }

    fn overlaps_segment(
        &self,
        first: &Point3,
        second: &Point3,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.rectangular_initialized("OverlapsSegment")?;
        if !self
            .frustum_core
            .read()
            .has_segment_overlap(first, second, self.is_orthographic())
        {
            return Ok(false);
        }
        self.segment_segment_distance(first, second, pick);
        Ok(!clip.is_clipped(pick.depth()))
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
        self.rectangular_triangle_pick([*first, *second, *third], sensitivity, clip, pick)
    }
    fn overlaps_sphere(
        &self,
        center: &Point3,
        radius: f64,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        self.rectangular_initialized("OverlapsSphere")?;
        self.frustum_core.read().has_sphere_overlap(
            self,
            *center,
            radius,
            self.is_orthographic(),
            inside,
        )
    }
    fn overlaps_sphere_pick(
        &self,
        center: &Point3,
        radius: f64,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.rectangular_sphere_pick(center, radius, clip, pick)
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
        self.rectangular_cylinder_pick((bottom, top), height, *transform, hollow, clip, pick)
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
        self.rectangular_initialized("OverlapsCylinder")?;
        self.frustum_core.read().has_cylinder_overlap(
            bottom,
            top,
            height,
            *transform,
            hollow,
            self.is_orthographic(),
            inside,
        )
    }
    fn overlaps_circle_pick(
        &self,
        radius: f64,
        transform: &Transform3,
        filled: bool,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.rectangular_circle_pick(radius, *transform, filled, clip, pick)
    }
    fn overlaps_circle(
        &self,
        radius: f64,
        transform: &Transform3,
        filled: bool,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        self.rectangular_initialized("OverlapsCircle")?;
        self.frustum_core.read().has_circle_overlap(
            radius,
            *transform,
            filled,
            self.is_orthographic(),
            inside,
        )
    }
    fn get_near_pnt(&self) -> Result<Point3, Failure> {
        Ok(self.ray.read().near)
    }
    fn get_far_pnt(&self) -> Result<Point3, Failure> {
        Ok(self.ray.read().far)
    }
    fn get_view_ray_direction(&self) -> Result<Direction3, Failure> {
        Ok(self.ray.read().direction)
    }
    fn get_mouse_position(&self) -> Point2 {
        if self.is_scalable() {
            return self.selection_rectangle.read().mouse_pos();
        }
        Point2::from(Xy::new(f64::MAX, f64::MAX))
    }
    fn dist_to_geometry_center(&self, point: &Point3) -> Result<f64, Failure> {
        self.rectangular_initialized("DistToGeometryCenter")?;
        let ray = *self.ray.read();
        Ok(point.distance(&ray.near) * ray.scale)
    }
    fn detected_point(&self, depth: f64) -> Result<Point3, Failure> {
        if !self.is_scalable() {
            return Err(Failure::with_message(
                "Standard_ProgramError",
                &format!(
                    "RectangularFrustum::DetectedPoint selection={:?}; expected Point",
                    self.get_selection_type()
                ),
            ));
        }
        let ray = *self.ray.read();
        Ok(Point3::from(
            ray.near.xyz() + ray.direction.xyz() * depth / ray.scale,
        ))
    }
}
