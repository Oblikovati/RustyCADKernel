use super::*;
use crate::imesh_data::MEMORY_BLOCK_SIZE_HUGE;
use rustycad_tkernel::ncollection::IncrementalAllocatorObject;
use rustycad_tkgeombase::gcpnts::TangentialDeflection;
use rustycad_tkmath::bvh::BvhScalar;
/// Conical interior-node distribution; `ConeRangeSplitter::default()`.
#[derive(Default)]
pub struct ConeRangeSplitter {
    base: DefaultRangeSplitter,
}
impl std::ops::Deref for ConeRangeSplitter {
    type Target = DefaultRangeSplitter;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for ConeRangeSplitter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl ConeRangeSplitter {
    /// Compute native U/V grid spacing and integer counts; `splitter.get_split_steps(&parameters,&mut counts)?`.
    pub fn get_split_steps(
        &self,
        parameters: &MeshParameters,
        counts: &mut (i32, i32),
    ) -> Result<(f64, f64), Failure> {
        let (step, radius) = self.angular_step(parameters)?;
        let diff_u = self.range_u.1 - self.range_u.0;
        let diff_v = self.range_v.1 - self.range_v.0;
        let scale = step * radius;
        let ratio = std_max(1.0, (diff_v / scale).ln());
        *counts = (
            (diff_u / step).truncate_to_int(),
            (diff_v / scale / ratio).truncate_to_int(),
        );
        Ok((
            diff_u / f64::from(counts.0.wrapping_add(1)),
            diff_v / f64::from(counts.1.wrapping_add(ratio.truncate_to_int())),
        ))
    }
    fn angular_step(&self, parameters: &MeshParameters) -> Result<(f64, f64), Failure> {
        let cone = self.get_surface().cone()?;
        let radius = std_max(
            (cone.ref_radius() + self.range_v.0 * cone.semi_angle().sin()).abs(),
            (cone.ref_radius() + self.range_v.1 * cone.semi_angle().sin()).abs(),
        );
        let step = TangentialDeflection::arc_angular_step(
            radius,
            self.face.get_deflection(),
            parameters.angle,
            parameters.min_size,
        )?;
        Ok((step, radius))
    }
    /// Generate rows in native V-then-U order; `splitter.generate_surface_nodes(&parameters)?`.
    pub fn generate_surface_nodes(
        &self,
        parameters: &MeshParameters,
    ) -> Result<SurfaceNodes, Failure> {
        let steps = self.get_split_steps(parameters, &mut (0, 0))?;
        let allocator = IncrementalAllocatorObject::new(MEMORY_BLOCK_SIZE_HUGE);
        let mut nodes = List::with_allocator(allocator.resource().clone());
        let max_v = self.range_v.1 - steps.1 * 0.5;
        let max_u = self.range_u.1 - steps.0 * 0.5;
        let mut v = self.range_v.0 + steps.1;
        while v < max_v {
            let mut u = self.range_u.0 + steps.0;
            while u < max_u {
                nodes.append(Point2::new(u, v));
                u += steps.0;
            }
            v += steps.1;
        }
        Ok(Handle::new(Shared::new(nodes)))
    }
}
