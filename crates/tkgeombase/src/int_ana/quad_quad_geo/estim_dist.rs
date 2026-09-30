// Rust translation of the file-local EstimDist of IntAna_QuadQuadGeo.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! EstimDist: the minimal distance from the apexes of two cones with parallel,
//! coplanar axes to their intersection.
use crate::gce::{MakePln, Root};
use crate::int_ana2d::AnaIntersection;
use crate::proj_lib::project_line_on_plane;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::gp::{Cone3, Line2, Line3, Plane3, Point3, Vector3};

/// EstimDist: in the plane of the two axes, the generatrices of both cones
/// (the axes rotated by ± the semi-angle) are intersected pairwise; the result
/// is the larger of the smallest |parameter| on each cone, or
/// Precision::Infinite() when the apexes and the axis do not span a plane.
pub(super) fn estim_dist(first: Cone3, second: Cone3) -> Result<f64, Failure> {
    let (apex1, apex2) = (first.apex(), second.apex());
    let third: Point3 = apex1.translated(Vector3::from(first.position().direction()));
    let maker = MakePln::through_points(apex1, apex2, third)?;
    if !maker.is_done() {
        return Ok(precision::infinite());
    }
    let plane: Plane3 = maker.value()?;
    let lines1 = generatrices(
        plane,
        Line3::new(apex1, first.position().direction()),
        first.semi_angle(),
    )?;
    let lines2 = generatrices(
        plane,
        Line3::new(apex2, second.position().direction()),
        second.semi_angle(),
    )?;
    let mut min_dist: [f64; 2] = [precision::infinite(), precision::infinite()];
    let mut inter = AnaIntersection::new();
    for line1 in lines1 {
        for line2 in lines2 {
            inter.perform_lines(line1, line2);
            if !inter.is_done() {
                continue;
            }
            for k in 1..=inter.nb_points()? {
                let point = inter.point(k)?;
                min_dist[0] = std_min(point.param_on_first().abs(), min_dist[0]);
                min_dist[1] = std_min(point.param_on_second()?.abs(), min_dist[1]);
            }
        }
    }
    Ok(std_max(min_dist[0], min_dist[1]))
}

/// The two generatrices of a cone in `plane`: the projected axis rotated by
/// `angle` and by `-angle` around its location.
fn generatrices(plane: Plane3, axis: Line3, angle: f64) -> Result<[Line2; 2], Failure> {
    let axis2d: Line2 = project_line_on_plane(&plane, &axis)?;
    Ok([
        axis2d.rotated(axis2d.location(), angle),
        axis2d.rotated(axis2d.location(), -angle),
    ])
}
