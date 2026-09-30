// Rust translation of ModelingData/TKBRep/BinTools/BinTools_ShapeSet.hxx and the
// members Clear, Add, Shape, Index, Locations, ChangeLocations, NbShapes and
// AddShapes of BinTools_ShapeSet.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BinTools_ShapeSet`: the grouped binary format. The shapes and their geometry are
//! collected into indexed tables, written section by section (locations, 2D curves,
//! curves, polygons, surfaces, triangulations, then the `TShapes`), and referred to by
//! index.
use super::curve_set::BinCurveSet;
use super::curve2d_set::BinCurve2dSet;
use super::handle_table::{FlaggedHandleTable, HandleTable};
use super::location_set::BinLocationSet;
use super::shape_set_base::{BinShapeSetBase, ShapeSetBaseData};
use super::surface_set::BinSurfaceSet;
use crate::brep::BRepBuilder;
use crate::topods::{Shape, ShapeIterator};
use crate::toptools::IndexedMapOfShape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkgeombase::geom_tools::{ConsoleOutput, StdoutConsole};
use rustycad_tkmath::poly::{Polygon2D, Polygon3D, PolygonOnTriangulation, Triangulation};
use rustycad_tkmath::toploc::Location;
use std::sync::Arc;

/// Writes and reads the topology of shapes in the grouped binary format.
///
/// No OCCT class derives from `BinTools_ShapeSet`, so its own virtual members
/// (`WriteGeometry`, `ReadShape`, `AddShape`...) are inherent methods; the members it
/// overrides from `BinTools_ShapeSetBase` are the [`BinShapeSetBase`] trait methods.
///
/// ```
/// use rustycad_tkbrep::bintools::{BinShapeSet, BinShapeSetBase};
/// use rustycad_tkbrep::brep::BRepBuilder;
/// use rustycad_tkbrep::topods::Vertex;
/// use rustycad_tkmath::gp::Point3;
/// let mut vertex = Vertex::default();
/// BRepBuilder::default().make_vertex_at(&mut vertex, Point3::new(1.0, 2.0, 3.0), 1e-7).unwrap();
/// let mut set = BinShapeSet::new();
/// assert_eq!(set.add(&vertex).unwrap(), 1);
/// assert_eq!(set.nb_shapes(), 1);
/// assert!(!set.is_with_triangles());
/// ```
pub struct BinShapeSet {
    pub(super) base: ShapeSetBaseData,
    pub(super) shapes: IndexedMapOfShape,
    pub(super) locations: BinLocationSet,
    pub(super) builder: BRepBuilder,
    pub(super) surfaces: BinSurfaceSet,
    pub(super) curves: BinCurveSet,
    pub(super) curves2d: BinCurve2dSet,
    pub(super) polygons2d: HandleTable<Polygon2D>,
    pub(super) polygons3d: HandleTable<Polygon3D>,
    /// Each triangulation with the flag that says whether its normals are written.
    pub(super) triangulations: FlaggedHandleTable<Triangulation>,
    pub(super) nodes: HandleTable<PolygonOnTriangulation>,
    /// The native `std::cout` of `Read`.
    pub(super) console: Arc<dyn ConsoleOutput>,
}

impl Default for BinShapeSet {
    fn default() -> Self {
        Self::new()
    }
}

impl BinShapeSet {
    /// `BinTools_ShapeSet()`: an empty set that reports on standard output;
    /// `BinShapeSet::new()`.
    pub fn new() -> Self {
        Self::with_console(Arc::new(StdoutConsole))
    }

    /// An empty set whose `Read` writes its version message to `console` (the native
    /// `std::cout`); `BinShapeSet::with_console(Arc::new(StdoutConsole))`.
    pub fn with_console(console: Arc<dyn ConsoleOutput>) -> Self {
        Self {
            base: ShapeSetBaseData::default(),
            shapes: IndexedMapOfShape::default(),
            locations: BinLocationSet::new(),
            builder: BRepBuilder::default(),
            surfaces: BinSurfaceSet::new(),
            curves: BinCurveSet::new(),
            curves2d: BinCurve2dSet::new(),
            polygons2d: HandleTable::new("BinTools_ShapeSet"),
            polygons3d: HandleTable::new("BinTools_ShapeSet"),
            triangulations: FlaggedHandleTable::new("BinTools_ShapeSet"),
            nodes: HandleTable::new("BinTools_ShapeSet"),
            console,
        }
    }

