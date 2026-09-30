use super::*;
use rustycad_tkernel::ncollection::{BaseAllocator, DataMap};
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag};
use std::sync::Arc;
// Retain the curve handle beside its address key, as the native Handle-keyed map does.
type CurveBinding = (Handle<dyn GeomCurve>, Cached<ProjectPointOnCurve>);
pub(super) struct CurveCache {
    allocator: BaseAllocator,
    entries: SharedState<DataMap<usize, CurveBinding>>,
}
impl CurveCache {
    pub(super) fn new(allocator: &BaseAllocator) -> Self {
        Self {
            allocator: allocator.clone(),
            entries: SharedState::new(DataMap::with_allocator(
                Default::default(),
                100,
                allocator.clone(),
            )),
        }
    }
    fn get(&self, curve: &Handle<dyn GeomCurve>) -> Result<Cached<ProjectPointOnCurve>, Failure> {
        let key = curve.cast::<StandardTransient>().as_ptr() as usize;
        let mut entries = self.entries.write();
        if let Some((_, projector)) = entries.seek(&key) {
            return Ok(projector.clone());
        }
        let mut projector = ProjectPointOnCurve::new();
        projector.init_curve(curve, curve.first_parameter(), curve.last_parameter())?;
        let projector = Arc::new(self.allocator.allocate_value(SharedState::new(projector)));
        entries.bind(key, (curve.clone(), projector.clone()));
        Ok(projector)
    }
}
impl Context {
    /// Returns a minimum-distance surface projector on face bounds; `context.proj_ps(&face)?`.
    pub fn proj_ps(&self, face: &Face) -> Result<Cached<ProjectPointOnSurf>, Failure> {
        self.surface_projectors.get(face, || {
            let mut projector = ProjectPointOnSurf::new();
            projector.init_surface_tolerance(
                &BRepTool::surface(face)?,
                self.uv_bounds(face)?,
                *self.tolerance.read(),
                ExtAlgo::Grad,
            )?;
            projector.set_extrema_flag(ExtFlag::Min);
            Ok(projector)
        })
    }
    /// Returns a projector on the edge's spatial curve and parameter range; `context.proj_pc(&edge)?`.
    pub fn proj_pc(&self, edge: &Edge) -> Result<Cached<ProjectPointOnCurve>, Failure> {
        self.edge_projectors.get(edge, || {
            let (curve, first, last) = BRepTool::curve(edge)?;
            let mut projector = ProjectPointOnCurve::new();
            projector.init_curve(&curve, first, last)?;
            Ok(projector)
        })
    }
    /// Caches by retained curve handle identity; `context.proj_pt(&curve)?`.
    pub fn proj_pt(
        &self,
        curve: &Handle<dyn GeomCurve>,
    ) -> Result<Cached<ProjectPointOnCurve>, Failure> {
        self.curve_projectors.get(curve)
    }
    /// Invalidates surface projectors even when the tolerance is unchanged; `context.set_pons_projection_tolerance(1e-10)`.
    pub fn set_pons_projection_tolerance(&self, value: f64) {
        *self.tolerance.write() = value;
        self.surface_projectors.clear();
    }
}
