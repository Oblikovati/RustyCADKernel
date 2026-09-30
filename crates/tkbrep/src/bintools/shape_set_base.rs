// Rust translation of ModelingData/TKBRep/BinTools/BinTools_ShapeSetBase.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BinTools_ShapeSetBase`: the options every binary shape reader and writer shares
//! (format version, triangles, normals) and their virtual entry points.
use super::format_version;
use super::scalars::put_real;
use crate::topods::Shape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkmath::gp::Point3;

/// `THE_ASCII_VERSIONS`: the version line of each format number (index 0 is unused).
pub const THE_ASCII_VERSIONS: [&str; format_version::UPPER as usize + 1] = [
    "",
    "Open CASCADE Topology V1 (c)",
    "Open CASCADE Topology V2 (c)",
    "Open CASCADE Topology V3 (c)",
    "Open CASCADE Topology V4, (c) Open Cascade",
];

/// `operator<<(Standard_OStream&, const gp_Pnt&)`: X, Y and Z with `PutReal`.
///
/// ```
/// use rustycad_tkernel::storage::IoStream;
/// use rustycad_tkmath::gp::Point3;
/// let mut stream = IoStream::from_bytes(Vec::new());
/// rustycad_tkbrep::bintools::put_point(&mut stream, Point3::new(1.0, 2.0, 3.0));
/// assert_eq!(stream.tellp(), 24);
/// ```
pub fn put_point(stream: &mut IoStream, point: Point3) -> &mut IoStream {
    put_real(stream, point.x());
    put_real(stream, point.y());
    put_real(stream, point.z())
}

/// The private fields of `BinTools_ShapeSetBase`: the format number and the triangle
/// and normal options. A new set writes the current format without triangles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShapeSetBaseData {
    format_nb: i32,
    with_triangles: bool,
    with_normals: bool,
}

impl Default for ShapeSetBaseData {
    /// `BinTools_ShapeSetBase()`: `BinTools_FormatVersion_CURRENT`, no triangles, no
    /// normals.
    fn default() -> Self {
        Self {
            format_nb: format_version::CURRENT,
            with_triangles: false,
            with_normals: false,
        }
    }
}

/// `BinTools_ShapeSetBase`: the class is concrete with empty virtual members, which
/// the binary shape set, reader and writer override. Its fields are
/// [`ShapeSetBaseData`], read through the two accessors; [`ShapeSetBaseData`] itself is
/// the base class instance.
///
/// ```
/// use rustycad_tkbrep::bintools::{BinShapeSetBase, ShapeSetBaseData, format_version};
/// let mut base = ShapeSetBaseData::default();
/// base.set_format_nb(7);
/// assert_eq!(base.format_nb(), format_version::CURRENT);
/// base.set_format_nb(format_version::VERSION_2);
/// assert_eq!(base.format_nb(), 2);
/// ```
pub trait BinShapeSetBase {
    /// Borrows the base fields.
    fn base_data(&self) -> &ShapeSetBaseData;
    /// Borrows the base fields mutably.
    fn base_data_mut(&mut self) -> &mut ShapeSetBaseData;

    /// `IsWithTriangles()`; `set.is_with_triangles()`.
    fn is_with_triangles(&self) -> bool {
        self.base_data().with_triangles
    }
    /// `IsWithNormals()`; `set.is_with_normals()`.
    fn is_with_normals(&self) -> bool {
        self.base_data().with_normals
    }
    /// `SetWithTriangles(flag)`: ignored for a face with only a triangulation, which is
    /// always written; `set.set_with_triangles(true)`.
    fn set_with_triangles(&mut self, with_triangles: bool) {
        self.base_data_mut().with_triangles = with_triangles;
    }
    /// `SetWithNormals(flag)`: ignored for a face with only a triangulation;
    /// `set.set_with_normals(true)`.
    fn set_with_normals(&mut self, with_normals: bool) {
        self.base_data_mut().with_normals = with_normals;
    }
    /// `SetFormatNb(n)`: a number outside `LOWER..=UPPER` is ignored
    /// (`Standard_ASSERT_RETURN` in a release build); `set.set_format_nb(2)`.
    fn set_format_nb(&mut self, format_nb: i32) {
        if (format_version::LOWER..=format_version::UPPER).contains(&format_nb) {
            self.base_data_mut().format_nb = format_nb;
        }
    }
    /// `FormatNb()`; `set.format_nb()`.
    fn format_nb(&self) -> i32 {
        self.base_data().format_nb
    }

    /// `Clear()`: nothing in the base class; `set.clear()`.
    fn clear(&mut self) {}
    /// `Write(OS, theRange)`: nothing in the base class;
    /// `set.write(&mut stream, &ProgressRange::default())?`.
    fn write(
        &mut self,
        _stream: &mut IoStream,
        _progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        Ok(())
    }
    /// `Read(IS, theRange)`: nothing in the base class;
    /// `set.read(&mut stream, &ProgressRange::default())?`.
    fn read(
        &mut self,
        _stream: &mut IoStream,
        _progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        Ok(())
    }
    /// `Write(theShape, theStream)`: nothing in the base class;
    /// `set.write_shape(&shape, &mut stream)?`.
    fn write_shape(&mut self, _shape: &Shape, _stream: &mut IoStream) -> Result<(), Failure> {
        Ok(())
    }
    /// `Read(theStream, theShape)`: nothing in the base class (and in the shape set);
    /// `set.read_shape(&mut stream, &mut shape)?`.
    fn read_shape(&mut self, _stream: &mut IoStream, _shape: &mut Shape) -> Result<(), Failure> {
        Ok(())
    }
}

impl BinShapeSetBase for ShapeSetBaseData {
    fn base_data(&self) -> &ShapeSetBaseData {
        self
    }
    fn base_data_mut(&mut self) -> &mut ShapeSetBaseData {
        self
    }
}
