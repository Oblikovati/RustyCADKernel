//! `StdPrs_ShapeTool`: the faces, the edges with their adjacent faces and the free or
//! internal vertices of a shape, iterated as the wireframe and shading builders do.
//! Native planarity uses exact dynamic type equality and unwraps one rectangular trim only.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, ShapeIterator, Vertex};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape};
use rustycad_tkernel::ncollection::HSequence;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg3d::geom::{GeomPlane, GeomRectangularTrimmedSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::poly::{Polygon3D, PolygonOnTriangulation, Triangulation, mesh_purpose};
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;

/// The face, edge and vertex iterator of a shape.
///
/// Example: `let mut tool = StdPrsShapeTool::new(&shape, false)?; tool.init_face()?;`.
pub struct StdPrsShapeTool {
    shape: Shape,
    face_explorer: Explorer,
    edge_map: IndexedDataMapOfShapeListOfShape,
    vertex_map: IndexedMapOfShape,
    edge: i32,
    vertex: i32,
}

impl StdPrsShapeTool {
    /// The tool of a shape: its edges with their faces, and every vertex when
    /// `all_vertices`, else the vertices outside the edges and the internal edge vertices.
    /// Example: `StdPrsShapeTool::new(&shape, true)?`.
    pub fn new(shape: &Shape, all_vertices: bool) -> Result<Self, Failure> {
        let mut edge_map = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(shape, ShapeEnum::Edge, ShapeEnum::Face, &mut edge_map)?;
        let mut vertex_map = IndexedMapOfShape::default();
        if all_vertices {
            for vertex in Explorer::new(shape, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                vertex_map.add(vertex);
            }
        } else {
            for vertex in Explorer::new(shape, ShapeEnum::Vertex, ShapeEnum::Edge)? {
                vertex_map.add(vertex);
            }
            for edge in Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
                for vertex in ShapeIterator::new(&edge, false, true) {
                    if vertex.orientation() == Orientation::Internal {
                        vertex_map.add(vertex);
                    }
                }
            }
        }
        Ok(Self {
            shape: shape.clone(),
            face_explorer: Explorer::default(),
            edge_map,
            vertex_map,
            edge: 0,
            vertex: 0,
        })
    }

    /// `InitFace`: restarts the face exploration; `tool.init_face()?`.
    pub fn init_face(&mut self) -> Result<(), Failure> {
        let shape = self.shape.clone();
        self.face_explorer
            .init(&shape, ShapeEnum::Face, ShapeEnum::Shape)
    }
    /// `MoreFace`; `tool.more_face()`.
    pub fn more_face(&self) -> bool {
        self.face_explorer.more()
    }
    /// `NextFace`; `tool.next_face()?`.
    pub fn next_face(&mut self) -> Result<(), Failure> {
        self.face_explorer.next()
    }
    /// `GetFace`: the current face (`TopoDS::Face` of the explorer position).
    /// Example: `tool.get_face()?`.
    pub fn get_face(&self) -> Result<Face, Failure> {
        topods::face(self.face_explorer.current())
    }
    /// `FaceBound`: the box of the current face; `tool.face_bound()?`.
    pub fn face_bound(&self) -> Result<BoundingBox3, Failure> {
        let mut bounds = BoundingBox3::default();
        BRepBndLib::add(&Shape::from(self.get_face()?), &mut bounds, true)?;
        Ok(bounds)
    }
    /// `IsPlanarFace()`: whether the current face lies on a plane.
    /// Example: `tool.is_planar_current_face()?`.
    pub fn is_planar_current_face(&self) -> Result<bool, Failure> {
        Self::is_planar_face(&self.get_face()?)
    }

    /// `InitCurve`; `tool.init_curve()`.
    pub fn init_curve(&mut self) {
        self.edge = 1;
    }
    /// `MoreCurve`; `tool.more_curve()`.
    pub fn more_curve(&self) -> bool {
        self.edge as usize <= self.edge_map.extent() && self.edge >= 0
    }
    /// `NextCurve`; `tool.next_curve()`.
    pub fn next_curve(&mut self) {
        self.edge += 1;
    }
    /// `GetCurve`: the current edge; Standard_OutOfRange past the end.
    /// Example: `tool.get_curve()?`.
    pub fn get_curve(&self) -> Result<Edge, Failure> {
        topods::edge(self.edge_map.find_key(self.edge as usize)?)
    }
    /// `CurveBound`: the box of the current edge; `tool.curve_bound()?`.
    pub fn curve_bound(&self) -> Result<BoundingBox3, Failure> {
        let mut bounds = BoundingBox3::default();
        BRepBndLib::add(&Shape::from(self.get_curve()?), &mut bounds, true)?;
        Ok(bounds)
    }
    /// `Neighbours`: the number of faces of the current edge; `tool.neighbours()?`.
    pub fn neighbours(&self) -> Result<i32, Failure> {
        Ok(self.edge_map.find_from_index(self.edge as usize)?.extent())
    }
    /// `FacesOfEdge`: the faces of the current edge; `tool.faces_of_edge()?`.
    pub fn faces_of_edge(&self) -> Result<Handle<HSequence<Shape>>, Failure> {
        let faces = Handle::new(HSequence::default());
        for face in self.edge_map.find_from_index(self.edge as usize)?.iter() {
            faces.append(face.clone());
        }
        Ok(faces)
    }

    /// `InitVertex`; `tool.init_vertex()`.
    pub fn init_vertex(&mut self) {
        self.vertex = 1;
    }
    /// `MoreVertex`; `tool.more_vertex()`.
    pub fn more_vertex(&self) -> bool {
        self.vertex as usize <= self.vertex_map.extent() && self.vertex >= 0
    }
    /// `NextVertex`; `tool.next_vertex()`.
    pub fn next_vertex(&mut self) {
        self.vertex += 1;
    }
    /// `GetVertex`: the current vertex; Standard_OutOfRange past the end.
    /// Example: `tool.get_vertex()?`.
    pub fn get_vertex(&self) -> Result<Vertex, Failure> {
        topods::vertex(self.vertex_map.find_key(self.vertex as usize)?)
    }

    /// `HasSurface`: whether the current face has a surface; `tool.has_surface()?`.
    pub fn has_surface(&self) -> Result<bool, Failure> {
        let (surface, _) = BRepTool::surface_with_location(&self.get_face()?)?;
        Ok(!surface.is_null())
    }
    /// `CurrentTriangulation`: the triangulation of the current face and its location.
    /// Example: `let (triangulation, location) = tool.current_triangulation()?;`.
    pub fn current_triangulation(&self) -> Result<(Handle<Triangulation>, Location), Failure> {
        BRepTool::triangulation(&self.get_face()?, mesh_purpose::NONE)
    }
    /// `HasCurve`: whether the current edge is geometric; `tool.has_curve()?`.
    pub fn has_curve(&self) -> Result<bool, Failure> {
        BRepTool::is_geometric_edge(&self.get_curve()?)
    }
    /// `PolygonOnTriangulation`: the first polygon on triangulation of the current edge, its
    /// triangulation and location. Example: `let (indices, triangulation, location) = tool.polygon_on_triangulation()?;`.
    pub fn polygon_on_triangulation(
        &self,
    ) -> Result<
        (
            Handle<PolygonOnTriangulation>,
            Handle<Triangulation>,
            Location,
        ),
        Failure,
    > {
        BRepTool::first_polygon_on_triangulation(&self.get_curve()?)
    }
    /// `Polygon3D`: the 3d polygon of the current edge and its location.
    /// Example: `let (polygon, location) = tool.polygon_3d()?;`.
    pub fn polygon_3d(&self) -> Result<(Handle<Polygon3D>, Location), Failure> {
        BRepTool::polygon3d(&self.get_curve()?)
    }

    /// `IsPlanarFace(face)`: exact Geom_Plane identity after at most one trim.
    /// Example: `StdPrsShapeTool::is_planar_face(&face)?`.
    pub fn is_planar_face(face: &Face) -> Result<bool, Failure> {
        let (mut surface, _) = BRepTool::surface_with_location(face)?;
        if surface.is_null() {
            return Ok(false);
        }
        if exact_shaded_surface_type::<GeomRectangularTrimmedSurface>(&surface) {
            surface = surface
                .cast::<GeomRectangularTrimmedSurface>()
                .basis_surface();
        }
        Ok(exact_shaded_surface_type::<GeomPlane>(&surface))
    }
}

fn exact_shaded_surface_type<T: RuntimeTyped>(
    surface: &Handle<dyn rustycad_tkg3d::geom::GeomSurface>,
) -> bool {
    surface
        .dynamic_type()
        .is_some_and(|descriptor| std::ptr::eq(descriptor, T::static_type()))
}
