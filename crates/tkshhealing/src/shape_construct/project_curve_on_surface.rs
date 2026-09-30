// ShapeConstruct_ProjectCurveOnSurface at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
mod analytic;
mod degenerate_adjustment;
mod extremity;
mod interpolation;
mod iso_sampling;
mod isoparametric;
mod line_fit;
mod perform;
mod periodicity;
mod point_projection;
mod sampling;
mod seam_insertion;
mod seams;
mod walking;

use crate::shape_analysis::ShapeAnalysisSurface;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::{impl_transient, precision};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkmath::gp::{Point2, Point3};

struct SurfaceCurveProjectionFields {
    surface: Handle<ShapeAnalysisSurface>,
    precision: f64,
    status: i32,
    adjust_over_degen: i32,
    cache: Vec<(Point3, Point2)>,
}

/// Projects a three-dimensional curve into a surface's parameter space.
/// `let projector = ShapeConstructProjectCurveOnSurface::new();`
pub struct ShapeConstructProjectCurveOnSurface {
    parent: StandardTransient,
    fields: SharedState<SurfaceCurveProjectionFields>,
}
impl_transient!(
    ShapeConstructProjectCurveOnSurface,
    "ShapeConstruct_ProjectCurveOnSurface",
    StandardTransient,
    parent
);

impl Default for ShapeConstructProjectCurveOnSurface {
    fn default() -> Self {
        let fields = SurfaceCurveProjectionFields {
            surface: Handle::default(),
            precision: precision::confusion(),
            status: 0,
            adjust_over_degen: 1,
            cache: Vec::new(),
        };
        Self {
            parent: StandardTransient::default(),
            fields: SharedState::new(fields),
        }
    }
}

impl ShapeConstructProjectCurveOnSurface {
    /// Creates an empty projector; `ShapeConstructProjectCurveOnSurface::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Initializes surface and precision; `projector.init(&surface,1e-7)?`.
    pub fn init(&self, surface: &Handle<dyn GeomSurface>, precision: f64) -> Result<(), Failure> {
        self.set_surface(surface)?;
        self.set_precision(precision);
        Ok(())
    }

    /// Initializes with an existing analysis object; `projector.init_analysis(&analysis,1e-7)`.
    pub fn init_analysis(&self, surface: &Handle<ShapeAnalysisSurface>, precision: f64) {
        self.set_analysis_surface(surface);
        self.set_precision(precision);
    }

    /// Creates an analyzer for a new surface; `projector.set_surface(&surface)?`.
    pub fn set_surface(&self, surface: &Handle<dyn GeomSurface>) -> Result<(), Failure> {
        let analysis = Handle::new(ShapeAnalysisSurface::new(surface)?);
        self.set_analysis_surface(&analysis);
        Ok(())
    }

    /// Changes analysis identity and clears only its endpoint cache; `projector.set_analysis_surface(&analysis)`.
    pub fn set_analysis_surface(&self, surface: &Handle<ShapeAnalysisSurface>) {
        let mut fields = self.fields.write();
        if fields.surface == *surface {
            return;
        }
        fields.surface = surface.clone();
        fields.cache.clear();
    }

    /// Changes projection precision; `projector.set_precision(1e-7)`.
    pub fn set_precision(&self, precision: f64) {
        self.fields.write().precision = precision;
    }
    /// Returns the modifiable native mode's value; `let mode=projector.adjust_over_degen_mode();`.
    pub fn adjust_over_degen_mode(&self) -> i32 {
        self.fields.read().adjust_over_degen
    }
    /// Mutates the native adjustment mode; `projector.set_adjust_over_degen_mode(0)`.
    pub fn set_adjust_over_degen_mode(&self, mode: i32) {
        self.fields.write().adjust_over_degen = mode;
    }
    /// Queries the last operation's status bits; `projector.status(ShapeExtendStatus::Done)`.
    pub fn status(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.fields.read().status, status)
    }

    fn analysis(&self) -> Handle<ShapeAnalysisSurface> {
        self.fields.read().surface.clone()
    }
    fn set_status(&self, status: ShapeExtendStatus) {
        self.fields.write().status = ShapeExtend::encode_status(status);
    }
    fn add_status(&self, status: ShapeExtendStatus) {
        self.fields.write().status |= ShapeExtend::encode_status(status);
    }
}

#[cfg(test)]
mod regression_tests;
