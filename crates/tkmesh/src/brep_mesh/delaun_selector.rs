//! Native selector accumulation over the planar mesh connectivity graph.
use super::{BRepMeshDataStructureOfDelaun, BRepMeshEdge, BRepMeshTriangle, BRepMeshVertex};
use rustycad_tkernel::{
    impl_transient,
    ncollection::Map,
    standard::{Failure, Handle, SharedState, StandardTransient},
};
#[derive(Default)]
struct DelaunSelection {
    mesh: Handle<BRepMeshDataStructureOfDelaun>,
    nodes: Map<i32>,
    links: Map<i32>,
    elements: Map<i32>,
    frontier: Map<i32>,
}
/// Accumulating native neighborhood selection; `BRepMeshSelectorOfDataStructureOfDelaun::new(&mesh)`.
#[derive(Default)]
pub struct BRepMeshSelectorOfDataStructureOfDelaun {
    transient: StandardTransient,
    selection: SharedState<DelaunSelection>,
}
impl_transient!(
    BRepMeshSelectorOfDataStructureOfDelaun,
    "BRepMesh_SelectorOfDataStructureOfDelaun",
    StandardTransient,
    transient
);
impl BRepMeshSelectorOfDataStructureOfDelaun {
    /// Retain the shared mesh without selecting anything; `BRepMeshSelectorOfDataStructureOfDelaun::new(&mesh)`.
    pub fn new(mesh: &Handle<BRepMeshDataStructureOfDelaun>) -> Self {
        Self {
            transient: StandardTransient::default(),
            selection: SharedState::new(DelaunSelection {
                mesh: mesh.clone(),
                ..Default::default()
            }),
        }
    }
    /// Replace the mesh and clear all four result maps; `selector.initialize(&mesh)`.
    pub fn initialize(&self, mesh: &Handle<BRepMeshDataStructureOfDelaun>) {
        let mut selection = self.selection.write();
        selection.mesh = mesh.clone();
        selection.nodes.clear(false);
        selection.links.clear(false);
        selection.elements.clear(false);
        selection.frontier.clear(false);
    }
    /// Select through the spatially coincident node's incident links; `selector.neighbours_of_vertex(&vertex)?`.
    pub fn neighbours_of_vertex(&self, vertex: &BRepMeshVertex) -> Result<(), Failure> {
        let mut selection = self.selection.write();
        let index = selection.mesh()?.index_of_node(vertex)?;
        selection.neighbours_of_node(index)
    }
    /// Accumulate elements attached to every link incident to a node; `selector.neighbours_of_node(1)?`.
    pub fn neighbours_of_node(&self, index: i32) -> Result<(), Failure> {
        self.selection.write().neighbours_of_node(index)
    }
    /// Select all elements at either endpoint, including elements not using the edge; `selector.neighbours_of_edge(&edge)?`.
    pub fn neighbours_of_edge(&self, edge: &BRepMeshEdge) -> Result<(), Failure> {
        self.selection.write().neighbours_of_edge(edge)
    }
    /// Select through both endpoints of a stored link; `selector.neighbours_of_link(1)?`.
    pub fn neighbours_of_link(&self, index: i32) -> Result<(), Failure> {
        let mut selection = self.selection.write();
        let edge = selection.mesh()?.get_link(index)?;
        selection.neighbours_of_edge(&edge)
    }
    /// Select through the triangle's three native derived nodes; `selector.neighbours_of_triangle(&triangle)?`.
    pub fn neighbours_of_triangle(&self, triangle: &BRepMeshTriangle) -> Result<(), Failure> {
        self.selection.write().neighbours_of_triangle(triangle)
    }
    /// Select through the vertices of a stored triangle; `selector.neighbours_of_element(1)?`.
    pub fn neighbours_of_element(&self, index: i32) -> Result<(), Failure> {
        let mut selection = self.selection.write();
        let triangle = selection.mesh()?.get_element(index)?;
        selection.neighbours_of_triangle(&triangle)
    }
    /// Select only elements sharing the given triangle's links; `selector.neighbours_by_edge_of(&triangle)?`.
    pub fn neighbours_by_edge_of(&self, triangle: &BRepMeshTriangle) -> Result<(), Failure> {
        let mut selection = self.selection.write();
        for edge in triangle.edges {
            selection.elements_of_link(edge)?;
        }
        Ok(())
    }
    /// Preserve the native empty selector-overload body; `selector.neighbours_of_selector(&other)`.
    pub fn neighbours_of_selector(&self, _other: &Self) {}
    /// Preserve the native empty expansion body; `selector.add_neighbours()`.
    pub fn add_neighbours(&self) {}
    /// Snapshot the native node map (selection does not populate it); `selector.nodes()`.
    pub fn nodes(&self) -> Map<i32> {
        self.selection.read().nodes.clone()
    }
    /// Snapshot the native link map (selection does not populate it); `selector.links()`.
    pub fn links(&self) -> Map<i32> {
        self.selection.read().links.clone()
    }
    /// Snapshot accumulated element indices in native hash order; `selector.elements()`.
    pub fn elements(&self) -> Map<i32> {
        self.selection.read().elements.clone()
    }
    /// Snapshot the native frontier map (selection does not populate it); `selector.frontier_links()`.
    pub fn frontier_links(&self) -> Map<i32> {
        self.selection.read().frontier.clone()
    }
}
impl DelaunSelection {
    fn mesh(&self) -> Result<&BRepMeshDataStructureOfDelaun, Failure> {
        self.mesh.get().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "BRepMesh selector mesh is null; expected an initialized mesh handle",
                "",
            )
        })
    }
    fn neighbours_of_node(&mut self, index: i32) -> Result<(), Failure> {
        let links = self.mesh()?.links_connected_to(index)?;
        for link in links.iter() {
            self.elements_of_link(*link)?;
        }
        Ok(())
    }
    fn neighbours_of_edge(&mut self, edge: &BRepMeshEdge) -> Result<(), Failure> {
        self.neighbours_of_node(edge.first_node())?;
        self.neighbours_of_node(edge.last_node())
    }
    fn neighbours_of_triangle(&mut self, triangle: &BRepMeshTriangle) -> Result<(), Failure> {
        let nodes = self.mesh()?.element_nodes(triangle)?;
        for node in nodes {
            self.neighbours_of_node(node)?;
        }
        Ok(())
    }
    fn elements_of_link(&mut self, index: i32) -> Result<(), Failure> {
        let pair = self.mesh()?.elements_connected_to(index)?;
        for position in 1..=pair.extent() {
            self.elements.add(pair.index(position)?);
        }
        Ok(())
    }
}
