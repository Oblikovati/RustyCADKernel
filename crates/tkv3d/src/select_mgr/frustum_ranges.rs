//! Projection ranges shared by the frustum's separating-axis tests.
use super::SelectMgrFrustumCore;
use rustycad_tkernel::standard::real::{real_first, real_last};
use rustycad_tkmath::bvh::{std_max, std_min};
use rustycad_tkmath::gp::{Point3, Xyz};

pub(crate) fn refine_frustum_inside(
    inside: &mut Option<&mut bool>,
    inner: (f64, f64),
    outer: (f64, f64),
) {
    if let Some(flag) = inside.as_mut() {
        **flag &= inner.0 >= outer.0 && inner.1 <= outer.1;
    }
}
impl<const N: usize> SelectMgrFrustumCore<N> {
    /// The min and max projections of the frustum vertices on a direction.
    pub(crate) fn vertex_range(&self, direction: &Xyz, initial: (f64, f64)) -> (f64, f64) {
        self.vertices.iter().fold(initial, |(lo, hi), point| {
            let projection = point.xyz().dot(*direction);
            (std_min(lo, projection), std_max(hi, projection))
        })
    }
    /// The min and max projections of points on a direction (starting at RealLast/RealFirst).
    pub(crate) fn points_range(points: &[Point3], direction: &Xyz) -> (f64, f64) {
        points
            .iter()
            .fold((real_last(), real_first()), |(lo, hi), point| {
                let projection = direction.dot(point.xyz());
                (std_min(lo, projection), std_max(hi, projection))
            })
    }
    pub(crate) fn polygon_axis_overlap(&self, points: &[Point3], direction: &Xyz) -> bool {
        let (lo, hi) = Self::points_range(points, direction);
        let (min, max) = self.vertex_range(direction, (real_last(), real_first()));
        !(lo > max || hi < min)
    }
}
