//! Native rectangular dump labels and reusable protected planar predicates.
use super::*;
use rustycad_tkernel::standard::{Failure, dump::DumpStream};
use rustycad_tkmath::gp::{Point3, Transform3, Vector3};
use rustycad_tkservice::json_dump::{base_class, class_begin, field_dumped, field_real};
use std::io;
impl SelectMgrRectangularFrustum {
    /// Test projected boundary edges against a circle; `volume.is_intersect_circle(1., center, transform, &vertices)?`.
    pub fn is_intersect_circle(
        &self,
        radius: f64,
        center: Point3,
        transform: Transform3,
        vertices: &[Point3],
    ) -> Result<bool, Failure> {
        super::frustum_quadric_geometry::frustum_circle_intersection(
            radius,
            center,
            transform,
            vertices,
            Vector3::from(self.frustum_core.read().edge_dirs[4]),
        )
    }
    /// Test coplanar segments using the native projected determinant; `volume.is_segments_intersect(a,b,c,d)`.
    pub fn is_segments_intersect(
        &self,
        first: Point3,
        second: Point3,
        third: Point3,
        fourth: Point3,
    ) -> bool {
        super::frustum_quadric_geometry::frustum_segments_intersect([first, second, third, fourth])
    }
    pub(super) fn rectangular_dump(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "SelectMgr_RectangularFrustum")?;
        base_class(stream, depth, &|stream, depth| {
            self.frustum_core
                .read()
                .dump_json(&self.base, self.is_orthographic(), stream, depth)
        })?;
        self.rectangular_dump_fields(stream, depth)
    }
    fn rectangular_dump_fields(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let ray = *self.ray.read();
        let rectangle = *self.selection_rectangle.read();
        field_dumped(stream, depth, "&myNearPickedPnt", &|stream, _| {
            ray.near.dump_json(stream)
        })?;
        field_dumped(stream, depth, "&myFarPickedPnt", &|stream, _| {
            ray.far.dump_json(stream)
        })?;
        field_dumped(stream, depth, "&myViewRayDir", &|stream, _| {
            ray.direction.dump_json(stream)
        })?;
        field_dumped(stream, depth, "&mySelRectangle.MinPnt()", &|stream, _| {
            rectangle.min_pnt().dump_json(stream)
        })?;
        field_dumped(stream, depth, "&mySelRectangle.MaxPnt()", &|stream, _| {
            rectangle.max_pnt().dump_json(stream)
        })?;
        field_real(stream, "myScale", ray.scale)
    }
}
