// Rust translation of the BRepMesh_Delaun constructors, Init and RemoveVertex of
// BRepMesh_Delaun.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{BRepMeshCircleTool, BRepMeshDataStructureOfDelaun, BRepMeshDelaun, BRepMeshVertex};
use crate::brep_mesh::BRepMeshSelectorOfDataStructureOfDelaun;
use rustycad_tkernel::ncollection::{
    DataMap, DefaultHashPolicy, DynamicArray, IncrementalAllocatorObject, Sequence,
};
use rustycad_tkernel::standard::{Failure, Handle};

impl BRepMeshDelaun {
    /// An unfilled triangulator over `mesh` with a circle tool reserved for `reserved`
    /// circles (`myCircles(theVertices.Length(), allocator)`).
    fn reserved(mesh: &Handle<BRepMeshDataStructureOfDelaun>, reserved: i32) -> Self {
        let allocator = Handle::new(IncrementalAllocatorObject::new(
            crate::imesh_data::MEMORY_BLOCK_SIZE_HUGE,
        ));
        Self {
            mesh: mesh.clone(),
            circles: BRepMeshCircleTool::with_reserved(reserved, &allocator),
            auxiliary: DynamicArray::with_increment(3),
            super_edges: [0; 3],
            initialized_circles: false,
        }
    }

    /// `BRepMesh_Delaun(theVertices)`: the triangulation of the vertices in a new
    /// structure (null when there are two vertices or fewer, as natively);
    /// `BRepMeshDelaun::with_vertices(&vertices)?`.
    pub fn with_vertices(vertices: &[BRepMeshVertex]) -> Result<Self, Failure> {
        let count = vertices.len() as i32;
        let mut result = Self::reserved(&Handle::default(), count);
        if vertices.len() > 2 {
            let allocator = Handle::new(IncrementalAllocatorObject::new(
                crate::imesh_data::MEMORY_BLOCK_SIZE_HUGE,
            ));
            result.mesh = Handle::new(BRepMeshDataStructureOfDelaun::new(&allocator, count));
            result.init(vertices)?;
        }
        Ok(result)
    }

    /// `BRepMesh_Delaun(theOldMesh, theVertices)`: the vertices added to an existing
    /// structure and triangulated; `BRepMeshDelaun::with_mesh_vertices(&mesh, &vertices)?`.
    pub fn with_mesh_vertices(
        mesh: &Handle<BRepMeshDataStructureOfDelaun>,
        vertices: &[BRepMeshVertex],
    ) -> Result<Self, Failure> {
        let mut result = Self::reserved(mesh, vertices.len() as i32);
        if vertices.len() > 2 {
            result.init(vertices)?;
        }
        Ok(result)
    }

    /// `BRepMesh_Delaun(theOldMesh, theVertexIndices[, theCellsCountU, theCellsCountV])`:
    /// the triangulation of nodes already in the structure; the indices are sorted in
    /// place; `(-1, -1)` cells select the default grid;
    /// `BRepMeshDelaun::with_indices(&mesh, &mut indices, (-1, -1))?`.
    pub fn with_indices(
        mesh: &Handle<BRepMeshDataStructureOfDelaun>,
        indices: &mut [i32],
        cells: (i32, i32),
    ) -> Result<Self, Failure> {
        let mut result = Self::reserved(mesh, indices.len() as i32);
        result.perform(indices, cells)?;
        Ok(result)
    }

    /// `Init(theVertices)`: adds the vertices to the structure, then triangulates them;
    /// `delaun.init(&vertices)?`.
    pub fn init(&mut self, vertices: &[BRepMeshVertex]) -> Result<(), Failure> {
        let mut indices = Vec::with_capacity(vertices.len());
        for vertex in vertices {
            indices.push(self.mesh_ref()?.add_node(vertex, false)?);
        }
        self.perform(&mut indices, (-1, -1))
    }

    /// `RemoveVertex(theVertex)`: deletes the triangles around the node, walks their
    /// boundary into a polygon from the first loop edge, and meshes it again;
    /// `delaun.remove_vertex(&vertex)?`.
    pub fn remove_vertex(&mut self, vertex: &BRepMeshVertex) -> Result<(), Failure> {
        let selector = BRepMeshSelectorOfDataStructureOfDelaun::new(&self.mesh);
        selector.neighbours_of_vertex(vertex)?;
        // Native `IMeshData::MapOfIntegerInteger aLoopEdges;`: one bucket, which fixes
        // the first loop edge the walk starts from.
        let mut boundary = DataMap::with_hasher(DefaultHashPolicy, 1);
        for triangle in selector.elements().iter() {
            self.delete_triangle(triangle, &mut boundary)?;
        }
        let Some((first_edge, forward)) = boundary.items().next().map(|(e, f)| (*e, *f)) else {
            return Ok(());
        };
        let polygon = self.walk_loop(first_edge, forward, &mut boundary)?;
        let mut polygon = self.boxed_polygon(&polygon)?;
        self.mesh_polygon(&mut polygon, None)
    }

    /// The native boundary walk of RemoveVertex: from the pivot node, the first bound
    /// link of its link list continues the polygon; at most `Extent` steps.
    fn walk_loop(
        &self,
        first_edge: i32,
        forward: bool,
        boundary: &mut super::triangle_build::LoopEdges,
    ) -> Result<Sequence<i32>, Failure> {
        let mut steps = boundary.extent() as i32;
        let edge = self.get_edge(first_edge)?;
        let (mut first, mut pivot) = (edge.first_node(), edge.last_node());
        let mut polygon = Sequence::default();
        if !forward {
            std::mem::swap(&mut first, &mut pivot);
        }
        polygon.append(if forward { first_edge } else { -first_edge });
        boundary.unbind(&first_edge);
        let mut edge_id = first_edge;
        while pivot != first {
            if let Some((next, link)) = self.next_loop_link(pivot, edge_id, boundary)? {
                polygon.append(link);
                boundary.unbind(&link.abs());
                (pivot, edge_id) = (next, link.abs());
            }
            if steps <= 0 {
                break;
            }
            steps -= 1;
        }
        Ok(polygon)
    }

    /// The first link around `pivot`, other than `edge_id`, still in the boundary, with
    /// the native node choice: its first node (and a negative index) when it does not
    /// end at pivot, else its last node. Both choices are the pivot itself, so the
    /// native walk never advances and stops on its step count (a native quirk kept).
    fn next_loop_link(
        &self,
        pivot: i32,
        edge_id: i32,
        boundary: &super::triangle_build::LoopEdges,
    ) -> Result<Option<(i32, i32)>, Failure> {
        for link in self.mesh_ref()?.links_connected_to(pivot)?.iter() {
            if *link == edge_id || !boundary.is_bound(link) {
                continue;
            }
            let edge = self.get_edge(*link)?;
            if edge.last_node() != pivot {
                return Ok(Some((edge.first_node(), -*link)));
            }
            return Ok(Some((edge.last_node(), *link)));
        }
        Ok(None)
    }
}
