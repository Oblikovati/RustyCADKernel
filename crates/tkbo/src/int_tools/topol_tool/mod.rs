// IntTools_TopolTool.hxx/.cxx at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Intersection-specific surface sampling with inherited rectangular topology.
use rustycad_tkernel::{
    impl_transient,
    ncollection::Array1,
    standard::{Failure, Handle, SharedState},
};
use rustycad_tkg3d::adaptor3d::{Adaptor3dSurface, Adaptor3dTopolTool};
use rustycad_tkmath::gp::{Point2, Point3};
mod analyse;
mod counts;
mod sampling;
#[derive(Default)]
struct SamplingState {
    surface: Handle<dyn Adaptor3dSurface>,
    counts: [i32; 2],
    base_counts: [i32; 2],
    origin: [f64; 2],
    step: [f64; 2],
    parameters: Option<[Array1<'static, f64>; 2]>,
}
/// `IntTools_TopolTool`; `TopolTool::with_surface(&surface)?`.
#[derive(Default)]
pub struct TopolTool {
    parent: Adaptor3dTopolTool,
    sampling: SharedState<SamplingState>,
}
impl_transient!(TopolTool, "IntTools_TopolTool", Adaptor3dTopolTool, parent);
impl std::ops::Deref for TopolTool {
    type Target = Adaptor3dTopolTool;
    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}
impl TopolTool {
    /// Initializes inherited restrictions and resets sampling; `TopolTool::with_surface(&surface)?`.
    pub fn with_surface(surface: &Handle<dyn Adaptor3dSurface>) -> Result<Self, Failure> {
        let tool = Self::default();
        tool.initialize_surface(surface)?;
        Ok(tool)
    }
    /// Preserves the native unimplemented overload; `tool.initialize().is_err()`.
    pub fn initialize(&self) -> Result<(), Failure> {
        Err(Failure::with_type(
            "Standard_NotImplemented",
            "IntTools_TopolTool::Initialize ()",
            "",
        ))
    }
    /// Reinitializes the surface and sample caches; `tool.initialize_surface(&surface)?`.
    pub fn initialize_surface(
        &self,
        surface: &Handle<dyn Adaptor3dSurface>,
    ) -> Result<(), Failure> {
        self.parent.initialize_surface(surface)?;
        let mut state = self.sampling.write();
        state.surface = surface.clone();
        state.counts = [0; 2];
        state.base_counts[0] = -1;
        state.step = [1.0; 2];
        // Native Initialize preserves any previously allocated parameter arrays.
        Ok(())
    }
    /// Computes interior grid counts for every native surface kind; `tool.compute_sample_points()?`.
    pub fn compute_sample_points(&self) -> Result<(), Failure> {
        let surface = self.sampling.read().surface.clone();
        let bounds = counts::finite_bounds(&*surface)?;
        let counts = counts::sample_counts(&*surface, bounds)?;
        let mut state = self.sampling.write();
        state.counts = counts;
        state.base_counts = counts;
        state.origin = [bounds[0], bounds[2]];
        state.step = [
            (bounds[1] - bounds[0]) / f64::from(counts[0] + 1),
            (bounds[3] - bounds[2]) / f64::from(counts[1] + 1),
        ];
        Ok(())
    }
    fn counts(&self) -> Result<[i32; 2], Failure> {
        if self.sampling.read().counts[0] <= 0 {
            self.compute_sample_points()?;
        }
        Ok(self.sampling.read().counts)
    }
    /// Reads the lazy U count; `tool.nb_samples_u()?`.
    pub fn nb_samples_u(&self) -> Result<i32, Failure> {
        Ok(self.counts()?[0])
    }
    /// Reads the lazy V count; `tool.nb_samples_v()?`.
    pub fn nb_samples_v(&self) -> Result<i32, Failure> {
        Ok(self.counts()?[1])
    }
    /// Reads the lazy total; `tool.nb_samples()?`.
    pub fn nb_samples(&self) -> Result<i32, Failure> {
        let counts = self.counts()?;
        Ok(counts[0].wrapping_mul(counts[1]))
    }
}
