//! CheckMixedContinuity, preserving the upstream C0Value classification criterion.
use rustycad_tkbrep::{
    brep::BRepTool,
    brep_tools::FaceSurfaceCurves,
    topods::{Edge, Face},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::local_analysis::{
    LocalAnalysisSurfaceContinuity, SurfaceContinuityTolerances,
};
use rustycad_tkmath::geom_abs::Shape;
use std::f64::consts::FRAC_PI_2;
#[derive(Default)]
struct MixedEvidence {
    valid: usize,
    convex: bool,
    concave: bool,
}
pub(super) fn check_mixed_continuity(
    edge: &Edge,
    first: &Face,
    second: &Face,
    angle: f64,
) -> Result<bool, Failure> {
    if BRepTool::continuity_on_faces(edge, first, second)? != Shape::C0 {
        return Ok(false);
    }
    let tolerance = (1.5 * BRepTool::edge_tolerance(edge)?).max(0.001);
    let Some(curves) = FaceSurfaceCurves::load(edge, first, second)? else {
        return Ok(false);
    };
    let tolerances = SurfaceContinuityTolerances {
        eps_c0: tolerance,
        eps_c1: angle,
        eps_c2: angle,
        eps_g1: angle,
        ..Default::default()
    };
    sample_mixed_continuity(&curves, &tolerances, angle)
}
fn sample_mixed_continuity(
    curves: &FaceSurfaceCurves,
    tolerances: &SurfaceContinuityTolerances,
    angle: f64,
) -> Result<bool, Failure> {
    let mut evidence = MixedEvidence::default();
    let delta = (curves.last - curves.first) / 22.;
    for index in 0..23 {
        let parameter = if index == 22 {
            curves.last
        } else {
            curves.first + f64::from(index) * delta
        };
        let analysis = LocalAnalysisSurfaceContinuity::on_curves(
            (&curves.first_curve, &curves.second_curve),
            parameter,
            &curves.first_surface,
            &curves.second_surface,
            Shape::G1,
            tolerances,
        )?;
        evidence.observe(&analysis, angle)?;
    }
    Ok(evidence.valid >= 11 && evidence.convex && evidence.concave)
}
impl MixedEvidence {
    fn observe(
        &mut self,
        analysis: &LocalAnalysisSurfaceContinuity,
        angle: f64,
    ) -> Result<(), Failure> {
        if !analysis.is_done() {
            return Ok(());
        }
        self.valid += 1;
        if analysis.is_g1()? || (self.convex && self.concave) {
            return Ok(());
        }
        // Native reads C0Value (distance), despite naming the local anAngle.
        let separation = analysis.c0_value()?;
        self.convex |= separation > FRAC_PI_2 + angle;
        self.concave |= separation < FRAC_PI_2 - angle;
        Ok(())
    }
}
