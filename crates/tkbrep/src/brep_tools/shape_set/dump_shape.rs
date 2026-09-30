// Rust translation of BRepTools_ShapeSet::DumpGeometry(S, OS)
// (ModelingData/TKBRep/BRepTools/BRepTools_ShapeSet.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The readable geometry entry of one shape in `Dump`.
use super::BRepToolsShapeSet;
use super::regularity::{gcurve_range, print_regularity, uv_points, uv_points2};
use crate::brep::access::{edge_tshape, face_tshape, vertex_tshape};
use crate::brep::{BRepCurveRepresentation, BRepPointRepresentation, BRepTool};
use crate::topods::{self, Shape};
use crate::toptools::ShapeSet;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::poly::mesh_purpose;
use rustycad_tkmath::toploc::Location;

fn put_int(stream: &mut IoStream, value: i32) {
    stream.insert_integer(i64::from(value));
}

/// `" location " << Locations().Index(L)` when `location` is not the identity.
fn dump_location(set: &BRepToolsShapeSet, location: &Location, stream: &mut IoStream) {
    if !location.is_identity() {
        stream.insert_bytes(b" location ");
        put_int(stream, set.locations().index(location));
    }
}

/// `"  UV Points : " << Pf.X() << ", " << Pf.Y() << " " << Pl.X() << ", " << Pl.Y() << "\n"`.
fn dump_uv_points((first, last): (Point2, Point2), stream: &mut IoStream) {
    stream.insert_bytes(b"  UV Points : ");
    stream.insert_real(first.x()).insert_bytes(b", ");
    stream.insert_real(first.y()).insert_bytes(b" ");
    stream.insert_real(last.x()).insert_bytes(b", ");
    stream.insert_real(last.y()).insert_bytes(b"\n");
}

impl BRepToolsShapeSet {
    /// `DumpGeometry(S, OS)`: the readable geometry of a vertex, an edge or a face,
    /// ended by an empty line for every shape.
    pub(super) fn dump_geometry_of(
        &self,
        shape: &Shape,
        stream: &mut IoStream,
    ) -> Result<(), Failure> {
        match shape.shape_type()? {
            ShapeEnum::Vertex => self.dump_vertex(shape, stream)?,
            ShapeEnum::Edge => self.dump_edge(shape, stream)?,
            ShapeEnum::Face => self.dump_face(shape, stream)?,
            _ => {}
        }
        stream.insert_bytes(b"\n");
        Ok(())
    }

    fn dump_vertex(&self, shape: &Shape, stream: &mut IoStream) -> Result<(), Failure> {
        let vertex = topods::vertex(shape)?;
        stream.insert_bytes(b"    Tolerance : ");
        stream
            .insert_real(BRepTool::vertex_tolerance(&vertex)?)
            .insert_bytes(b"\n");
        let point = BRepTool::pnt(&vertex)?;
        stream.insert_bytes(b"    - Point 3D : ");
        stream.insert_real(point.x()).insert_bytes(b", ");
        stream.insert_real(point.y()).insert_bytes(b", ");
        stream.insert_real(point.z()).insert_bytes(b"\n");
        let tvertex = vertex_tshape(shape, "BRepTools_ShapeSet::DumpGeometry")?;
        for point in tvertex.points() {
            let Some(point) = point.get() else { continue };
            self.dump_point_representation(point, stream)?;
        }
        Ok(())
    }

    fn dump_point_representation(
        &self,
        point: &dyn BRepPointRepresentation,
        stream: &mut IoStream,
    ) -> Result<(), Failure> {
        stream.insert_bytes(b"    - Parameter : ");
        stream.insert_real(point.parameter());
        if point.is_point_on_curve() {
            stream.insert_bytes(b" on curve ");
            put_int(stream, self.curves.index(&point.curve()?));
        } else if point.is_point_on_curve_on_surface() {
            stream.insert_bytes(b" on pcurve ");
            put_int(stream, self.curves2d.index(&point.pcurve()?));
            stream.insert_bytes(b" on surface ");
            put_int(stream, self.surfaces.index(&point.surface()?));
        } else if point.is_point_on_surface() {
            stream.insert_bytes(b", ");
            stream.insert_real(point.parameter2()?);
            stream.insert_bytes(b" on surface ");
            put_int(stream, self.surfaces.index(&point.surface()?));
        }
        dump_location(self, &point.location(), stream);
        stream.insert_bytes(b"\n");
        Ok(())
    }

    fn dump_edge(&self, shape: &Shape, stream: &mut IoStream) -> Result<(), Failure> {
        let tedge = edge_tshape(shape, "BRepTools_ShapeSet::DumpGeometry")?;
        stream.insert_bytes(b"    Tolerance : ");
        stream.insert_real(tedge.tolerance()).insert_bytes(b"\n");
        if tedge.same_parameter() {
            stream.insert_bytes(b"     same parametrisation of curves\n");
        }
        if tedge.same_range() {
            stream.insert_bytes(b"     same range on curves\n");
        }
        if tedge.degenerated() {
            stream.insert_bytes(b"     degenerated\n");
        }
        for representation in tedge.curves() {
            self.dump_curve_representation(&representation, stream)?;
        }
        Ok(())
    }

    fn dump_curve_representation(
        &self,
        representation: &Handle<dyn BRepCurveRepresentation>,
        stream: &mut IoStream,
    ) -> Result<(), Failure> {
        let Some(curve) = representation.get() else {
            return Ok(());
        };
        if curve.is_curve3d() {
            return self.dump_curve3d(representation, curve, stream);
        }
        if curve.is_curve_on_surface() {
            return self.dump_curve_on_surface(representation, curve, stream);
        }
        if curve.is_regularity() {
            return self.dump_regularity(curve, stream);
        }
        if curve.is_polygon3d() {
            return self.dump_polygon3d_entry(curve, stream);
        }
        if curve.is_polygon_on_triangulation() {
            return self.dump_polygon_on_triangulation_entry(curve, stream);
        }
        Ok(())
    }

