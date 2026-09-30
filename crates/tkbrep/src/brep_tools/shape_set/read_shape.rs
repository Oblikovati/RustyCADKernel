// Rust translation of the vertex, face and container cases of
// BRepTools_ShapeSet::ReadGeometry(T, IS, S)
// (ModelingData/TKBRep/BRepTools/BRepTools_ShapeSet.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Reading the geometry record of one shape back and making the shape.
use super::BRepToolsShapeSet;
use crate::brep::access::vertex_tshape;
use crate::brep::{
    BRepPointOnCurve, BRepPointOnCurveOnSurface, BRepPointOnSurface, BRepPointRepresentation,
};
use crate::topods::{CompSolid, Compound, Face, Shape, Shell, Solid, Vertex, Wire};
use crate::toptools::ShapeSet;
use crate::toptools::shape_set_text::c_atoi;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::geom_tools::get_real;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::toploc::Location;

/// `char string[260]` read with `getline(string, 256, '\n')`.
const FACE_LINE: i64 = 256;

/// `IS >> value` into an `int`; the value is kept when the stream has already failed.
pub(super) fn read_int(stream: &mut IoStream, value: &mut i32) {
    stream.extract_int(value);
}

impl BRepToolsShapeSet {
    /// `ReadGeometry(T, IS, S)`: makes `shape` of type `kind` with the builder and, for
    /// a vertex, an edge or a face, reads its geometry record. References to missing
    /// geometry (a null curve or surface, an index out of range) are skipped, as natively.
    pub(super) fn read_geometry_of(
        &mut self,
        kind: ShapeEnum,
        stream: &mut IoStream,
        shape: &mut Shape,
    ) -> Result<(), Failure> {
        *shape = match kind {
            ShapeEnum::Vertex => self.read_vertex(stream)?.into_shape(),
            ShapeEnum::Edge => self.read_edge(stream)?.into_shape(),
            ShapeEnum::Wire => self.make_container(Wire::default(), |b, s| b.make_wire(s)),
            ShapeEnum::Face => self.read_face(stream)?.into_shape(),
            ShapeEnum::Shell => self.make_container(Shell::default(), |b, s| b.make_shell(s)),
            ShapeEnum::Solid => self.make_container(Solid::default(), |b, s| b.make_solid(s)),
            ShapeEnum::CompSolid => {
                self.make_container(CompSolid::default(), |b, s| b.make_comp_solid(s))
            }
            ShapeEnum::Compound => {
                self.make_container(Compound::default(), |b, s| b.make_compound(s))
            }
            ShapeEnum::Shape => return Ok(()),
        };
        Ok(())
    }

    /// `myBuilder.MakeWire(...)` and the other containers without geometry.
    fn make_container<T: Into<Shape>>(
        &self,
        mut shape: T,
        make: impl FnOnce(&crate::topods::Builder, &mut T),
    ) -> Shape {
        make(&self.builder, &mut shape);
        shape.into()
    }

    /// `tolerance x y z` then the point representations, ended by a representation
    /// kind of 0 (or by a stream failure, where the native loop would not end).
    fn read_vertex(&mut self, stream: &mut IoStream) -> Result<Vertex, Failure> {
        let tolerance = get_real(stream);
        let (x, y, z) = (get_real(stream), get_real(stream), get_real(stream));
        let mut vertex = Vertex::default();
        self.builder
            .make_vertex_at(&mut vertex, Point3::new(x, y, z), tolerance)?;
        let mut kind = 0;
        loop {
            let parameter = get_real(stream);
            read_int(stream, &mut kind);
            let representation = self.read_point_representation(stream, kind, parameter)?;
            if kind > 0 {
                let mut location_index = 0;
                read_int(stream, &mut location_index);
                self.attach_point(&vertex, representation, location_index)?;
            }
            if kind <= 0 || stream.fail() {
                return Ok(vertex);
            }
        }
    }

