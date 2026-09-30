// Rust translation of BinTools_ShapeSet::ReadShape (BinTools_ShapeSet.cxx), except
// the edge case, at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Reading the geometry record of one shape: the vertices and faces here, the edges
//! in `shape_set_read_edge`, the other types have no record.
use super::errors::wrap_failure;
use super::format_version;
use super::raw_enums::shape_enum_from_raw;
use super::scalars::{get_bool, get_integer, get_real, stream_get};
use super::shape_set::BinShapeSet;
use super::shape_set_base::BinShapeSetBase;
use crate::brep::BRepBuilder;
use crate::brep::access::vertex_tshape;
use crate::brep::{
    BRepPointOnCurve, BRepPointOnCurveOnSurface, BRepPointOnSurface, BRepPointRepresentation,
};
use crate::topods::{CompSolid, Compound, Face, Shape, Shell, Solid, Vertex, Wire};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::toploc::Location;

impl BinShapeSet {
    /// `ReadShape(T, IS, S)`: makes `shape` of the type `kind` (the raw byte of the
    /// record) and reads its geometry. A value outside `VERTEX..=COMPOUND` raises
    /// `"Unexpected topology type = <value>"`, and every failure is wrapped in "EXCEPTION
    /// in BinTools_ShapeSet::ReadGeometry(S,OS)";
    /// `set.read_shape_geometry(kind, &mut stream, &mut shape)?`.
    pub fn read_shape_geometry(
        &mut self,
        kind: i32,
        stream: &mut IoStream,
        shape: &mut Shape,
    ) -> Result<(), Failure> {
        wrap_failure("EXCEPTION in BinTools_ShapeSet::ReadGeometry(S,OS)", || {
            match shape_enum_from_raw(kind) {
                Some(ShapeEnum::Vertex) => self.read_vertex(stream, shape),
                Some(ShapeEnum::Edge) => self.read_edge(stream, shape),
                Some(ShapeEnum::Face) => self.read_face(stream, shape),
                Some(ShapeEnum::Shape) | None => Err(Failure::new(&format!(
                    "Unexpected topology type = {kind}\n"
                ))),
                Some(other) => {
                    *shape = make_topology(&self.builder, other);
                    Ok(())
                }
            }
        })
    }

    /// The tolerance and the point, then the point representations up to a 0 type.
    fn read_vertex(&self, stream: &mut IoStream, shape: &mut Shape) -> Result<(), Failure> {
        let tolerance = get_real(stream)?;
        let point = Point3::new(get_real(stream)?, get_real(stream)?, get_real(stream)?);
        let mut vertex = Vertex::default();
        self.builder.make_vertex_at(&mut vertex, point, tolerance)?;
        *shape = vertex.clone().into_shape();
        let tvertex = vertex_tshape(&vertex, "BinTools_ShapeSet::ReadShape")?;
        loop {
            let (kind, parameter) = self.read_point_head(stream)?;
            let representation = self.read_point_record(kind, parameter, stream)?;
            if kind <= 0 {
                return Ok(());
            }
            let location = get_integer(stream)?;
            if let Some(representation) = representation {
                representation.set_location(&self.locations.location(location)?);
                tvertex.change_points(|points| points.push(representation));
            }
        }
    }

    /// The type of the next point representation and its first parameter. Version 3
    /// writes the type first; the other versions first try the pre-6.2.1 layout
    /// (parameter, then type) and go back when the byte after the parameter is not a
    /// type 1 to 3. The writers always use the type-first layout, so a parameter whose
    /// last byte is 1, 2 or 3 (a tiny double) is misread by versions 1, 2 and 4, as
    /// natively.
    fn read_point_head(&self, stream: &mut IoStream) -> Result<(i32, f64), Failure> {
        let mut parameter = 0.0;
        if self.format_nb() == format_version::VERSION_3 {
            let kind = stream_get(stream);
            if (1..=3).contains(&kind) {
                parameter = get_real(stream)?;
            }
            return Ok((kind, parameter));
        }
        let position = stream.tellg();
        parameter = get_real(stream)?;
        let mut kind = stream_get(stream);
        if !(1..=3).contains(&kind) {
            stream.seekg(position);
            kind = stream_get(stream);
            if (1..=3).contains(&kind) {
                parameter = get_real(stream)?;
            }
        }
        Ok((kind, parameter))
    }

