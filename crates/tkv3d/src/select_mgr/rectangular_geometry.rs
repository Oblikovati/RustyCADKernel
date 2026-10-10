//! Vertex ordering, edge directions and orthographic cache shortcuts in OCCT 3d097a0.
use super::{SelectMgrFrustumBuilder, SelectMgrFrustumCore};
use rustycad_tkmath::{
    bvh::{std_max, std_min},
    gp::{Point2, Xy},
};

pub(super) fn rectangular_pixel_bounds(mouse: Point2, tolerance: i32) -> (Point2, Point2) {
    let half = f64::from(tolerance) * 0.5;
    (
        Point2::from(Xy::new(mouse.x() - half, mouse.y() - half)),
        Point2::from(Xy::new(mouse.x() + half, mouse.y() + half)),
    )
}
pub(super) fn rectangular_compute_vertices(
    core: &mut SelectMgrFrustumCore<4>,
    bounds: (Point2, Point2),
    builder: &SelectMgrFrustumBuilder,
) {
    let (min, max) = bounds;
    for (index, vertex) in core.vertices.iter_mut().enumerate() {
        let x = if index < 4 { min.x() } else { max.x() };
        let y = if index % 4 < 2 { max.y() } else { min.y() };
        *vertex = builder.project_pnt_on_view_plane(x, y, (index % 2) as f64);
    }
    rectangular_compute_edges(core);
}
pub(super) fn rectangular_compute_edges(core: &mut SelectMgrFrustumCore<4>) {
    for (index, (first, last)) in [(4, 0), (2, 0), (2, 3), (6, 7), (0, 1), (4, 5)]
        .into_iter()
        .enumerate()
    {
        core.edge_dirs[index] = core.vertices[first].xyz() - core.vertices[last].xyz();
    }
}
pub(super) fn rectangular_cache_geometry(core: &mut SelectMgrFrustumCore<4>, ortho: bool) {
    let edges = &core.edge_dirs;
    core.planes[0] = edges[0].crossed(edges[4]);
    core.planes[1] = edges[2].crossed(edges[0]);
    core.planes[2] = edges[4].crossed(edges[1]);
    core.planes[3] = edges[1].crossed(edges[5]);
    core.planes[4] = edges[0].crossed(edges[1]);
    core.planes[5] = core.planes[4].reversed();
    if ortho {
        rectangular_orthographic_planes(core);
    } else {
        rectangular_all_planes(core);
    }
    rectangular_coordinate_projections(core);
}
fn rectangular_orthographic_planes(core: &mut SelectMgrFrustumCore<4>) {
    // Opposite faces are translates; native code only touches even cache entries.
    for (index, (first, last)) in [(0, 2), (2, 6), (3, 6)].into_iter().enumerate() {
        let index = index * 2;
        let a = core.planes[index].dot(core.vertices[first].xyz());
        let b = core.planes[index].dot(core.vertices[last].xyz());
        core.min_verts_projections[index] = std_min(a, b);
        core.max_verts_projections[index] = std_max(a, b);
    }
}
fn rectangular_all_planes(core: &mut SelectMgrFrustumCore<4>) {
    for index in 0..6 {
        let (mut min, mut max) = (f64::MAX, -f64::MAX);
        for vertex in &core.vertices {
            let projection = core.planes[index].dot(vertex.xyz());
            min = std_min(min, projection);
            max = std_max(max, projection);
        }
        core.min_verts_projections[index] = min;
        core.max_verts_projections[index] = max;
    }
}
fn rectangular_coordinate_projections(core: &mut SelectMgrFrustumCore<4>) {
    for index in 0..3 {
        let (mut min, mut max) = (f64::MAX, -f64::MAX);
        for vertex in &core.vertices {
            let coordinate = vertex.xyz().coordinates()[index];
            // Native operand order differs from triangular caching on equal signed zeros.
            min = std_min(coordinate, min);
            max = std_max(coordinate, max);
        }
        core.min_ortho_verts_projections[index] = min;
        core.max_ortho_verts_projections[index] = max;
    }
}
