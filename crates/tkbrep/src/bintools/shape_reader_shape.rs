// Rust translation of the geometry part (the `try` body) of
// BinTools_ShapeReader::ReadShape (BinTools_ShapeReader.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The geometry of one new shape record of the shape-reference format.
use super::istream::BinIStream;
use super::raw_enums::continuity_from_raw;
use super::shape_reader::BinShapeReader;
use super::shape_set_read_shape::make_topology;
use crate::brep::access::vertex_tshape;
use crate::brep::{
    BRepPointOnCurve, BRepPointOnCurveOnSurface, BRepPointOnSurface, BRepPointRepresentation,
};
use crate::topods::{Edge, Face, Shape, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::toploc::Location;

impl BinShapeReader {
    /// Makes `shape` of type `kind` and reads its geometry.
    pub(super) fn read_shape_geometry(
        &mut self,
        input: &mut BinIStream<'_>,
        kind: ShapeEnum,
        shape: &mut Shape,
    ) -> Result<(), Failure> {
        match kind {
            ShapeEnum::Vertex => self.read_vertex(input, shape),
            ShapeEnum::Edge => self.read_edge(input, shape),
            ShapeEnum::Face => self.read_face(input, shape),
            other => {
                *shape = make_topology(&self.builder, other);
                Ok(())
            }
        }
    }

    /// The tolerance and the point, then point representations while the stream is
    /// good, up to a 0 type byte.
    fn read_vertex(
        &mut self,
        input: &mut BinIStream<'_>,
        shape: &mut Shape,
    ) -> Result<(), Failure> {
        let tolerance = input.read_real()?;
        let point = input.read_pnt()?;
        let mut vertex = Vertex::default();
        self.builder.make_vertex_at(&mut vertex, point, tolerance)?;
        *shape = vertex.clone().into_shape();
        let tvertex = vertex_tshape(&vertex, "BinTools_ShapeReader::ReadShape")?;
        while input.is_ok() {
            let kind = input.read_byte();
            if kind == 0 {
                break;
            }
            let parameter = input.read_real()?;
            let representation = self.read_point_record(input, kind, parameter)?;
            let location = self.read_location(input)?;
            if let Some(representation) = representation {
                representation.set_location(&location);
                tvertex.change_points(|points| points.push(representation));
            }
        }
        Ok(())
    }

    /// One point representation, `None` when its geometry is null. A point on a
    /// p-curve is kept only when its surface is null: the native test reads
    /// `!aCurve2d.IsNull() && aSurface.IsNull()`, so every written one is dropped.
    fn read_point_record(
        &mut self,
        input: &mut BinIStream<'_>,
        kind: u8,
        parameter: f64,
    ) -> Result<Option<Handle<dyn BRepPointRepresentation>>, Failure> {
        let identity = Location::default();
        match kind {
            1 => {
                let curve = self.read_curve(input)?;
                Ok((!curve.is_null()).then(|| {
                    Handle::new(BRepPointOnCurve::new(parameter, &curve, &identity)).into_cast()
                }))
            }
            2 => {
                let pcurve = self.read_curve2d(input)?;
                let surface = self.read_surface(input)?;
                Ok((!pcurve.is_null() && surface.is_null()).then(|| {
                    Handle::new(BRepPointOnCurveOnSurface::new(
                        parameter, &pcurve, &surface, &identity,
                    ))
                    .into_cast()
                }))
            }
            3 => {
                let parameter2 = input.read_real()?;
                let surface = self.read_surface(input)?;
                Ok((!surface.is_null()).then(|| {
                    Handle::new(BRepPointOnSurface::new(
                        parameter, parameter2, &surface, &identity,
                    ))
                    .into_cast()
                }))
            }
            _ => Err(Failure::new(&format!(
                "BinTools_ShapeReader::Read: UnExpected BRep_PointRepresentation = {}\n",
                super::errors::byte_text(kind)
            ))),
        }
    }

    /// The tolerance, the three flags byte, then representations while the stream is
    /// good, up to a 0 type byte.
    fn read_edge(&mut self, input: &mut BinIStream<'_>, shape: &mut Shape) -> Result<(), Failure> {
        let mut edge = Edge::default();
        self.builder.make_edge(&mut edge)?;
        *shape = edge.clone().into_shape();
        let tolerance = input.read_real()?;
        let [same_parameter, same_range, degenerated] = input.read_bools3();
        self.builder.same_parameter(&edge, same_parameter)?;
        self.builder.same_range(&edge, same_range)?;
        self.builder.degenerated(&edge, degenerated)?;
        while input.is_ok() {
            match input.read_byte() {
                0 => break,
                1 => self.read_curve3d_record(input, &edge, tolerance)?,
                kind @ (2 | 3) => self.read_pcurve_record(input, &edge, tolerance, kind == 3)?,
                4 => self.read_regularity_record(input, &edge)?,
                5 => {
                    let polygon = self.read_polygon3d(input)?;
                    let location = self.read_location(input)?;
                    self.builder
                        .update_edge_polygon3d(&edge, &polygon, &location)?;
                }
                kind @ (6 | 7) => self.read_triangulation_record(input, &edge, kind == 7)?,
                other => {
                    return Err(Failure::new(&format!(
                        "Unexpected Curve Representation ={}\n",
                        super::errors::byte_text(other)
                    )));
                }
            }
        }
        Ok(())
    }

    fn read_curve3d_record(
        &mut self,
        input: &mut BinIStream<'_>,
        edge: &Edge,
        tolerance: f64,
    ) -> Result<(), Failure> {
        let curve = self.read_curve(input)?;
        let location = self.read_location(input)?;
        let (first, last) = (input.read_real()?, input.read_real()?);
        if curve.is_null() {
            return Ok(());
        }
        self.builder
            .update_edge_located_curve(edge, &curve, &location, tolerance)?;
        self.builder.range(edge, first, last, true)
    }

    /// `2` or `3` (seam, with the second p-curve and the continuity byte); skipped when
    /// a p-curve or the surface is null.
    fn read_pcurve_record(
        &mut self,
        input: &mut BinIStream<'_>,
        edge: &Edge,
        tolerance: f64,
        closed: bool,
    ) -> Result<(), Failure> {
        let pcurve = self.read_curve2d(input)?;
        let (pcurve2, continuity) = if closed {
            (self.read_curve2d(input)?, input.read_byte())
        } else {
            (Handle::default(), 0)
        };
        let surface = self.read_surface(input)?;
        let location = self.read_location(input)?;
        let (first, last) = (input.read_real()?, input.read_real()?);
        if pcurve.is_null() || (closed && pcurve2.is_null()) || surface.is_null() {
            return Ok(());
        }
        if closed {
            self.builder.update_edge_pcurves(
                edge,
                (&pcurve, &pcurve2),
                &surface,
                &location,
                tolerance,
                None,
            )?;
            let continuity = continuity_from_raw(i32::from(continuity))?;
            self.builder.continuity(
                edge,
                (&surface, &surface),
                (&location, &location),
                continuity,
            )?;
        } else {
            self.builder
                .update_edge_pcurve(edge, &pcurve, &surface, &location, tolerance, None)?;
        }
        self.builder
            .range_on_surface(edge, &surface, &location, first, last)
    }

    fn read_regularity_record(
        &mut self,
        input: &mut BinIStream<'_>,
        edge: &Edge,
    ) -> Result<(), Failure> {
        let continuity = input.read_byte();
        let surface1 = self.read_surface(input)?;
        let location1 = self.read_location(input)?;
        let surface2 = self.read_surface(input)?;
        let location2 = self.read_location(input)?;
        if surface1.is_null() || surface2.is_null() {
            return Ok(());
        }
        let continuity = continuity_from_raw(i32::from(continuity))?;
        self.builder.continuity(
            edge,
            (&surface1, &surface2),
            (&location1, &location2),
            continuity,
        )
    }

    fn read_triangulation_record(
        &mut self,
        input: &mut BinIStream<'_>,
        edge: &Edge,
        closed: bool,
    ) -> Result<(), Failure> {
        let polygon = self.read_polygon_on_triangulation(input)?;
        let polygon2 = if closed {
            self.read_polygon_on_triangulation(input)?
        } else {
            Handle::default()
        };
        let triangulation = self.read_triangulation(input)?;
        let location = self.read_location(input)?;
        if closed {
            self.builder.update_edge_polygons_on_triangulation(
                edge,
                (&polygon, &polygon2),
                &triangulation,
                &location,
            )
        } else {
            self.builder.update_edge_polygon_on_triangulation(
                edge,
                &polygon,
                &triangulation,
                &location,
            )
        }
    }

    /// The natural restriction flag, the tolerance, the surface, the location, then a
    /// triangulation after the byte 2.
    fn read_face(&mut self, input: &mut BinIStream<'_>, shape: &mut Shape) -> Result<(), Failure> {
        let mut face = Face::default();
        self.builder.make_face(&mut face);
        *shape = face.clone().into_shape();
        let restriction = input.read_bool();
        let tolerance = input.read_real()?;
        let surface = self.read_surface(input)?;
        let location = self.read_location(input)?;
        self.builder
            .update_face(&face, &surface, &location, tolerance)?;
        self.builder.natural_restriction(&face, restriction)?;
        if input.read_byte() == 2 {
            let triangulation = self.read_triangulation(input)?;
            self.builder
                .update_face_triangulation(&face, &triangulation, true)?;
        }
        Ok(())
    }
}
