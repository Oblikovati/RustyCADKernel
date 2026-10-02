//! Existing-mesh Delaun core from OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Insertion and constraint orchestration are separate, still unported dependencies.
use super::{
    BRepMeshCircleTool, BRepMeshDataStructureOfDelaun, BRepMeshEdge, BRepMeshTriangle,
    BRepMeshVertex, DegreeOfFreedom,
};
use rustycad_tkernel::{
    ncollection::{DynamicArray, IncrementalAllocatorObject, Map},
    standard::{Failure, Handle},
};
mod check_intersection;
mod circles;
mod cleanup;
mod cleanup_mesh;
mod cleanup_polygon;
mod contains;
mod frontier_reachability;
mod insertion;
mod insertion_order;
mod mesh_left_polygon;
mod next_polygon_link;
mod polygon;
mod polygon_decompose;
mod polygon_mesh;
mod polygon_state;
mod triangle_build;
/// Delaunay machinery over an existing mesh; `BRepMeshDelaun::from_mesh(&mesh,8,8,true)?`.
pub struct BRepMeshDelaun {
    mesh: Handle<BRepMeshDataStructureOfDelaun>,
    circles: BRepMeshCircleTool,
    auxiliary: DynamicArray<i32>,
    super_edges: [i32; 3],
    initialized_circles: bool,
}
impl BRepMeshDelaun {
    /// Native constructor that does not triangulate; `BRepMeshDelaun::from_mesh(&mesh,8,8,true)?`.
    pub fn from_mesh(
        mesh: &Handle<BRepMeshDataStructureOfDelaun>,
        u: i32,
        v: i32,
        fill: bool,
    ) -> Result<Self, Failure> {
        let allocator = Handle::new(IncrementalAllocatorObject::new(
            crate::imesh_data::MEMORY_BLOCK_SIZE_HUGE,
        ));
        let mut result = Self {
            mesh: mesh.clone(),
            circles: BRepMeshCircleTool::new(&allocator),
            auxiliary: DynamicArray::with_increment(3),
            super_edges: [0; 3],
            initialized_circles: false,
        };
        if fill {
            result.init_circles_tool(u, v)?;
        }
        Ok(result)
    }
    /// Retain live mesh identity; `triangulator.result()`.
    pub fn result(&self) -> &Handle<BRepMeshDataStructureOfDelaun> {
        &self.mesh
    }
    /// Borrow native circle storage; `triangulator.circles().is_empty()`.
    pub fn circles(&self) -> &BRepMeshCircleTool {
        &self.circles
    }
    /// Read a native node; `triangulator.get_vertex(1)?`.
    pub fn get_vertex(&self, index: i32) -> Result<BRepMeshVertex, Failure> {
        self.mesh_ref()?.get_node(index)
    }
    /// Read a native edge; `triangulator.get_edge(1)?`.
    pub fn get_edge(&self, index: i32) -> Result<BRepMeshEdge, Failure> {
        self.mesh_ref()?.get_link(index)
    }
    /// Read a native triangle; `triangulator.get_triangle(1)?`.
    pub fn get_triangle(&self, index: i32) -> Result<BRepMeshTriangle, Failure> {
        self.mesh_ref()?.get_element(index)
    }
    /// Native implementation always returns false; `triangulator.use_edge(1)`.
    pub fn use_edge(&self, _index: i32) -> bool {
        false
    }
    /// Copy auxiliary IDs for later deletion; `triangulator.set_aux_vertices(&indices)`.
    pub fn set_aux_vertices(&mut self, indices: &DynamicArray<i32>) {
        self.auxiliary = indices.clone();
    }
    /// Collect frontier edges in native hash order; `triangulator.frontier()?`.
    pub fn frontier(&self) -> Result<Map<i32>, Failure> {
        self.edges_by_type(DegreeOfFreedom::Frontier)
    }
    /// Collect fixed internal edges; `triangulator.internal_edges()?`.
    pub fn internal_edges(&self) -> Result<Map<i32>, Failure> {
        self.edges_by_type(DegreeOfFreedom::Fixed)
    }
    /// Native free-edge query selects every edge used at most once, regardless of movability; `triangulator.free_edges()?`.
    pub fn free_edges(&self) -> Result<Map<i32>, Failure> {
        self.edges_by_type(DegreeOfFreedom::Free)
    }
    fn mesh_ref(&self) -> Result<&BRepMeshDataStructureOfDelaun, Failure> {
        self.mesh.get().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "BRepMesh_Delaun mesh=null; expected an existing mesh",
                "",
            )
        })
    }
    fn edges_by_type(&self, kind: DegreeOfFreedom) -> Result<Map<i32>, Failure> {
        let mesh = self.mesh_ref()?;
        let mut result = Map::default();
        for index in mesh.links_of_domain().iter() {
            let add = if kind == DegreeOfFreedom::Free {
                mesh.elements_connected_to(*index)?.extent() <= 1
            } else {
                self.get_edge(*index)?.movability() == kind
            };
            if add {
                result.add(*index);
            }
        }
        Ok(result)
    }
}
