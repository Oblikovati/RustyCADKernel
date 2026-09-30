// Rust translation of ModelingData/TKBRep/BRepTools/BRepTools_ShapeSet.hxx and the
// constructors, Clear, AddShapes and Check of BRepTools_ShapeSet.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepTools_ShapeSet`: the BRep ASCII format. The TopTools shape set writes the
//! topology; this set adds the geometry of the vertices, edges and faces, with the
//! curves, surfaces, polygons and triangulations as indexed tables.
mod add_geometry;
mod dump_shape;
mod geometry_io;
mod mesh_io;
mod polygon_io;
mod read_edge;
mod read_shape;
mod regularity;
mod write_shape;

use super::update_face;
use crate::bintools::handle_table::{FlaggedHandleTable, HandleTable};
use crate::brep::BRepBuilder;
use crate::topods::{self, Shape};
use crate::toptools::{ShapeSet, ShapeSetData, shape_set_io};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::geom_tools::{
    ConsoleOutput, Curve2dSet, CurveSet, StdoutConsole, SurfaceSet,
};
use rustycad_tkmath::poly::{Polygon2D, Polygon3D, PolygonOnTriangulation, Triangulation};
use std::sync::Arc;

/// Writes, reads and dumps shapes in the BRep ASCII format (`BRepTools_ShapeSet`).
///
/// It is the `TopTools_ShapeSet` trait [`ShapeSet`] over its own data, with the
/// geometry hooks overridden.
///
/// ```
/// use rustycad_tkbrep::brep::BRepBuilder;
/// use rustycad_tkbrep::brep_tools::BRepToolsShapeSet;
/// use rustycad_tkbrep::topods::Vertex;
/// use rustycad_tkbrep::toptools::ShapeSet;
/// use rustycad_tkmath::gp::Point3;
/// let mut vertex = Vertex::default();
/// BRepBuilder::default().make_vertex_at(&mut vertex, Point3::new(1.0, 2.0, 3.0), 1e-7)?;
/// let mut set = BRepToolsShapeSet::new(true, false);
/// assert_eq!(set.add(&vertex)?, 1);
/// assert!(set.is_with_triangles());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct BRepToolsShapeSet {
    data: ShapeSetData,
    builder: BRepBuilder,
    surfaces: SurfaceSet,
    curves: CurveSet,
    curves2d: Curve2dSet,
    polygons2d: HandleTable<Polygon2D>,
    polygons3d: HandleTable<Polygon3D>,
    /// Each triangulation with the flag that says whether its normals are written.
    triangulations: FlaggedHandleTable<Triangulation>,
    nodes: HandleTable<PolygonOnTriangulation>,
    with_triangles: bool,
    with_normals: bool,
}

impl Default for BRepToolsShapeSet {
    fn default() -> Self {
        Self::new(true, false)
    }
}

impl BRepToolsShapeSet {
    /// `BRepTools_ShapeSet(theWithTriangles, theWithNormals)`: an empty set that reports
    /// on standard output; `BRepToolsShapeSet::new(true, false)`.
    pub fn new(with_triangles: bool, with_normals: bool) -> Self {
        Self::with_builder(BRepBuilder::default(), with_triangles, with_normals)
    }

    /// `BRepTools_ShapeSet(theBuilder, theWithTriangles, theWithNormals)`;
    /// `BRepToolsShapeSet::with_builder(BRepBuilder::default(), true, false)`.
    pub fn with_builder(builder: BRepBuilder, with_triangles: bool, with_normals: bool) -> Self {
        Self::with_console(
            builder,
            with_triangles,
            with_normals,
            Arc::new(StdoutConsole),
        )
    }

    /// An empty set whose `Read` messages (and those of its geometry sets) go to
    /// `console`, the native `std::cout`;
    /// `BRepToolsShapeSet::with_console(BRepBuilder::default(), true, false, console)`.
    pub fn with_console(
        builder: BRepBuilder,
        with_triangles: bool,
        with_normals: bool,
        console: Arc<dyn ConsoleOutput>,
    ) -> Self {
        Self {
            data: ShapeSetData::with_console(console.clone()),
            builder,
            surfaces: SurfaceSet::with_console(console.clone()),
            curves: CurveSet::with_console(console.clone()),
            curves2d: Curve2dSet::with_console(console),
            polygons2d: HandleTable::new("BRepTools_ShapeSet"),
            polygons3d: HandleTable::new("BRepTools_ShapeSet"),
            triangulations: FlaggedHandleTable::new("BRepTools_ShapeSet"),
            nodes: HandleTable::new("BRepTools_ShapeSet"),
            with_triangles,
            with_normals,
        }
    }

    /// `IsWithTriangles()`; `set.is_with_triangles()`.
    pub fn is_with_triangles(&self) -> bool {
        self.with_triangles
    }

    /// `IsWithNormals()`; `set.is_with_normals()`.
    pub fn is_with_normals(&self) -> bool {
        self.with_normals
    }

    /// `SetWithTriangles(theWithTriangles)`; `set.set_with_triangles(false)`.
    pub fn set_with_triangles(&mut self, with_triangles: bool) {
        self.with_triangles = with_triangles;
    }

    /// `SetWithNormals(theWithNormals)`; `set.set_with_normals(true)`.
    pub fn set_with_normals(&mut self, with_normals: bool) {
        self.with_normals = with_normals;
    }
}

impl ShapeSet for BRepToolsShapeSet {
    fn shape_set_data(&self) -> &ShapeSetData {
        &self.data
    }
    fn shape_set_data_mut(&mut self) -> &mut ShapeSetData {
        &mut self.data
    }
    /// `Clear()`: the geometry tables, then the base class tables.
    fn clear(&mut self) {
        self.surfaces.clear();
        self.curves.clear();
        self.curves2d.clear();
        self.polygons3d.clear();
        self.polygons2d.clear();
        self.nodes.clear();
        self.triangulations.clear();
        shape_set_io::clear(&mut self.data);
    }
    fn add_geometry(&mut self, shape: &Shape) -> Result<(), Failure> {
        self.add_shape_geometry(shape)
    }
    fn dump_geometry(&self, stream: &mut IoStream) -> Result<(), Failure> {
        self.dump_tables(stream)
    }
    fn write_geometry(
        &mut self,
        stream: &mut IoStream,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        self.write_tables(stream, progress)
    }
    fn read_geometry(
        &mut self,
        stream: &mut IoStream,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        self.read_tables(stream, progress)
    }
    fn dump_shape_geometry(&self, shape: &Shape, stream: &mut IoStream) -> Result<(), Failure> {
        self.dump_geometry_of(shape, stream)
    }
    fn write_shape_geometry(&self, shape: &Shape, stream: &mut IoStream) -> Result<(), Failure> {
        self.write_geometry_of(shape, stream)
    }
    fn read_shape_geometry(
        &mut self,
        kind: ShapeEnum,
        stream: &mut IoStream,
        shape: &mut Shape,
    ) -> Result<(), Failure> {
        self.read_geometry_of(kind, stream, shape)
    }
    /// `AddShapes(S1, S2)`: `myBuilder.Add(S1, S2)`.
    fn add_shapes(&mut self, shape: &mut Shape, child: &Shape) -> Result<(), Failure> {
        self.builder.add(shape, child)
    }
    /// `Check(T, S)`: a face read from a version 1 file gets its UV points updated.
    fn check(&mut self, kind: ShapeEnum, shape: &mut Shape) -> Result<(), Failure> {
        if kind == ShapeEnum::Face {
            update_face(&topods::face(shape)?)?;
        }
        Ok(())
    }
}
