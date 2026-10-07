//! Insert ordered vertices by replacing their connected circumcircle cavity.
use super::triangle_build::{LoopEdges, loop_edges};
use super::{BRepMeshDelaun, DegreeOfFreedom};
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::{
    bnd::BoundingBox2,
    gp::{Point2, Xy},
};

impl BRepMeshDelaun {
    /// `AddVertices(theVerticesIndices, theRange)`: sorts the indices in place by the
    /// native `x + y` heap order, inserts each into its circumcircle cavity, then
    /// processes the constraints; `delaun.add_vertices(&mut indices, &range)?`.
    pub fn add_vertices(
        &mut self,
        indices: &mut [i32],
        range: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        self.sort_vertex_indices(indices)?;
        self.create_triangles_on_new_vertices(indices, range)
    }

    /// `perform(theVertexIndices, theCellsCountU, theCellsCountV)`: nothing for two
    /// indices or fewer; else the circle grid over the indexed vertices, the super
    /// triangle, the sorted insertion and the removal of the auxiliary elements.
    pub(super) fn perform(
        &mut self,
        indices: &mut [i32],
        cells: (i32, i32),
    ) -> Result<(), Failure> {
        if indices.len() <= 2 {
            return Ok(());
        }
        let bounds = self.insertion_bounds(indices)?;
        self.initialize_circle_bounds(&bounds, cells.0, cells.1)?;
        let (_, super_edges) = self.build_super_mesh(bounds.get()?)?;
        self.sort_vertex_indices(indices)?;
        self.compute(indices, super_edges)
    }

    /// `compute`: the fan of the first vertex over the super triangle, every vertex
    /// (the first one again included, as natively), then RemoveAuxElements.
    fn compute(&mut self, indices: &[i32], super_edges: [i32; 3]) -> Result<(), Failure> {
        let mut boundary = super_triangle_edges(super_edges);
        if let Some(first) = indices.first() {
            self.create_triangles(*first, &mut boundary)?;
            self.create_triangles_on_new_vertices(indices, &ProgressRange::default())?;
        }
        self.remove_aux_elements()
    }

    /// `createTrianglesOnNewVertices`: the progress scope counts `Upper` (length less
    /// one) steps, as natively; a user break stops before ProcessConstraints.
    fn create_triangles_on_new_vertices(
        &mut self,
        indices: &[i32],
        range: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let (tolerance_u, tolerance_v) = self.mesh_ref()?.vertex_tool().get_tolerance();
        let squared_tolerance = tolerance_u * tolerance_u + tolerance_v * tolerance_v;
        let upper = indices.len() as f64 - 1.0;
        let scope =
            ProgressScope::with_literal(range, "Create triangles on new vertices", upper, false);
        for index in indices {
            if !scope.more() {
                return Ok(());
            }
            let vertex = self.get_vertex(*index)?;
            self.insert_vertex_in_circle_cavity(*index, &vertex, squared_tolerance)?;
            drop(scope.next(1.0));
        }
        self.process_constraints()
    }

    fn insert_vertex_in_circle_cavity(
        &mut self,
        vertex_id: i32,
        vertex: &super::BRepMeshVertex,
        squared_tolerance: f64,
    ) -> Result<(), Failure> {
        let mut boundary = loop_edges();
        let mut candidates = self.selected_circles(vertex.coord())?;
        let Some(triangle) =
            self.find_insert_triangle(vertex, squared_tolerance, &mut candidates)?
        else {
            return Ok(());
        };
        self.delete_triangle(triangle, &mut boundary)?;
        self.expand_cavity(&mut boundary, &mut candidates)?;
        self.create_triangles(vertex_id, &mut boundary)
    }

    fn selected_circles(&mut self, point: &Xy) -> Result<Vec<i32>, Failure> {
        Ok(self.circles.select(point)?.iter().copied().collect())
    }

    fn find_insert_triangle(
        &mut self,
        vertex: &super::BRepMeshVertex,
        squared_tolerance: f64,
        candidates: &mut Vec<i32>,
    ) -> Result<Option<i32>, Failure> {
        let mut candidate_index = 0;
        while candidate_index < candidates.len() {
            let triangle = candidates[candidate_index];
            if self.is_insert_candidate(triangle, vertex, squared_tolerance)? {
                candidates.remove(candidate_index);
                return Ok(Some(triangle));
            }
            candidate_index += 1;
        }
        Ok(None)
    }

    fn is_insert_candidate(
        &self,
        triangle: i32,
        vertex: &super::BRepMeshVertex,
        squared_tolerance: f64,
    ) -> Result<bool, Failure> {
        let mut edge_on = 0;
        if !self.contains(triangle, vertex, squared_tolerance, &mut edge_on)? {
            return Ok(false);
        }
        if edge_on != 0
            && self.get_edge(edge_on)?.movability() != DegreeOfFreedom::Free
            && vertex.movability() == DegreeOfFreedom::Free
        {
            return Ok(false);
        }
        Ok(true)
    }

    fn expand_cavity(
        &mut self,
        boundary: &mut LoopEdges,
        candidates: &mut Vec<i32>,
    ) -> Result<(), Failure> {
        while let Some((index, victim)) = self.find_adjacent_circle(candidates, boundary)? {
            self.delete_triangle(victim, boundary)?;
            candidates.remove(index);
        }
        Ok(())
    }

    fn find_adjacent_circle(
        &self,
        candidates: &[i32],
        boundary: &LoopEdges,
    ) -> Result<Option<(usize, i32)>, Failure> {
        for (index, triangle_id) in candidates.iter().copied().enumerate() {
            let triangle = self.get_triangle(triangle_id)?;
            if triangle.edges.iter().any(|edge| boundary.is_bound(edge)) {
                return Ok(Some((index, triangle_id)));
            }
        }
        Ok(None)
    }

    fn insertion_bounds(&self, indices: &[i32]) -> Result<BoundingBox2, Failure> {
        let mut bounds = BoundingBox2::default();
        for index in indices {
            bounds.add_point(Point2::from(*self.get_vertex(*index)?.coord()));
        }
        bounds.enlarge(rustycad_tkernel::precision::p_confusion());
        Ok(bounds)
    }
}

fn super_triangle_edges(edges: [i32; 3]) -> LoopEdges {
    let mut boundary = loop_edges();
    for edge in edges {
        boundary.bind(edge.abs(), true);
    }
    boundary
}
