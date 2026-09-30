//! Virtual splitter hooks shared by native surface-specific implementations.
use super::*;
/// Polymorphic splitter contract; `let splitter: &dyn RangeSplitter = &cone`.
pub trait RangeSplitter {
    /// Read the common range state; `splitter.range_state()`.
    fn range_state(&self) -> &DefaultRangeSplitter;
    /// Change common range state; `splitter.range_state_mut().add_point(point)`.
    fn range_state_mut(&mut self) -> &mut DefaultRangeSplitter;
    /// Initialize a border collection; `splitter.reset(&face, &parameters)`.
    fn reset(&mut self, face: &FaceHandle, parameters: &MeshParameters) {
        self.range_state_mut().reset(face, parameters);
    }
    /// Accumulate a border point; `splitter.add_point(point)`.
    fn add_point(&mut self, point: Point2) {
        self.range_state_mut().add_point(point);
    }
    /// Adjust bounds and dispatch tolerance/scaling overrides; `splitter.adjust_range()?`.
    fn adjust_range(&mut self) -> Result<(), Failure> {
        let Some((length_u, length_v)) = self.range_state_mut().adjust_lengths()? else {
            return Ok(());
        };
        self.compute_tolerance(length_u, length_v)?;
        self.compute_delta(length_u, length_v);
        Ok(())
    }
    /// Report the last range computation; `splitter.is_valid()`.
    fn is_valid(&self) -> bool {
        self.range_state().is_valid()
    }
    /// Generate surface-specific interior nodes; `splitter.generate_surface_nodes(&parameters)?`.
    fn generate_surface_nodes(&self, parameters: &MeshParameters) -> Result<SurfaceNodes, Failure> {
        self.range_state().generate_surface_nodes(parameters)
    }
    /// Override native parametric tolerance calculation; `splitter.compute_tolerance(1.0, 1.0)?`.
    fn compute_tolerance(&mut self, length_u: f64, length_v: f64) -> Result<(), Failure> {
        self.range_state_mut().compute_tolerance(length_u, length_v)
    }
    /// Override native physical scaling; `splitter.compute_delta(1.0, 1.0)`.
    fn compute_delta(&mut self, length_u: f64, length_v: f64) {
        self.range_state_mut().compute_delta(length_u, length_v);
    }
}
impl RangeSplitter for DefaultRangeSplitter {
    fn range_state(&self) -> &DefaultRangeSplitter {
        self
    }
    fn range_state_mut(&mut self) -> &mut DefaultRangeSplitter {
        self
    }
}
impl RangeSplitter for ConeRangeSplitter {
    fn range_state(&self) -> &DefaultRangeSplitter {
        self
    }
    fn range_state_mut(&mut self) -> &mut DefaultRangeSplitter {
        self
    }
    fn generate_surface_nodes(&self, parameters: &MeshParameters) -> Result<SurfaceNodes, Failure> {
        ConeRangeSplitter::generate_surface_nodes(self, parameters)
    }
}
