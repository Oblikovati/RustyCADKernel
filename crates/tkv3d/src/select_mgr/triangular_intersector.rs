//! Native virtual method bodies for the triangular selecting volume (OCCT 3d097a0).
use super::triangular_geometry::{triangular_build_core, triangular_transformed_core};
use super::*;
use crate::select_basics::SelectBasicsPickResult;
use rustycad_tkernel::{
    ncollection::{DynamicArray, Vec3, Vec4},
    standard::{Failure, Handle, dump::DumpStream},
};
use rustycad_tkmath::gp::{GeneralTransform3, Point3, Transform3, Xyz};
use rustycad_tkservice::{
    graphic3d::Graphic3dCamera,
    json_dump::{base_class, class_begin},
};
use std::io;

#[allow(clippy::too_many_arguments)]
impl SelectMgrBaseIntersector for SelectMgrTriangularFrustum {
    fn intersector_base(&self) -> &SelectMgrBaseIntersectorBase {
        &self.base.intersector
    }
    fn build(&self) -> Result<(), Failure> {
        let builder = self.base.builder.read().clone();
        let builder = builder.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "triangular frustum builder=null; expected a builder before Build",
            )
        })?;
        let triangle = *self.selection_triangle.read();
        *self.frustum_core.write() = triangular_build_core(triangle, builder);
        Ok(())
    }
    fn is_scalable(&self) -> bool {
        false
    }
    fn scale_and_transform(
        &self,
        _scale: i32,
        transform: &GeneralTransform3,
        _builder: &Handle<SelectMgrFrustumBuilder>,
    ) -> Result<Handle<dyn SelectMgrBaseIntersector>, Failure> {
        let result = Self::new();
        result.set_camera(&self.camera());
        *result.frustum_core.write() =
            triangular_transformed_core(&self.frustum_core.read(), transform);
        *result.selection_triangle.write() = *self.selection_triangle.read();
        Ok(Handle::new(result).cast())
    }
    fn copy_with_builder(
        &self,
        builder: &Handle<SelectMgrFrustumBuilder>,
    ) -> Result<Handle<dyn SelectMgrBaseIntersector>, Failure> {
        let result = Self::new();
        *result.selection_triangle.write() = *self.selection_triangle.read();
        result.set_builder(builder);
        Ok(Handle::new(result).cast())
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
        let core = self.frustum_core.read();
        for (index, normal) in core.planes.iter().enumerate() {
            let vertex = core.vertices[if index % 2 == 0 { index } else { 1 }];
            planes.append(Vec4::new(
                normal.x(),
                normal.y(),
                normal.z(),
                -normal.dot(vertex.xyz()),
            ));
        }
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "SelectMgr_TriangularFrustum")?;
        base_class(stream, depth, &|stream, depth| {
            self.frustum_core
                .read()
                .dump_json(&self.base, self.is_orthographic(), stream, depth)
        })
    }
    fn overlaps_box_pick(
        &self,
        min: &Vec3<f64>,
        max: &Vec3<f64>,
        _clip: &SelectMgrViewClipRange,
        _pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.overlaps_box(min, max, None)
    }
    fn overlaps_box(
        &self,
        min: &Vec3<f64>,
        max: &Vec3<f64>,
        _inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        self.frustum_core
            .read()
            .has_box_overlap(min, max, self.is_orthographic(), None)
    }
    fn overlaps_point_pick(
        &self,
        point: &Point3,
        _clip: &SelectMgrViewClipRange,
        _pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        Ok(self
            .frustum_core
            .read()
            .has_point_overlap(point, self.is_orthographic()))
    }
    fn overlaps_point(&self, _point: &Point3) -> Result<bool, Failure> {
        Ok(false)
    }
    fn overlaps_polygon(
        &self,
        points: &[Point3],
        sensitivity: i32,
        _clip: &SelectMgrViewClipRange,
        _pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        let core = self.frustum_core.read();
        if sensitivity == 0 {
            return core.has_polygon_overlap(
                points,
                self.is_orthographic(),
                &mut Xyz::new(f64::MAX, f64::MAX, f64::MAX),
            );
        }
        if sensitivity == 1 {
            return Ok(super::triangular_geometry::triangular_boundary_result(
                &core,
                points,
                self.is_orthographic(),
            ));
        }
        Ok(false)
    }

    fn overlaps_segment(
        &self,
        first: &Point3,
        second: &Point3,
        _clip: &SelectMgrViewClipRange,
        _pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        Ok(self
            .frustum_core
            .read()
            .has_segment_overlap(first, second, self.is_orthographic()))
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
        if sensitivity == 1 {
            return self.overlaps_polygon(&[*first, *second, *third], sensitivity, clip, pick);
        }
        if sensitivity == 0 {
            return Ok(self.frustum_core.read().has_triangle_overlap(
                first,
                second,
                third,
                self.is_orthographic(),
                &mut Xyz::new(f64::MAX, f64::MAX, f64::MAX),
            ));
        }
        Ok(true)
    }
    fn overlaps_sphere(
        &self,
        center: &Point3,
        radius: f64,
        _inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        // The native inside-pointer overload deliberately selects the enclosing box, unlike the pick overload.
        let min = Vec3::new(
            center.x() - radius,
            center.y() - radius,
            center.z() - radius,
        );
        let max = Vec3::new(
            center.x() + radius,
            center.y() + radius,
            center.z() + radius,
        );
        self.overlaps_box(&min, &max, None)
    }
    fn overlaps_sphere_pick(
        &self,
        center: &Point3,
        radius: f64,
        _clip: &SelectMgrViewClipRange,
        _pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.frustum_core.read().has_sphere_overlap(
            self,
            *center,
            radius,
            self.is_orthographic(),
            None,
        )
    }
    fn overlaps_cylinder_pick(
        &self,
        bottom: f64,
        top: f64,
        height: f64,
        transform: &Transform3,
        hollow: bool,
        _clip: &SelectMgrViewClipRange,
        _pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.overlaps_cylinder(bottom, top, height, transform, hollow, None)
    }
    fn overlaps_cylinder(
        &self,
        bottom: f64,
        top: f64,
        height: f64,
        transform: &Transform3,
        hollow: bool,
        _inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        self.frustum_core.read().has_cylinder_overlap(
            bottom,
            top,
            height,
            *transform,
            hollow,
            self.is_orthographic(),
            None,
        )
    }
    fn overlaps_circle_pick(
        &self,
        radius: f64,
        transform: &Transform3,
        filled: bool,
        _clip: &SelectMgrViewClipRange,
        _pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.overlaps_circle(radius, transform, filled, None)
    }
    fn overlaps_circle(
        &self,
        radius: f64,
        transform: &Transform3,
        filled: bool,
        _inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        self.frustum_core.read().has_circle_overlap(
            radius,
            *transform,
            filled,
            self.is_orthographic(),
            None,
        )
    }
}
