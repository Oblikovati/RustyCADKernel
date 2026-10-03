//! ShapeUpgrade_FixSmallBezierCurves at OCCT 3d097a0328e71b8.
use super::{
    FixSmallCurves, FixSmallCurvesData, ShapeUpgradeFixSmallCurves, ShapeUpgradeSplitCurve2d,
    SplitCurve,
};
use crate::{shape_analysis::ShapeAnalysisEdge, shape_extend::ShapeExtendStatus};
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{self, Edge, Face},
};
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::{
    adaptor3d::Adaptor3dSurface,
    geom::{GeomCurve, GeomTrimmedCurve},
    geom_adaptor::GeomAdaptorSurface,
};
use rustycad_tkgeombase::{
    geom_convert::ApproxCurve as ApproxCurve3d, geom2d_convert::ApproxCurve as ApproxCurve2d,
};
use rustycad_tkmath::geom_abs::Shape;
mod planar;
mod spatial;

/// Approximate small curve portions through the configured splitting tools.
/// `let fixer = ShapeUpgradeFixSmallBezierCurves::new();`
#[derive(Default)]
pub struct ShapeUpgradeFixSmallBezierCurves {
    parent: ShapeUpgradeFixSmallCurves,
}
impl_transient!(ShapeUpgradeFixSmallBezierCurves,"ShapeUpgrade_FixSmallBezierCurves",ShapeUpgradeFixSmallCurves,parent,interfaces:[dyn FixSmallCurves]);
impl std::ops::Deref for ShapeUpgradeFixSmallBezierCurves {
    type Target = ShapeUpgradeFixSmallCurves;
    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}
impl ShapeUpgradeFixSmallBezierCurves {
    /// Create native null splitting tools and OK status; `ShapeUpgradeFixSmallBezierCurves::new()`.
    pub fn new() -> Self {
        Self::default()
    }
}
impl FixSmallCurves for ShapeUpgradeFixSmallBezierCurves {
    fn fix_small_curves_data(&self) -> &FixSmallCurvesData {
        self.parent.fix_small_curves_data()
    }
    fn approx(
        &self,
        curve_3d: &mut Handle<dyn GeomCurve>,
        curve_2d: &mut Handle<dyn Geom2dCurve>,
        reversed: &mut Handle<dyn Geom2dCurve>,
        first: &mut f64,
        last: &mut f64,
    ) -> Result<bool, Failure> {
        let edge = self.fix_small_curves_data().with(|f| f.edge.clone());
        if !self.approximate_spatial(&edge, curve_3d, first, last)? {
            return Ok(false);
        }
        let face = self.fix_small_curves_data().with(|f| f.face.clone());
        if face.is_null() {
            return Ok(true);
        }
        self.approximate_planar_pair(&edge, &face, curve_2d, reversed, first, last)
    }
}
fn clamp_range(first: &mut f64, last: &mut f64, bounds: (f64, f64)) {
    if *first < bounds.0 {
        *first = bounds.0;
    }
    if *last > bounds.1 {
        *last = bounds.1;
    }
}
fn parameter_tolerance(face: &Face, precision: f64) -> Result<f64, Failure> {
    let (surface, _) = BRepTool::surface_with_location(face)?;
    let adaptor = GeomAdaptorSurface::with_surface(&surface)?;
    let u = adaptor.u_resolution(precision)?;
    let v = adaptor.v_resolution(precision)?;
    Ok(if u < v { v } else { u })
}

fn assign_single_segment<T: Clone + Send + Sync + 'static>(
    segments: &Handle<HArray1<T>>,
    output: &mut T,
) -> Result<bool, Failure> {
    if segments.array1().len() > 1 {
        return Ok(false);
    }
    *output = segments
        .array1()
        .value(1)
        .map_err(|e| e.into_failure("ShapeUpgrade_FixSmallBezierCurves"))?
        .clone();
    Ok(true)
}
fn clamp_reversed(first: &mut f64, last: &mut f64, bounds: (f64, f64)) {
    // OCCT 3d097a0 reuses the forward bounds and reverses only the First comparison.
    if *first > bounds.0 {
        *first = bounds.0;
    }
    if *last > bounds.1 {
        *last = bounds.1;
    }
}
