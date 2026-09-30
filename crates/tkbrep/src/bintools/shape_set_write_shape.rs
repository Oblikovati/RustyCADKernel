// Rust translation of BinTools_ShapeSet::WriteShape (BinTools_ShapeSet.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The geometry record of one shape in the `TShapes` table: vertex point and point
//! representations, edge flags and curve representations, face surface and
//! triangulation, each written as indices into the geometry tables.
use super::errors::wrap_failure;
use super::format_version;
use super::scalars::{put_bool, put_byte, put_integer, put_real};
use super::shape_set::BinShapeSet;
use super::shape_set_base::{BinShapeSetBase, put_point};
use crate::brep::access::{edge_tshape, face_tshape, vertex_tshape};
use crate::brep::update_curves::as_gcurve;
use crate::brep::{
    BRepCurveOnClosedSurface, BRepCurveOnSurface, BRepCurveRepresentation, BRepPointRepresentation,
    BRepTool,
};
use crate::topods::{Face, Shape, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg3d::top_abs::ShapeEnum;

impl BinShapeSet {
    /// `WriteShape(S, OS)`: the geometry of a vertex, an edge or a face (nothing for the
    /// other types). Every failure is wrapped in "EXCEPTION in
    /// BinTools_ShapeSet::WriteGeometry(S,OS)"; `set.write_shape_geometry(&shape, &mut stream)?`.
    pub fn write_shape_geometry(
        &self,
        shape: &Shape,
        stream: &mut IoStream,
    ) -> Result<(), Failure> {
        wrap_failure(
            "EXCEPTION in BinTools_ShapeSet::WriteGeometry(S,OS)",
            || match shape.shape_type()? {
                ShapeEnum::Vertex => self.write_vertex_geometry(shape, stream),
                ShapeEnum::Edge => self.write_edge_geometry(shape, stream),
                ShapeEnum::Face => self.write_face_geometry(shape, stream),
                _ => Ok(()),
            },
        )
    }

    /// The tolerance (at least Precision::Confusion), the point, each point
    /// representation (type byte, parameters, geometry indices, location index), then 0.
    fn write_vertex_geometry(&self, shape: &Shape, stream: &mut IoStream) -> Result<(), Failure> {
        let vertex = Vertex::unchecked(shape.clone());
        put_real(stream, BRepTool::vertex_tolerance(&vertex)?);
        put_point(stream, BRepTool::pnt(&vertex)?);
        for point in vertex_tshape(shape, "BinTools_ShapeSet::WriteShape")?.points() {
            let Some(point) = point.get() else { continue };
            self.write_point_record(point, stream)?;
            put_integer(stream, self.locations.index(&point.location()));
        }
        put_byte(stream, 0);
        Ok(())
    }

    /// One point representation. A point on a surface writes its second parameter
    /// first, and the reader takes the first value read as the first parameter, so the
    /// two parameters are swapped by a round trip, as natively.
    fn write_point_record(
        &self,
        point: &dyn BRepPointRepresentation,
        stream: &mut IoStream,
    ) -> Result<(), Failure> {
        if point.is_point_on_curve() {
            put_byte(stream, 1);
            put_real(stream, point.parameter());
            put_integer(stream, self.curves.index(&point.curve()?));
        } else if point.is_point_on_curve_on_surface() {
            put_byte(stream, 2);
            put_real(stream, point.parameter());
            put_integer(stream, self.curves2d.index(&point.pcurve()?));
            put_integer(stream, self.surfaces.index(&point.surface()?));
        } else if point.is_point_on_surface() {
            put_byte(stream, 3);
            put_real(stream, point.parameter2()?);
            put_real(stream, point.parameter());
            put_integer(stream, self.surfaces.index(&point.surface()?));
        }
        Ok(())
    }

    /// The raw tolerance, the SameParameter, SameRange and Degenerated flags, each
    /// representation, then 0.
    fn write_edge_geometry(&self, shape: &Shape, stream: &mut IoStream) -> Result<(), Failure> {
        let tedge = edge_tshape(shape, "BinTools_ShapeSet::WriteShape")?;
        put_real(stream, tedge.tolerance());
        put_bool(stream, tedge.same_parameter());
        put_bool(stream, tedge.same_range());
        put_bool(stream, tedge.degenerated());
        for representation in tedge.curves() {
            self.write_curve_record(&representation, stream)?;
        }
        put_byte(stream, 0);
        Ok(())
    }

    fn write_curve_record(
        &self,
        representation: &Handle<dyn BRepCurveRepresentation>,
        stream: &mut IoStream,
    ) -> Result<(), Failure> {
        let Some(curve) = representation.get() else {
            return Ok(());
        };
        if curve.is_curve3d() {
            self.write_curve3d_record(representation, stream)
        } else if curve.is_curve_on_surface() {
            self.write_pcurve_record(representation, stream)
        } else if curve.is_regularity() {
            put_byte(stream, 4);
            put_byte(stream, curve.continuity()? as u8);
            put_integer(stream, self.surfaces.index(&curve.surface()?));
            put_integer(stream, self.locations.index(&curve.location()));
            put_integer(stream, self.surfaces.index(&curve.surface2()?));
            put_integer(stream, self.locations.index(&curve.location2()?));
            Ok(())
        } else if self.is_with_triangles() {
            self.write_mesh_record(curve, stream)
        } else {
            Ok(())
        }
    }

    /// `1`, the curve and location indices and the range, for a non-null 3D curve.
    fn write_curve3d_record(
        &self,
        representation: &Handle<dyn BRepCurveRepresentation>,
        stream: &mut IoStream,
    ) -> Result<(), Failure> {
        let curve = representation.curve3d()?;
        if curve.is_null() {
            return Ok(());
        }
        let (first, last) = gcurve_range(representation)?;
        put_byte(stream, 1);
        put_integer(stream, self.curves.index(&curve));
        put_integer(stream, self.locations.index(&representation.location()));
        put_real(stream, first);
        put_real(stream, last);
        Ok(())
    }

    /// `2` (or `3` for a seam, with the second p-curve and the continuity), the p-curve,
    /// surface and location indices, the range and, for versions 2 and 3, the UV points
    /// (of the second p-curve for a seam).
    fn write_pcurve_record(
        &self,
        representation: &Handle<dyn BRepCurveRepresentation>,
        stream: &mut IoStream,
    ) -> Result<(), Failure> {
        let (first, last) = gcurve_range(representation)?;
        let closed = representation.is_curve_on_closed_surface();
        put_byte(stream, if closed { 3 } else { 2 });
        put_integer(stream, self.curves2d.index(&representation.pcurve()?));
        if closed {
            put_integer(stream, self.curves2d.index(&representation.pcurve2()?));
            put_byte(stream, representation.continuity()? as u8);
        }
        put_integer(stream, self.surfaces.index(&representation.surface()?));
        put_integer(stream, self.locations.index(&representation.location()));
        put_real(stream, first);
        put_real(stream, last);
        if matches!(
            self.format_nb(),
            format_version::VERSION_2 | format_version::VERSION_3
        ) {
            let (uv_first, uv_last) = stored_uv_points(representation, closed);
            for value in [uv_first.x(), uv_first.y(), uv_last.x(), uv_last.y()] {
                put_real(stream, value);
            }
        }
        Ok(())
    }

    /// The polygons written with triangles: `5` for a non-null 3D polygon, `6` (or `7`
    /// with a second node array) for a polygon on a triangulation.
    fn write_mesh_record(
        &self,
        curve: &dyn BRepCurveRepresentation,
        stream: &mut IoStream,
    ) -> Result<(), Failure> {
        if curve.is_polygon3d() {
            let polygon = curve.polygon3d()?;
            if !polygon.is_null() {
                put_byte(stream, 5);
                put_integer(stream, self.polygons3d.index(&polygon));
                put_integer(stream, self.locations.index(&curve.location()));
            }
        } else if curve.is_polygon_on_triangulation() {
            let closed = curve.is_polygon_on_closed_triangulation();
            put_byte(stream, if closed { 7 } else { 6 });
            put_integer(stream, self.nodes.index(&curve.polygon_on_triangulation()?));
            if closed {
                put_integer(
                    stream,
                    self.nodes.index(&curve.polygon_on_triangulation2()?),
                );
            }
            put_integer(stream, self.triangulations.index(&curve.triangulation()?));
            put_integer(stream, self.locations.index(&curve.location()));
        }
        Ok(())
    }

    /// The natural restriction flag, the raw tolerance, the surface index (0 without a
    /// surface), the location index, then `2` and the triangulation index, `1` without a
    /// triangulation, or `0` when triangulations are not written.
    fn write_face_geometry(&self, shape: &Shape, stream: &mut IoStream) -> Result<(), Failure> {
        let tface = face_tshape(shape, "BinTools_ShapeSet::WriteShape")?;
        let face = Face::unchecked(shape.clone());
        put_bool(stream, BRepTool::natural_restriction(&face)?);
        put_real(stream, tface.tolerance());
        let surface = tface.surface();
        put_integer(
            stream,
            if surface.is_null() {
                0
            } else {
                self.surfaces.index(&surface)
            },
        );
        put_integer(stream, self.locations.index(&tface.location()));
        if !self.is_with_triangles() && !surface.is_null() {
            put_byte(stream, 0);
            return Ok(());
        }
        let triangulation = tface.active_triangulation();
        if triangulation.is_null() {
            put_byte(stream, 1);
        } else {
            put_byte(stream, 2);
            put_integer(stream, self.triangulations.index(&triangulation));
        }
        Ok(())
    }
}

/// `occ::down_cast<BRep_GCurve>(CR)->Range(first, last)`, shared with the shape
/// writer; Standard_NullObject for a representation that is not parametric.
pub(super) fn gcurve_range(
    representation: &Handle<dyn BRepCurveRepresentation>,
) -> Result<(f64, f64), Failure> {
    let gcurve = as_gcurve(representation);
    let gcurve = gcurve.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "BinTools: the representation is not a BRep_GCurve",
            "",
        )
    })?;
    Ok(gcurve.range())
}

/// `UVPoints2` of a seam, else `UVPoints` of the p-curve.
fn stored_uv_points(
    representation: &Handle<dyn BRepCurveRepresentation>,
    closed: bool,
) -> (rustycad_tkmath::gp::Point2, rustycad_tkmath::gp::Point2) {
    if closed {
        let seam = representation.cast::<BRepCurveOnClosedSurface>();
        return seam.get().map(|seam| seam.uv_points2()).unwrap_or_default();
    }
    let pcurve = representation.cast::<BRepCurveOnSurface>();
    pcurve
        .get()
        .map(|pcurve| pcurve.uv_points())
        .unwrap_or_default()
}
