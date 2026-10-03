//! Rust translation of `ShapeUpgrade_ConvertSurfaceToBezierBasis` at OCCT 3d097a0.
mod build;
mod compute;
mod swept;
type ParameterBounds = (f64, f64, f64, f64);

use crate::shape_extend::{ShapeExtend, ShapeExtendCompositeSurface};
use crate::shape_upgrade::{SplitSurface, SplitSurfaceData};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, SharedState, StandardTransient},
};
use rustycad_tkg3d::geom::GeomSurface;

#[derive(Clone, Copy)]
struct ConversionModes {
    plane: bool,
    revolution: bool,
    extrusion: bool,
    bspline: bool,
}
impl Default for ConversionModes {
    fn default() -> Self {
        Self {
            plane: true,
            revolution: true,
            extrusion: true,
            bspline: true,
        }
    }
}

/// Converts supported surfaces into a patch grid whose analytic spline patches use Bezier bases.
/// Example: `converter.init_surface(&surface)?; converter.perform(true)?;`.
pub struct ShapeUpgradeConvertSurfaceToBezierBasis {
    parent: StandardTransient,
    state: SplitSurfaceData,
    modes: SharedState<ConversionModes>,
    segments: SharedState<Handle<ShapeExtendCompositeSurface>>,
}
impl_transient!(ShapeUpgradeConvertSurfaceToBezierBasis, "ShapeUpgrade_ConvertSurfaceToBezierBasis", StandardTransient, parent, interfaces:[dyn SplitSurface]);
impl Default for ShapeUpgradeConvertSurfaceToBezierBasis {
    fn default() -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SplitSurfaceData::default(),
            modes: SharedState::new(ConversionModes::default()),
            segments: SharedState::new(Handle::default()),
        }
    }
}
impl ShapeUpgradeConvertSurfaceToBezierBasis {
    /// Creates a converter with all native conversion modes enabled.
    pub fn new() -> Self {
        Self::default()
    }
    /// Replaces plane conversion mode; `converter.set_plane_mode(false)`.
    pub fn set_plane_mode(&self, enabled: bool) {
        self.modes.write().plane = enabled;
    }
    /// Reads plane conversion mode; `converter.plane_mode()`.
    pub fn plane_mode(&self) -> bool {
        self.modes.read().plane
    }
    /// Replaces revolution conversion mode; `converter.set_revolution_mode(false)`.
    pub fn set_revolution_mode(&self, enabled: bool) {
        self.modes.write().revolution = enabled;
    }
    /// Reads revolution conversion mode; `converter.revolution_mode()`.
    pub fn revolution_mode(&self) -> bool {
        self.modes.read().revolution
    }
    /// Replaces extrusion conversion mode; `converter.set_extrusion_mode(false)`.
    pub fn set_extrusion_mode(&self, enabled: bool) {
        self.modes.write().extrusion = enabled;
    }
    /// Reads extrusion conversion mode; `converter.extrusion_mode()`.
    pub fn extrusion_mode(&self) -> bool {
        self.modes.read().extrusion
    }
    /// Replaces B-spline conversion mode; `converter.set_bspline_mode(false)`.
    pub fn set_bspline_mode(&self, enabled: bool) {
        self.modes.write().bspline = enabled;
    }
    /// Reads B-spline conversion mode; `converter.bspline_mode()`.
    pub fn bspline_mode(&self) -> bool {
        self.modes.read().bspline
    }
    /// Returns the intermediate native segment grid after Compute; null before Compute.
    pub fn segments(&self) -> Handle<ShapeExtendCompositeSurface> {
        self.segments.read().clone()
    }
    /// Initializes the inherited splitter on a surface and its natural bounds.
    pub fn init_surface(&self, surface: &Handle<dyn GeomSurface>) -> Result<(), Failure> {
        SplitSurface::init_surface(self, surface)
    }
    /// Initializes the inherited splitter with explicit parameter bounds.
    pub fn init_surface_bounds(
        &self,
        surface: &Handle<dyn GeomSurface>,
        bounds: (f64, f64, f64, f64),
    ) -> Result<(), Failure> {
        SplitSurface::init_surface_bounds(self, surface, bounds, 0.)
    }
    fn modes(&self) -> ConversionModes {
        *self.modes.read()
    }
    fn set_segments(&self, segments: ShapeExtendCompositeSurface) {
        *self.segments.write() = Handle::new(segments);
    }
    fn state(&self) -> &SplitSurfaceData {
        &self.state
    }
}
impl SplitSurface for ShapeUpgradeConvertSurfaceToBezierBasis {
    fn split_surface_data(&self) -> &SplitSurfaceData {
        &self.state
    }
    fn compute(&self, segment: bool) -> Result<(), Failure> {
        self.compute_conversion(segment)
    }
    fn build(&self, segment: bool) -> Result<(), Failure> {
        self.build_conversion(segment)
    }
}