    /// The fields of one point representation; `None` when its geometry index names a
    /// null geometry (the record is still consumed).
    fn read_point_record(
        &self,
        kind: i32,
        parameter: f64,
        stream: &mut IoStream,
    ) -> Result<Option<Handle<dyn BRepPointRepresentation>>, Failure> {
        let identity = Location::default();
        match kind {
            0 => Ok(None),
            1 => {
                let curve = self.curves.curve(get_integer(stream)?)?;
                Ok((!curve.is_null()).then(|| {
                    Handle::new(BRepPointOnCurve::new(parameter, &curve, &identity)).into_cast()
                }))
            }
            2 => {
                let (pcurve_index, surface_index) = (get_integer(stream)?, get_integer(stream)?);
                let pcurve = self.curves2d.curve2d(pcurve_index)?;
                if pcurve.is_null() {
                    return Ok(None);
                }
                let surface = self.surfaces.surface(surface_index)?;
                Ok((!surface.is_null()).then(|| {
                    Handle::new(BRepPointOnCurveOnSurface::new(
                        parameter, &pcurve, &surface, &identity,
                    ))
                    .into_cast()
                }))
            }
            3 => {
                let parameter2 = get_real(stream)?;
                let surface = self.surfaces.surface(get_integer(stream)?)?;
                Ok((!surface.is_null()).then(|| {
                    Handle::new(BRepPointOnSurface::new(
                        parameter, parameter2, &surface, &identity,
                    ))
                    .into_cast()
                }))
            }
            _ => Err(Failure::new(&format!(
                "BinTools_SurfaceSet::ReadGeometry: UnExpected BRep_PointRepresentation = {kind}\n"
            ))),
        }
    }

    /// The natural restriction flag, the tolerance, the surface and location indices
    /// (a surface index up to 0 gives no surface), then an optional triangulation.
    fn read_face(&self, stream: &mut IoStream, shape: &mut Shape) -> Result<(), Failure> {
        let mut face = Face::default();
        self.builder.make_face(&mut face);
        *shape = face.clone().into_shape();
        let restriction = get_bool(stream);
        let tolerance = get_real(stream)?;
        let surface_index = get_integer(stream)?;
        let location_index = get_integer(stream)?;
        let surface = if surface_index > 0 {
            self.surfaces.surface(surface_index)?
        } else {
            Handle::default()
        };
        let location = self.locations.location(location_index)?;
        self.builder
            .update_face(&face, &surface, &location, tolerance)?;
        self.builder.natural_restriction(&face, restriction)?;
        if stream_get(stream) as u8 == 2 {
            let (triangulation, _) = self.triangulations.get(get_integer(stream)?)?;
            self.builder
                .update_face_triangulation(&face, &triangulation, true)?;
        }
        Ok(())
    }
}

/// The wire, shell, solid, composite solid and compound records: a new empty shape.
pub(super) fn make_topology(builder: &BRepBuilder, kind: ShapeEnum) -> Shape {
    match kind {
        ShapeEnum::Wire => {
            let mut wire = Wire::default();
            builder.make_wire(&mut wire);
            wire.into_shape()
        }
        ShapeEnum::Shell => {
            let mut shell = Shell::default();
            builder.make_shell(&mut shell);
            shell.into_shape()
        }
        ShapeEnum::Solid => {
            let mut solid = Solid::default();
            builder.make_solid(&mut solid);
            solid.into_shape()
        }
        ShapeEnum::CompSolid => {
            let mut comp_solid = CompSolid::default();
            builder.make_comp_solid(&mut comp_solid);
            comp_solid.into_shape()
        }
        _ => {
            let mut compound = Compound::default();
            builder.make_compound(&mut compound);
            compound.into_shape()
        }
    }
}
