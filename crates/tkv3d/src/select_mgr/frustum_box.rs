//! Axis-aligned box/frustum SAT and native full-inside flag refinement.
use super::{SelectMgrFrustumCore, frustum_ranges::refine_frustum_inside};
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::standard::{Failure, exceptions::program_error};
use rustycad_tkmath::{
    bvh::{std_max, std_min},
    gp::Xyz,
};

/// The coordinate of a vector along an axis index.
fn axis_of(v: &Vec3<f64>, axis: usize) -> f64 {
    match axis {
        0 => v.x,
        1 => v.y,
        _ => v.z,
    }
}
/// The projections of a box on a direction: the minimum and maximum corners chosen by sign.
fn box_projection(dir: &Xyz, min: &Vec3<f64>, max: &Vec3<f64>) -> Result<(f64, f64), Failure> {
    // The native comparison is against `0.f`.
    let pick = |d: f64, lo: f64, hi: f64| if d < 0.0 { hi } else { lo };
    let low = dir.x() * pick(dir.x(), min.x, max.x)
        + dir.y() * pick(dir.y(), min.y, max.y)
        + dir.z() * pick(dir.z(), min.z, max.z);
    let high = dir.x() * pick(dir.x(), max.x, min.x)
        + dir.y() * pick(dir.y(), max.y, min.y)
        + dir.z() * pick(dir.z(), max.z, min.z);
    if high.partial_cmp(&low).is_none_or(|order| order.is_lt()) {
        return Err(program_error(&format!(
            "box corners {min:?}..{max:?}, direction {dir:?}: expected ordered finite projections"
        )));
    }
    Ok((low, high))
}
fn cross_box_axis(dim: usize, edge: Xyz) -> Xyz {
    let (next, last) = ((dim + 1) % 3, (dim + 2) % 3);
    let mut direction = [0.0; 3];
    direction[next] = -edge.coordinates()[last];
    direction[last] = edge.coordinates()[next];
    Xyz::new(direction[0], direction[1], direction[2])
}
impl<const N: usize> SelectMgrFrustumCore<N> {
    fn box_axis_range(
        &self,
        direction: &Xyz,
        low: f64,
        high: f64,
        inclusion: bool,
    ) -> Option<(f64, f64)> {
        let (mut min, mut max) = (f64::MAX, -f64::MAX);
        for vertex in &self.vertices {
            let projection = vertex.xyz().dot(*direction);
            min = std_min(min, projection);
            max = std_max(max, projection);
            if min <= high && max >= low && !inclusion {
                return None;
            }
        }
        Some((min, max))
    }
    /// `isSeparated(box, direction, inside)`: refines the full-inside flag.
    fn is_separated_box(
        &self,
        min: &Vec3<f64>,
        max: &Vec3<f64>,
        direction: &Xyz,
        inside: &mut Option<&mut bool>,
    ) -> Result<bool, Failure> {
        let (low, high) = box_projection(direction, min, max)?;
        let inclusion = inside.as_ref().is_some_and(|flag| **flag);
        let Some((min_f, max_f)) = self.box_axis_range(direction, low, high, inclusion) else {
            return Ok(false);
        };
        if min_f > high || max_f < low {
            return Ok(true);
        }
        refine_frustum_inside(inside, (low, high), (min_f, max_f));
        Ok(false)
    }
    /// The axis-aligned test of `hasBoxOverlap`; false when separated.
    fn box_ortho_overlap(
        &self,
        min: &Vec3<f64>,
        max: &Vec3<f64>,
        inside: &mut Option<&mut bool>,
    ) -> bool {
        for axis in 0..3 {
            let (lo, hi) = (axis_of(min, axis), axis_of(max, axis));
            let (lower, upper) = (
                self.min_ortho_verts_projections[axis],
                self.max_ortho_verts_projections[axis],
            );
            if lo > upper || hi < lower {
                return false;
            }
            refine_frustum_inside(inside, (lo, hi), (lower, upper));
        }
        true
    }
    /// The frustum plane test of `hasBoxOverlap`; false when separated.
    fn box_planes_overlap(
        &self,
        min: &Vec3<f64>,
        max: &Vec3<f64>,
        ortho: bool,
        inside: &mut Option<&mut bool>,
    ) -> Result<bool, Failure> {
        for index in (0..N + 1).step_by(self.inc_factor(ortho)) {
            let (lo, hi) = box_projection(&self.planes[index], min, max)?;
            let (lower, upper) = (
                self.min_verts_projections[index],
                self.max_verts_projections[index],
            );
            if lo > upper || hi < lower {
                return Ok(false);
            }
            refine_frustum_inside(inside, (lo, hi), (lower, upper));
        }
        Ok(true)
    }
    fn box_cross_axes_overlap(
        &self,
        min: &Vec3<f64>,
        max: &Vec3<f64>,
        ortho: bool,
        inside: &mut Option<&mut bool>,
    ) -> Result<bool, Failure> {
        let count = Self::directions_nb(ortho);
        for index in 0..3 * count {
            let direction = cross_box_axis(index / count, self.edge_dirs[index % count]);
            if self.is_separated_box(min, max, &direction, inside)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
    /// `hasBoxOverlap`: the separating axis test, refining full-inside; bad projections raise ProgramError.
    /// Example: `core.has_box_overlap(&min, &max, false, None)?`.
    pub fn has_box_overlap(
        &self,
        min: &Vec3<f64>,
        max: &Vec3<f64>,
        ortho: bool,
        mut inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        if !self.box_ortho_overlap(min, max, &mut inside)
            || !self.box_planes_overlap(min, max, ortho, &mut inside)?
        {
            return Ok(false);
        }
        self.box_cross_axes_overlap(min, max, ortho, &mut inside)
    }
}
