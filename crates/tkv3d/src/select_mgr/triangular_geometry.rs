//! Native edge ordering and projection caches for SelectMgr_TriangularFrustum.
use super::{SelectMgrFrustumBuilder, SelectMgrFrustumCore, SelectMgrSelectionTriangle};
use rustycad_tkmath::{
    bvh::{std_max, std_min},
    gp::{GeneralTransform3, Point3, Xyz},
};

pub(super) fn triangular_build_core(
    triangle: SelectMgrSelectionTriangle,
    builder: &SelectMgrFrustumBuilder,
) -> SelectMgrFrustumCore<3> {
    let mut core = SelectMgrFrustumCore::new();
    for (index, vertex) in core.vertices.iter_mut().enumerate() {
        let pixel = triangle.points[index % 3];
        *vertex = builder.project_pnt_on_view_plane(pixel.x(), pixel.y(), (index / 3) as f64);
    }
    triangular_cache_geometry(&mut core);
    core
}
pub(super) fn triangular_transformed_core(
    source: &SelectMgrFrustumCore<3>,
    transform: &GeneralTransform3,
) -> SelectMgrFrustumCore<3> {
    let mut core = SelectMgrFrustumCore::new();
    for (target, source) in core.vertices.iter_mut().zip(&source.vertices) {
        let mut xyz = source.xyz();
        transform.transforms(&mut xyz);
        *target = Point3::from(xyz);
    }
    triangular_cache_geometry(&mut core);
    core
}
fn triangular_cache_geometry(core: &mut SelectMgrFrustumCore<3>) {
    for (index, (first, last)) in [(0, 3), (1, 4), (2, 5), (1, 0), (2, 1), (2, 0)]
        .into_iter()
        .enumerate()
    {
        core.edge_dirs[index] = core.vertices[first].xyz() - core.vertices[last].xyz();
    }
    let edges = &core.edge_dirs;
    core.planes[0] = edges[0].crossed(edges[3]);
    core.planes[1] = edges[1].crossed(edges[4]);
    core.planes[2] = edges[0].crossed(edges[5]);
    core.planes[3] = edges[3].crossed(edges[4]);
    core.planes[4] = core.planes[3].reversed();
    triangular_plane_projections(core);
    triangular_coordinate_projections(core);
}
fn projection_limits(values: impl Iterator<Item = f64>) -> (f64, f64) {
    values.fold((f64::MAX, -f64::MAX), |(minimum, maximum), value| {
        // OCCT std::min/max keep the accumulator on equality, including signed zero.
        (std_min(minimum, value), std_max(maximum, value))
    })
}
fn triangular_plane_projections(core: &mut SelectMgrFrustumCore<3>) {
    for index in 0..5 {
        let normal: Xyz = core.planes[index];
        let limits = projection_limits(core.vertices.iter().map(|point| normal.dot(point.xyz())));
        core.min_verts_projections[index] = limits.0;
        core.max_verts_projections[index] = limits.1;
    }
}
fn triangular_coordinate_projections(core: &mut SelectMgrFrustumCore<3>) {
    for index in 0..3 {
        let limits = projection_limits(
            core.vertices
                .iter()
                .map(|point| point.xyz().coordinates()[index]),
        );
        core.min_ortho_verts_projections[index] = limits.0;
        core.max_ortho_verts_projections[index] = limits.1;
    }
}

// OCCT 3d097a0 tests boundary edges but falls through to false even if every edge overlaps.
pub(super) fn triangular_boundary_result(
    core: &SelectMgrFrustumCore<3>,
    points: &[Point3],
    ortho: bool,
) -> bool {
    for (index, point) in points.iter().enumerate() {
        let next = &points[(index + 1) % points.len()];
        if !core.has_segment_overlap(point, next, ortho) {
            return false;
        }
    }
    false
}