    /// The range is read before the null-curve test, as natively.
    fn dump_curve3d(
        &self,
        representation: &Handle<dyn BRepCurveRepresentation>,
        curve: &dyn BRepCurveRepresentation,
        stream: &mut IoStream,
    ) -> Result<(), Failure> {
        let (first, last) = gcurve_range(representation)?;
        let curve3d = curve.curve3d()?;
        if curve3d.is_null() {
            return Ok(());
        }
        stream.insert_bytes(b"    - Curve 3D : ");
        put_int(stream, self.curves.index(&curve3d));
        dump_location(self, &curve.location(), stream);
        dump_range(first, last, stream);
        Ok(())
    }

    fn dump_curve_on_surface(
        &self,
        representation: &Handle<dyn BRepCurveRepresentation>,
        curve: &dyn BRepCurveRepresentation,
        stream: &mut IoStream,
    ) -> Result<(), Failure> {
        let (first, last) = gcurve_range(representation)?;
        let closed = curve.is_curve_on_closed_surface();
        stream.insert_bytes(b"    - PCurve : ");
        put_int(stream, self.curves2d.index(&curve.pcurve()?));
        if closed {
            stream.insert_bytes(b", ");
            put_int(stream, self.curves2d.index(&curve.pcurve2()?));
            stream.insert_bytes(b" (");
            print_regularity(curve.continuity()?, stream);
            stream.insert_bytes(b")");
        }
        stream.insert_bytes(b" on surface ");
        put_int(stream, self.surfaces.index(&curve.surface()?));
        dump_location(self, &curve.location(), stream);
        dump_range(first, last, stream);
        dump_uv_points(uv_points(representation), stream);
        if closed {
            dump_uv_points(uv_points2(representation), stream);
        }
        Ok(())
    }

    fn dump_regularity(
        &self,
        curve: &dyn BRepCurveRepresentation,
        stream: &mut IoStream,
    ) -> Result<(), Failure> {
        stream.insert_bytes(b"    - Regularity ");
        print_regularity(curve.continuity()?, stream);
        stream.insert_bytes(b"   on surfaces : ");
        put_int(stream, self.surfaces.index(&curve.surface()?));
        dump_location(self, &curve.location(), stream);
        stream.insert_bytes(b", ");
        put_int(stream, self.surfaces.index(&curve.surface2()?));
        dump_location(self, &curve.location2()?, stream);
        stream.insert_bytes(b"\n");
        Ok(())
    }

    /// The native entry has no line end.
    fn dump_polygon3d_entry(
        &self,
        curve: &dyn BRepCurveRepresentation,
        stream: &mut IoStream,
    ) -> Result<(), Failure> {
        let polygon = curve.polygon3d()?;
        if polygon.is_null() {
            return Ok(());
        }
        stream.insert_bytes(b"    - Polygon 3D : ");
        put_int(stream, self.polygons3d.index(&polygon));
        dump_location(self, &curve.location(), stream);
        Ok(())
    }

    fn dump_polygon_on_triangulation_entry(
        &self,
        curve: &dyn BRepCurveRepresentation,
        stream: &mut IoStream,
    ) -> Result<(), Failure> {
        stream.insert_bytes(b"    - PolygonOnTriangulation ");
        put_int(stream, self.nodes.index(&curve.polygon_on_triangulation()?));
        if curve.is_polygon_on_closed_triangulation() {
            stream.insert_bytes(b" ");
            put_int(
                stream,
                self.nodes.index(&curve.polygon_on_triangulation2()?),
            );
        }
        stream.insert_bytes(b" on triangulation ");
        put_int(stream, self.triangulations.index(&curve.triangulation()?));
        dump_location(self, &curve.location(), stream);
        stream.insert_bytes(b"\n");
        Ok(())
    }

    /// The natural restriction, the surface and the triangulation of a face; the
    /// location tests read the shape's own location, as natively.
    fn dump_face(&self, shape: &Shape, stream: &mut IoStream) -> Result<(), Failure> {
        let face = topods::face(shape)?;
        if BRepTool::natural_restriction(&face)? {
            stream.insert_bytes(b"NaturalRestriction\n");
        }
        let tface = face_tshape(shape, "BRepTools_ShapeSet::DumpGeometry")?;
        let surface = tface.surface();
        if !surface.is_null() {
            stream.insert_bytes(b"    Tolerance : ");
            stream.insert_real(tface.tolerance()).insert_bytes(b"\n");
            stream.insert_bytes(b"    - Surface : ");
            put_int(stream, self.surfaces.index(&surface));
            dump_location(self, shape.location(), stream);
            stream.insert_bytes(b"\n");
        }
        let triangulation = tface.triangulation(mesh_purpose::NONE);
        if !triangulation.is_null() {
            stream.insert_bytes(b"    - Triangulation : ");
            put_int(stream, self.triangulations.index(&triangulation));
            if !shape.location().is_identity() {
                stream.insert_bytes(b" location ");
                put_int(stream, self.locations().index(&tface.location()));
            }
            stream.insert_bytes(b"\n");
        }
        Ok(())
    }
}

/// `", range : " << first << " " << last << "\n"`.
fn dump_range(first: f64, last: f64, stream: &mut IoStream) {
    stream.insert_bytes(b", range : ");
    stream.insert_real(first);
    stream.insert_bytes(b" ");
    stream.insert_real(last);
    stream.insert_bytes(b"\n");
}