    /// `Add(S)`: stores `shape` without its location and, first, its sub-shapes (each
    /// with `AddShape`); returns the index of `shape`, 0 for a null shape. The location
    /// goes to the location set; `set.add(&shape)?`.
    pub fn add(&mut self, shape: &Shape) -> Result<i32, Failure> {
        if shape.is_null() {
            return Ok(0);
        }
        self.locations.add(shape.location());
        let unlocated = shape.located(&Location::default(), false)?;
        let mut index = self.shapes.find_index(&unlocated) as i32;
        if index == 0 {
            self.add_shape(&unlocated)?;
            let mut children = ShapeIterator::new(&unlocated, false, false);
            while children.more() {
                self.add(children.value()?)?;
                children.next();
            }
            index = self.shapes.add(unlocated) as i32;
        }
        Ok(index)
    }

    /// `Shape(I)`: the stored shape of index `index`; Standard_OutOfRange outside
    /// `1..=NbShapes`; `set.shape(1)?`.
    pub fn shape(&self, index: i32) -> Result<Shape, Failure> {
        usize::try_from(index)
            .ok()
            .and_then(|position| self.shapes.find_key(position).ok())
            .cloned()
            .ok_or_else(|| {
                super::errors::index_out_of_range(
                    "BinTools_ShapeSet::Shape",
                    index,
                    self.shapes.extent(),
                )
            })
    }

    /// `Index(S)`: the index of `shape` (compared with `IsSame`), 0 when absent;
    /// `set.index(&shape)`.
    pub fn index(&self, shape: &Shape) -> i32 {
        self.shapes.find_index(shape) as i32
    }

    /// `Locations()`; `set.locations().nb_locations()`.
    pub fn locations(&self) -> &BinLocationSet {
        &self.locations
    }

    /// `ChangeLocations()`; `set.change_locations().add(&location)`.
    pub fn change_locations(&mut self) -> &mut BinLocationSet {
        &mut self.locations
    }

    /// `NbShapes()`: the number of stored (or read) shapes; `set.nb_shapes()`.
    pub fn nb_shapes(&self) -> i32 {
        self.shapes.extent() as i32
    }

    /// `AddShapes(S1, S2)`: `BRep_Builder::Add(S1, S2)`; `set.add_shapes(&mut wire, &edge)?`.
    pub fn add_shapes(&self, shape: &mut Shape, child: &Shape) -> Result<(), Failure> {
        self.builder.add(shape, child)
    }

    /// The native `std::cout` text of `Read`.
    pub(super) fn console_write(&self, text: &[u8]) {
        self.console.write(text);
    }
}

impl BinShapeSetBase for BinShapeSet {
    fn base_data(&self) -> &ShapeSetBaseData {
        &self.base
    }
    fn base_data_mut(&mut self) -> &mut ShapeSetBaseData {
        &mut self.base
    }

    /// `Clear()`: empties every table; the options are kept.
    fn clear(&mut self) {
        self.surfaces.clear();
        self.curves.clear();
        self.curves2d.clear();
        self.polygons3d.clear();
        self.polygons2d.clear();
        self.nodes.clear();
        self.triangulations.clear();
        self.shapes.clear(true);
        self.locations.clear();
    }

    /// `Write(OS, theRange)`: the version line, the locations, the geometry and the
    /// `TShapes` table.
    fn write(
        &mut self,
        stream: &mut IoStream,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        self.write_set(stream, progress)
    }

    /// `Read(IS, theRange)`: clears the set, then reads what `Write` writes.
    fn read(&mut self, stream: &mut IoStream, progress: &ProgressRange<'_>) -> Result<(), Failure> {
        self.read_set(stream, progress)
    }

    /// `Write(S, OS)`: `*` for a null shape, else the orientation byte, the index of the
    /// TShape counted from the end of the table and the index of the location.
    fn write_shape(&mut self, shape: &Shape, stream: &mut IoStream) -> Result<(), Failure> {
        self.write_shape_reference(shape, stream)
    }
}
