//! Insert ordered vertices by replacing their connected circumcircle cavity.
use super::triangle_build::{LoopEdges, loop_edges};
use super::{BRepMeshDelaun, DegreeOfFreedom};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::{
    bnd::BoundingBox2,
    gp::{Point2, Xy},
};

impl BRepMeshDelaun {
    /// Insert a batch by native coordinate order; `delaun.insert_vertices_probe(&mut indices)?`.
    #[doc(hidden)]
    pub fn insert_vertices_probe(&mut self, indices: &mut [i32]) -> Result<(), Failure> {
        self.sort_vertex_indices(indices)?;
        self.create_triangles_on_new_vertices(indices)
    }

    /// Perform the native supermesh, batch insertion, and auxiliary cleanup sequence.
    ///
    /// ```ignore
    /// delaun.triangulate_vertices_probe(&mut vertex_indices)?;
    /// ```
    #[doc(hidden)]
    pub fn triangulate_vertices_probe(&mut self, indices: &mut [i32]) -> Result<(), Failure> {
        if indices.len() <= 2 {
            return Ok(());
        }
        let bounds = self.insertion_bounds(indices)?;
        self.init_circles_tool(-1, -1)?;
        let (_, super_edges) = self.build_super_mesh(bounds)?;
        self.sort_vertex_indices(indices)?;
        let mut boundary = super_triangle_edges(super_edges);
        self.create_triangles(indices[0], &mut boundary)?;
        self.create_triangles_on_new_vertices(indices)?;
        self.remove_aux_elements()
    }

    fn create_triangles_on_new_vertices(&mut self, indices: &[i32]) -> Result<(), Failure> {
        let (tolerance_u, tolerance_v) = self.mesh_ref()?.vertex_tool().get_tolerance();
        let squared_tolerance = tolerance_u * tolerance_u + tolerance_v * tolerance_v;
        for index in indices {
            let vertex = self.get_vertex(*index)?;
            self.insert_vertex_in_circle_cavity(*index, &vertex, squared_tolerance)?;
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
        let Some(triangle) = self.find_insert_triangle(vertex, squared_tolerance)? else {
            return Ok(());
        };
        self.delete_triangle(triangle, &mut boundary)?;
        self.expand_cavity(vertex.coord(), &mut boundary)?;
        self.create_triangles(vertex_id, &mut boundary)
    }

    fn find_insert_triangle(
        &mut self,
        vertex: &super::BRepMeshVertex,
        squared_tolerance: f64,
    ) -> Result<Option<i32>, Failure> {
        let candidates = self
            .circles
            .select(vertex.coord())?
            .iter()
            .copied()
            .collect::<Vec<_>>();
        for triangle in candidates {
            if self.is_insert_candidate(triangle, vertex, squared_tolerance)? {
                self.remove_circle_candidate(vertex.coord(), triangle)?;
                return Ok(Some(triangle));
            }
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

    fn expand_cavity(&mut self, point: &Xy, boundary: &mut LoopEdges) -> Result<(), Failure> {
        loop {
            let candidates = self
                .circles
                .select(point)?
                .iter()
                .copied()
                .collect::<Vec<_>>();
            let Some(victim) = self.find_adjacent_circle(&candidates, boundary)? else {
                return Ok(());
            };
            self.remove_circle_candidate(point, victim)?;
            self.delete_triangle(victim, boundary)?;
        }
    }

    fn find_adjacent_circle(
        &self,
        candidates: &[i32],
        boundary: &LoopEdges,
    ) -> Result<Option<i32>, Failure> {
        for triangle_id in candidates {
            let triangle = self.get_triangle(*triangle_id)?;
            if triangle.edges.iter().any(|edge| boundary.is_bound(edge)) {
                return Ok(Some(*triangle_id));
            }
        }
        Ok(None)
    }

    fn remove_circle_candidate(&mut self, point: &Xy, triangle: i32) -> Result<(), Failure> {
        let selected = self.circles.select(point)?;
        let mut cursor = selected.cursor();
        while cursor.more() {
            if cursor.value(selected) == Some(&triangle) {
                selected.remove(&mut cursor)?;
                return Ok(());
            }
            cursor.next(selected);
        }
        Ok(())
    }

    fn insertion_bounds(&self, indices: &[i32]) -> Result<[f64; 4], Failure> {
        let mut bounds = BoundingBox2::default();
        for index in indices {
            bounds.add_point(Point2::from(*self.get_vertex(*index)?.coord()));
        }
        bounds.enlarge(rustycad_tkernel::precision::p_confusion());
        bounds.get()
    }
}

fn super_triangle_edges(edges: [i32; 3]) -> LoopEdges {
    let mut boundary = loop_edges();
    for edge in edges {
        boundary.bind(edge.abs(), true);
    }
    boundary
}
