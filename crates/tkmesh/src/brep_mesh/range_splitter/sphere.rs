// Rust translation of BRepMesh_SphereRangeSplitter.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::*;
use rustycad_tkgeombase::gcpnts::TangentialDeflection;
use rustycad_tkmath::bvh::BvhScalar;

/// `BRepMesh_SphereRangeSplitter`: a staggered grid of interior nodes on a sphere;
/// `SphereRangeSplitter::default()`.
#[derive(Default)]
pub struct SphereRangeSplitter {
    base: DefaultRangeSplitter,
}
impl std::ops::Deref for SphereRangeSplitter {
    type Target = DefaultRangeSplitter;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for SphereRangeSplitter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

/// `computeStep`: the step dividing `range` into `(int)(diff / step) + 1` parts and the
/// exclusive end of the range less Precision::PConfusion().
fn compute_step(range: (f64, f64), default_step: f64) -> (f64, f64) {
    let diff = range.1 - range.0;
    // Native `(int)(aDiff / theDefaultStep) + 1`: the truncating C++ conversion.
    let count = (diff / default_step).truncate_to_int().wrapping_add(1);
    (diff / f64::from(count), range.1 - precision::p_confusion())
}

impl SphereRangeSplitter {
    /// `GenerateSurfaceNodes`: rows along V with a step of 0.7 times the arc angular
    /// step of the radius, every other row shifted by half a U step;
    /// `splitter.generate_surface_nodes(&parameters)?`.
    pub fn generate_surface_nodes(
        &self,
        parameters: &MeshParameters,
    ) -> Result<SurfaceNodes, Failure> {
        let radius = self.get_surface().sphere()?.radius();
        let step = 0.7
            * TangentialDeflection::arc_angular_step(
                radius,
                self.face.get_deflection(),
                parameters.angle,
                parameters.min_size,
            )?;
        let step_v = compute_step(self.range_v, step);
        let step_u = compute_step(self.range_u, step);
        let mut nodes = new_surface_nodes();
        let half_du = step_u.0 * 0.5;
        let mut shift = false;
        let mut v = self.range_v.0 + step_v.0;
        while v < step_v.1 {
            shift = !shift;
            let mut u = self.range_u.0 + if shift { half_du } else { 0.0 };
            while u < step_u.1 {
                nodes.append(Point2::new(u, v));
                u += step_u.0;
            }
            v += step_v.0;
        }
        Ok(Handle::new(Shared::new(nodes)))
    }
}

#[cfg(test)]
mod tests {
    use super::compute_step;

    #[test]
    fn compute_step_divides_into_truncated_count_plus_one() {
        let (step, end) = compute_step((0.0, 1.0), 0.3);
        assert_eq!(step, 0.25);
        assert_eq!(end, 1.0 - rustycad_tkernel::precision::p_confusion());
    }
}
