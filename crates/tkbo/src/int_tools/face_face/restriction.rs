//! Restriction curves are trimmed against both rectangular surface domains.
use super::conic_support::{empty_descriptor, set_pcurve, trimmed, trimmed_pcurve};
use super::*;
use rustycad_tkgeomalgo::{
    geom_int::{GeomIntIntSS, GeomIntRLineCurves},
    int_patch::IntPatchRLine,
};
use rustycad_tkmath::{bnd::BoundingBox2, gp::Point2};
impl FaceFace {
    pub(super) fn restriction_curves(
        &mut self,
        line: &Handle<IntPatchRLine>,
        tolerance: f64,
    ) -> Result<(), Failure> {
        let mut curves = GeomIntRLineCurves::default();
        GeomIntIntSS::treat_rline(line, &self.surfaces[0], &self.surfaces[1], &mut curves)?;
        if curves.c3d.is_null() {
            return Ok(());
        }
        self.trim_restriction(&curves, tolerance)
    }
    fn trim_restriction(
        &mut self,
        curves: &GeomIntRLineCurves,
        tolerance: f64,
    ) -> Result<(), Failure> {
        let mut bounds = [
            restriction_box(&self.surfaces[0])?,
            restriction_box(&self.surfaces[1])?,
        ];
        let mut parameters = vec![curves.c3d.first_parameter(), curves.c3d.last_parameter()];
        GeomIntIntSS::trim_iline_on_surf_boundaries(
            &curves.c2d1,
            &curves.c2d2,
            &bounds[0],
            &bounds[1],
            &mut parameters,
        )?;
        bounds[0].enlarge(tolerance);
        bounds[1].enlarge(tolerance);
        self.append_restriction_parts(curves, &bounds, &mut parameters)
    }
    fn append_restriction_parts(
        &mut self,
        curves: &GeomIntRLineCurves,
        bounds: &[BoundingBox2; 2],
        parameters: &mut [f64],
    ) -> Result<(), Failure> {
        for index in 0..parameters.len() - 1 {
            let range = (parameters[index], parameters[index + 1]);
            if range.1 - range.0 <= precision::p_confusion() {
                if index < parameters.len() - 2 {
                    parameters[index + 1] = range.0;
                }
                continue;
            }
            if let Some(curve) = self.restriction_part(curves, bounds, range)? {
                self.curves.push(curve);
            }
        }
        Ok(())
    }
    fn restriction_part(
        &self,
        curves: &GeomIntRLineCurves,
        bounds: &[BoundingBox2; 2],
        range: (f64, f64),
    ) -> Result<Option<Curve>, Failure> {
        let mut descriptor = empty_descriptor(&Handle::default());
        for (index, pcurve) in [&curves.c2d1, &curves.c2d2].into_iter().enumerate() {
            if pcurve.is_null() {
                continue;
            }
            if bounds[index].is_out_point(pcurve.eval_d0(0.5 * (range.0 + range.1))?) {
                return Ok(None);
            }
            if self.approximation[index + 1] {
                set_pcurve(&mut descriptor, index, &trimmed_pcurve(pcurve, range)?);
            }
        }
        descriptor.set_curve(&trimmed(&curves.c3d, range)?);
        Ok(Some(descriptor))
    }
}
fn restriction_box(surface: &GeomAdaptorSurface) -> Result<BoundingBox2, Failure> {
    let mut bounds = BoundingBox2::default();
    bounds.add_point(Point2::new(
        surface.first_u_parameter()?,
        surface.first_v_parameter()?,
    ));
    bounds.add_point(Point2::new(
        surface.last_u_parameter()?,
        surface.last_v_parameter()?,
    ));
    Ok(bounds)
}
