use super::{BRepMeshDelaun, BRepMeshVertex, DegreeOfFreedom};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Xy;
struct TriangleDistances {
    cross: [f64; 3],
    squared_lengths: [f64; 3],
    nearest: usize,
    squared_min: f64,
}
impl BRepMeshDelaun {
    /// Native signed half-plane containment and nearest constrained edge; `triangulator.contains(1,&vertex,1e-18,&mut edge)?`.
    pub fn contains(
        &self,
        index: i32,
        vertex: &BRepMeshVertex,
        tolerance: f64,
        edge_on: &mut i32,
    ) -> Result<bool, Failure> {
        *edge_on = 0;
        let triangle = self.get_triangle(index)?;
        let edges = self.triangle_edges(&triangle)?;
        let points = self.triangle_points(&triangle)?;
        let Some(distances) = triangle_distances(&points, *vertex.coord()) else {
            return Ok(false);
        };
        Ok(contained_by_edges(
            &distances,
            tolerance,
            &edges,
            &triangle.edges,
            edge_on,
        ))
    }
    fn triangle_edges(
        &self,
        triangle: &super::BRepMeshTriangle,
    ) -> Result<[super::BRepMeshEdge; 3], Failure> {
        Ok([
            self.get_edge(triangle.edges[0])?,
            self.get_edge(triangle.edges[1])?,
            self.get_edge(triangle.edges[2])?,
        ])
    }
}
fn triangle_distances(points: &[Xy; 3], vertex: Xy) -> Option<TriangleDistances> {
    let precision = rustycad_tkernel::precision::p_confusion();
    let mut result = TriangleDistances {
        cross: [0.; 3],
        squared_lengths: [0.; 3],
        nearest: 0,
        squared_min: f64::MAX,
    };
    for i in 0..3 {
        if !measure_triangle_edge(
            &mut result,
            i,
            points[(i + 1) % 3] - points[i],
            vertex - points[i],
            precision * precision,
        ) {
            return None;
        }
    }
    Some(result)
}
fn classify_nearest(
    distances: &TriangleDistances,
    tolerance: f64,
    constrained: bool,
    index: i32,
    edge_on: &mut i32,
) -> bool {
    let n = distances.nearest;
    if distances.squared_min > tolerance {
        if constrained && distances.cross[n] < distances.squared_lengths[n] / 5. {
            *edge_on = index;
        }
        return true;
    }
    if constrained {
        return false;
    }
    *edge_on = index;
    true
}

fn contained_by_edges(
    distances: &TriangleDistances,
    tolerance: f64,
    edges: &[super::BRepMeshEdge; 3],
    indices: &[i32; 3],
    edge_on: &mut i32,
) -> bool {
    let nearest = distances.nearest;
    if !classify_nearest(
        distances,
        tolerance,
        edges[nearest].movability() != DegreeOfFreedom::Free,
        indices[nearest],
        edge_on,
    ) {
        return false;
    }
    distances.cross.iter().all(|distance| *distance >= 0.)
}

fn measure_triangle_edge(
    result: &mut TriangleDistances,
    index: usize,
    edge: Xy,
    relative: Xy,
    tolerance: f64,
) -> bool {
    result.squared_lengths[index] = edge.square_modulus();
    if result.squared_lengths[index] <= tolerance {
        return false;
    }
    result.cross[index] = edge.crossed(relative);
    let distance = result.cross[index] * result.cross[index] / result.squared_lengths[index];
    if distance < result.squared_min {
        result.nearest = index;
        result.squared_min = distance;
    }
    true
}