    /// The representation of kind 1 (on a curve), 2 (on a p-curve and a surface) or 3
    /// (on a surface), at the identity location; none for missing geometry or another
    /// kind.
    fn read_point_representation(
        &self,
        stream: &mut IoStream,
        kind: i32,
        parameter: f64,
    ) -> Result<Option<Handle<dyn BRepPointRepresentation>>, Failure> {
        let identity = Location::default();
        let (mut c, mut s) = (0, 0);
        match kind {
            1 => {
                read_int(stream, &mut c);
                let curve = self.curves.curve(c);
                if curve.is_null() {
                    return Ok(None);
                }
                let point = BRepPointOnCurve::new(parameter, &curve, &identity);
                Ok(Some(Handle::new(point).into_cast()))
            }
            2 => {
                read_int(stream, &mut c);
                read_int(stream, &mut s);
                let (pcurve, surface) = (self.curves2d.curve2d(c), self.surfaces.surface(s));
                if pcurve.is_null() || surface.is_null() {
                    return Ok(None);
                }
                let point = BRepPointOnCurveOnSurface::new(parameter, &pcurve, &surface, &identity);
                Ok(Some(Handle::new(point).into_cast()))
            }
            3 => {
                let parameter2 = get_real(stream);
                read_int(stream, &mut s);
                let surface = self.surfaces.surface(s);
                if surface.is_null() {
                    return Ok(None);
                }
                let point = BRepPointOnSurface::new(parameter, parameter2, &surface, &identity);
                Ok(Some(Handle::new(point).into_cast()))
            }
            _ => Ok(None),
        }
    }

    /// `PR->Location(Locations().Location(l)); lpr.Append(PR);`.
    fn attach_point(
        &self,
        vertex: &Vertex,
        representation: Option<Handle<dyn BRepPointRepresentation>>,
        location_index: i32,
    ) -> Result<(), Failure> {
        let Some(representation) = representation else {
            return Ok(());
        };
        if let Some(point) = representation.get() {
            point.set_location(&self.locations().location(location_index));
        }
        let tvertex = vertex_tshape(vertex, "BRepTools_ShapeSet::ReadGeometry")?;
        tvertex.change_points(|points| points.push(representation));
        Ok(())
    }

    /// `natural-restriction tolerance surface location` (or `2 triangulation` for a face
    /// with only a triangulation); then, reading two lines as natively (BUC60769), a
    /// second line starting with `2` names the triangulation.
    fn read_face(&mut self, stream: &mut IoStream) -> Result<Face, Failure> {
        let mut face = Face::default();
        self.builder.make_face(&mut face);
        let mut kind = 0;
        read_int(stream, &mut kind);
        if kind == 0 || kind == 1 {
            self.read_face_surface(stream, &face, kind == 1)?;
        } else if kind == 2 {
            let mut index = 0;
            read_int(stream, &mut index);
            let (triangulation, _) = self.triangulations.get(index)?;
            self.builder
                .update_face_triangulation(&face, &triangulation, true)?;
            return Ok(face);
        }
        let mut line = Vec::new();
        stream.getline(&mut line, FACE_LINE, b'\n');
        line.clear();
        stream.getline(&mut line, FACE_LINE, b'\n');
        if line.first() == Some(&b'2') {
            let index = c_atoi(line.get(2..).unwrap_or_default());
            if index > 0 && index <= self.triangulations.extent() {
                let (triangulation, _) = self.triangulations.get(index)?;
                self.builder
                    .update_face_triangulation(&face, &triangulation, true)?;
            }
        }
        Ok(face)
    }

    /// `tolerance surface location`: the surface is set with the natural restriction
    /// flag when it exists.
    fn read_face_surface(
        &self,
        stream: &mut IoStream,
        face: &Face,
        restriction: bool,
    ) -> Result<(), Failure> {
        let tolerance = get_real(stream);
        let (mut s, mut l) = (0, 0);
        read_int(stream, &mut s);
        read_int(stream, &mut l);
        let surface = self.surfaces.surface(s);
        if surface.is_null() {
            return Ok(());
        }
        self.builder
            .update_face(face, &surface, &self.locations().location(l), tolerance)?;
        self.builder.natural_restriction(face, restriction)
    }
}
