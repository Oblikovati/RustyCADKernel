// Rust translation of the geometry part (the `try` body) of
// BinTools_ShapeWriter::WriteShape (BinTools_ShapeWriter.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The geometry of one new shape record: every curve, surface, polygon, triangulation
//! and location is written inline or as a back reference.
use super::ostream::BinOStream;
use super::shape_set_base::BinShapeSetBase;
use super::shape_set_write_shape::gcurve_range;
use super::shape_writer::BinShapeWriter;
use crate::brep::access::{edge_tshape, face_tshape, vertex_tshape};
use crate::brep::{BRepCurveRepresentation, BRepPointRepresentation, BRepTool};
use crate::topods::{Face, Shape, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;

impl BinShapeWriter {
    /// The vertex, edge or face geometry of `shape` (nothing for the other types).
    pub(super) fn write_shape_geometry(
        &mut self,
        out: &mut BinOStream<'_>,
        shape: &Shape,
    ) -> Result<(), Failure> {
        match shape.shape_type()? {
            ShapeEnum::Vertex => self.write_vertex_geometry(out, shape),
            ShapeEnum::Edge => self.write_edge_geometry(out, shape),
            ShapeEnum::Face => self.write_face_geometry(out, shape),
            _ => Ok(()),
        }
    }

    /// The tolerance (at least Precision::Confusion), the point, each point
    /// representation with its location, then 0.
    fn write_vertex_geometry(
        &mut self,
        out: &mut BinOStream<'_>,
        shape: &Shape,
    ) -> Result<(), Failure> {
        let vertex = Vertex::unchecked(shape.clone());
        out.write_real(BRepTool::vertex_tolerance(&vertex)?);
        out.write_point3(BRepTool::pnt(&vertex)?);
        for point in vertex_tshape(shape, "BinTools_ShapeWriter::WriteShape")?.points() {
            let Some(point) = point.get() else { continue };
            self.write_point_record(out, point)?;
            self.write_location(out, &point.location())?;
        }
        out.write_byte(0);
        Ok(())
    }

    /// One point representation; a point on a surface writes its second parameter
    /// first (the reader swaps them back into the wrong order, as natively).
    fn write_point_record(
        &mut self,
        out: &mut BinOStream<'_>,
        point: &dyn BRepPointRepresentation,
    ) -> Result<(), Failure> {
        if point.is_point_on_curve() {
            out.write_byte(1);
            out.write_real(point.parameter());
            self.write_curve(out, &point.curve()?)?;
        } else if point.is_point_on_curve_on_surface() {
            out.write_byte(2);
            out.write_real(point.parameter());
            self.write_curve2d(out, &point.pcurve()?)?;
            self.write_surface(out, &point.surface()?)?;
        } else if point.is_point_on_surface() {
            out.write_byte(3);
            out.write_real(point.parameter2()?);
            out.write_real(point.parameter());
            self.write_surface(out, &point.surface()?)?;
        }
        Ok(())
    }

    /// The raw tolerance, the three edge flags as one byte, each representation, then 0.
    fn write_edge_geometry(
        &mut self,
        out: &mut BinOStream<'_>,
        shape: &Shape,
    ) -> Result<(), Failure> {
        let tedge = edge_tshape(shape, "BinTools_ShapeWriter::WriteShape")?;
        out.write_real(tedge.tolerance());
        out.put_bools3(
            tedge.same_parameter(),
            tedge.same_range(),
            tedge.degenerated(),
        );
        for representation in tedge.curves() {
            self.write_curve_record(out, &representation)?;
        }
        out.write_byte(0);
        Ok(())
    }

    fn write_curve_record(
        &mut self,
        out: &mut BinOStream<'_>,
        representation: &Handle<dyn BRepCurveRepresentation>,
    ) -> Result<(), Failure> {
        let Some(curve) = representation.get() else {
            return Ok(());
        };
        if curve.is_curve3d() {
            self.write_curve3d_record(out, representation)
        } else if curve.is_curve_on_surface() {
            self.write_pcurve_record(out, representation)
        } else if curve.is_regularity() {
            out.write_byte(4);
            out.write_byte(curve.continuity()? as u8);
            self.write_surface(out, &curve.surface()?)?;
            self.write_location(out, &curve.location())?;
            self.write_surface(out, &curve.surface2()?)?;
            self.write_location(out, &curve.location2()?)
        } else if self.is_with_triangles() {
            self.write_mesh_record(out, curve)
        } else {
            Ok(())
        }
    }

    /// `1`, the curve, the location and the range, for a non-null 3D curve.
    fn write_curve3d_record(
        &mut self,
        out: &mut BinOStream<'_>,
        representation: &Handle<dyn BRepCurveRepresentation>,
    ) -> Result<(), Failure> {
        let curve = representation.curve3d()?;
        if curve.is_null() {
            return Ok(());
        }
        let (first, last) = gcurve_range(representation)?;
        out.write_byte(1);
        self.write_curve(out, &curve)?;
        self.write_location(out, &representation.location())?;
        out.write_real(first);
        out.write_real(last);
        Ok(())
    }

    /// `2` (or `3` with the second p-curve and the continuity), the p-curve, the
    /// surface, the location and the range; this format never stores UV points.
    fn write_pcurve_record(
        &mut self,
        out: &mut BinOStream<'_>,
        representation: &Handle<dyn BRepCurveRepresentation>,
    ) -> Result<(), Failure> {
        let (first, last) = gcurve_range(representation)?;
        let closed = representation.is_curve_on_closed_surface();
        out.write_byte(if closed { 3 } else { 2 });
        self.write_curve2d(out, &representation.pcurve()?)?;
        if closed {
            self.write_curve2d(out, &representation.pcurve2()?)?;
            out.write_byte(representation.continuity()? as u8);
        }
        self.write_surface(out, &representation.surface()?)?;
        self.write_location(out, &representation.location())?;
        out.write_real(first);
        out.write_real(last);
        Ok(())
    }

    /// With triangles: `5` and a non-null 3D polygon, or `6` (`7` with a second node
    /// array), the node arrays, the triangulation (without normals) and the location.
    fn write_mesh_record(
        &mut self,
        out: &mut BinOStream<'_>,
        curve: &dyn BRepCurveRepresentation,
    ) -> Result<(), Failure> {
        if curve.is_polygon3d() {
            let polygon = curve.polygon3d()?;
            if !polygon.is_null() {
                out.write_byte(5);
                self.write_polygon3d(out, &polygon)?;
                self.write_location(out, &curve.location())?;
            }
        } else if curve.is_polygon_on_triangulation() {
            let closed = curve.is_polygon_on_closed_triangulation();
            out.write_byte(if closed { 7 } else { 6 });
            self.write_polygon_on_triangulation(out, &curve.polygon_on_triangulation()?)?;
            if closed {
                self.write_polygon_on_triangulation(out, &curve.polygon_on_triangulation2()?)?;
            }
            self.write_triangulation(out, &curve.triangulation()?, false)?;
            self.write_location(out, &curve.location())?;
        }
        Ok(())
    }

    /// The natural restriction flag, the raw tolerance, the surface, the location, then
    /// `2` and the triangulation (with its normals when it has them and normals are
    /// asked for or the face has no surface), `1` without one, or `0` when
    /// triangulations are not written.
    fn write_face_geometry(
        &mut self,
        out: &mut BinOStream<'_>,
        shape: &Shape,
    ) -> Result<(), Failure> {
        let tface = face_tshape(shape, "BinTools_ShapeWriter::WriteShape")?;
        out.write_bool(BRepTool::natural_restriction(&Face::unchecked(
            shape.clone(),
        ))?);
        out.write_real(tface.tolerance());
        let surface = tface.surface();
        self.write_surface(out, &surface)?;
        self.write_location(out, &tface.location())?;
        if !self.is_with_triangles() && !surface.is_null() {
            out.write_byte(0);
            return Ok(());
        }
        let triangulation = tface.active_triangulation();
        let Some(mesh) = triangulation.get() else {
            out.write_byte(1);
            return Ok(());
        };
        let normals = mesh.has_normals() && (self.is_with_normals() || surface.is_null());
        out.write_byte(2);
        self.write_triangulation(out, &triangulation, normals)
    }
}
