// Rust translation of ModelingData/TKBRep/BinTools/BinTools_ShapeReader.hxx and the
// members Clear, Read, ReadShape and ReadLocation of BinTools_ShapeReader.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BinTools_ShapeReader`: reads the shape-reference format back, following back
//! references to objects read before (or, when the reader has not met them yet, by
//! jumping to their position and back).
use super::errors::wrap_failure;
use super::istream::BinIStream;
use super::object_type::ObjectType;
use super::raw_enums::{orientation_from_raw, shape_enum_from_raw};
use super::shape_set_base::{BinShapeSetBase, ShapeSetBaseData};
use super::shape_set_read::apply_flags;
use crate::brep::BRepBuilder;
use crate::topods::Shape;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::poly::{Polygon3D, PolygonOnTriangulation, Triangulation};
use rustycad_tkmath::toploc::Location;

/// The objects read so far, by the byte position of their record.
pub(super) type ObjectMap<T> = DataMap<u64, Handle<T>>;

/// Reads shapes written by [`BinShapeWriter`](super::BinShapeWriter). The objects read
/// are kept between calls (`Clear` forgets them), so the shapes of one stream can be
/// read one after the other.
///
/// ```
/// use rustycad_tkbrep::bintools::{BinShapeReader, BinShapeSetBase};
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkernel::storage::IoStream;
/// let mut stream = IoStream::from_bytes(vec![198]);
/// let mut shape = Shape::default();
/// BinShapeReader::new().read_shape(&mut stream, &mut shape).unwrap();
/// assert!(shape.is_null());
/// ```
#[derive(Default)]
pub struct BinShapeReader {
    base: ShapeSetBaseData,
    pub(super) builder: BRepBuilder,
    pub(super) shape_positions: DataMap<u64, Shape>,
    pub(super) location_positions: DataMap<u64, Location>,
    pub(super) curve_positions: ObjectMap<dyn GeomCurve>,
    pub(super) curve2d_positions: ObjectMap<dyn Geom2dCurve>,
    pub(super) surface_positions: ObjectMap<dyn GeomSurface>,
    pub(super) polygon3d_positions: ObjectMap<Polygon3D>,
    pub(super) polygon_positions: ObjectMap<PolygonOnTriangulation>,
    pub(super) triangulation_positions: ObjectMap<Triangulation>,
}

impl BinShapeReader {
    /// `BinTools_ShapeReader()`; `BinShapeReader::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `ReadShape(theStream)` (private natively): a null shape for `EmptyShape` or
    /// `EndShape`; for a back reference, the shape read at that position (read now when
    /// it was not met yet) with the location and orientation that follow; else a new
    /// shape: its type and orientation byte, location, geometry, flags byte and
    /// sub-shapes up to `EndShape`. The shape is recorded before its own orientation and
    /// location are applied.
    pub(super) fn read_shape_record(
        &mut self,
        input: &mut BinIStream<'_>,
    ) -> Result<Shape, Failure> {
        let position = input.position();
        let kind = input.read_type();
        if kind == ObjectType::EMPTY_SHAPE || kind == ObjectType::END_SHAPE {
            return Ok(Shape::default());
        }
        if input.is_reference() {
            return self.read_shape_reference(input);
        }
        let (shape_type, orientation) = (input.shape_type(), input.shape_orientation());
        let location = self.read_location(input)?;
        let mut shape = Shape::default();
        wrap_failure(
            "EXCEPTION in BinTools_ShapeReader::Read",
            || match shape_enum_from_raw(shape_type) {
                Some(ShapeEnum::Shape) | None => Err(Failure::new(&format!(
                    "Unexpected topology type = {shape_type}\n"
                ))),
                Some(kind) => self.read_shape_geometry(input, kind, &mut shape),
            },
        )?;
        let flags = input.read_bools7();
        loop {
            let child = self.read_shape_record(input)?;
            if child.is_null() {
                break;
            }
            self.builder.add(&mut shape, &child)?;
        }
        apply_flags(&shape, flags)?;
        self.shape_positions.bind(position, shape.clone());
        shape.set_orientation(orientation);
        shape.set_location(&location, false)?;
        Ok(shape)
    }

    fn read_shape_reference(&mut self, input: &mut BinIStream<'_>) -> Result<Shape, Failure> {
        let reference = input.read_reference()?;
        let mut shape = match self.shape_positions.seek(&reference) {
            Some(found) => found.clone(),
            None => {
                let current = input.position();
                input.go_to(reference);
                let shape = self.read_shape_record(input)?;
                input.go_to(current);
                shape
            }
        };
        let location = self.read_location(input)?;
        shape.set_location(&location, false)?;
        shape.set_orientation(orientation_from_raw(i32::from(input.read_byte()))?);
        Ok(shape)
    }

    /// `ReadLocation(theStream)`: the identity for `EmptyLocation` or `LocationEnd`; for a
    /// back reference, the location read at that position; for `SimpleLocation`, a new
    /// datum from the matrix; for `Location`, the product of the powered datums up to
    /// the identity; any other type gives the identity. A new location is recorded at
    /// its position; `reader.read_location(&mut input)?`.
    pub fn read_location(&mut self, input: &mut BinIStream<'_>) -> Result<Location, Failure> {
        let position = input.position();
        let kind = input.read_type();
        if kind == ObjectType::EMPTY_LOCATION || kind == ObjectType::LOCATION_END {
            return Ok(Location::default());
        }
        if input.is_reference() {
            let reference = input.read_reference()?;
            if let Some(found) = self.location_positions.seek(&reference) {
                return Ok(found.clone());
            }
            let current = input.position();
            input.go_to(reference);
            let location = self.read_location(input)?;
            input.go_to(current);
            return Ok(location);
        }
        let mut location = Location::default();
        if kind == ObjectType::SIMPLE_LOCATION {
            location = Location::new(input.read_transform()?);
        } else if kind == ObjectType::LOCATION {
            location = self.read_location_chain(input)?;
        }
        self.location_positions.bind(position, location.clone());
        Ok(location)
    }

    /// `aLoc = aNextLoc->Powered(ReadInteger()) * aLoc` until an identity location.
    fn read_location_chain(&mut self, input: &mut BinIStream<'_>) -> Result<Location, Failure> {
        let mut location = Location::default();
        loop {
            let next = self.read_location(input)?;
            if next.is_identity() {
                return Ok(location);
            }
            location = next.powered(input.read_integer()?)?.multiplied(&location)?;
        }
    }
}

impl BinShapeSetBase for BinShapeReader {
    fn base_data(&self) -> &ShapeSetBaseData {
        &self.base
    }
    fn base_data_mut(&mut self) -> &mut ShapeSetBaseData {
        &mut self.base
    }

    /// `Clear()`: forgets every object read; the options are kept.
    fn clear(&mut self) {
        self.shape_positions.clear(true);
        self.location_positions.clear(true);
        self.curve_positions.clear(true);
        self.curve2d_positions.clear(true);
        self.surface_positions.clear(true);
        self.polygon3d_positions.clear(true);
        self.polygon_positions.clear(true);
        self.triangulation_positions.clear(true);
    }

    /// `Read(theStream, theShape)`: one shape through a `BinTools_IStream` that starts at
    /// the current stream position.
    fn read_shape(&mut self, stream: &mut IoStream, shape: &mut Shape) -> Result<(), Failure> {
        let mut input = BinIStream::new(stream);
        *shape = self.read_shape_record(&mut input)?;
        Ok(())
    }
}
