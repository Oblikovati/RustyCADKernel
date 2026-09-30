//! Bounded open conics get trimmed pcurves; infinite parts retain the whole conic.
use super::conic_support::{self, empty_descriptor, set_pcurve, trimmed, trimmed_pcurve};
use super::*;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkgeomalgo::{
    int_patch::{IntPatchIType, IntPatchLine},
    int_start::TheTopolTool,
};
use std::rc::Rc;
impl FaceFace {
    pub(super) fn open_conics(
        &mut self,
        line: &Handle<dyn IntPatchLine>,
        domains: &[Rc<dyn TheTopolTool>; 2],
        tolerance: &mut f64,
    ) -> Result<(), Failure> {
        let geometry = conic_support::geometry(line)?;
        for index in 1..=self.kernel.constructor.nb_parts()? {
            let range = self.kernel.constructor.part(index)?;
            let infinite = (
                precision::is_negative_infinite(range.0),
                precision::is_positive_infinite(range.1),
            );
            if !infinite.0 && !infinite.1 {
                self.finite_conic(
                    &geometry,
                    range,
                    line.arc_type() == IntPatchIType::Parabola,
                    tolerance,
                )?;
            } else {
                self.infinite_conic(&geometry, range, infinite, domains)?;
            }
        }
        Ok(())
    }
    fn finite_conic(
        &mut self,
        geometry: &Handle<dyn GeomCurve>,
        range: (f64, f64),
        parabola: bool,
        tolerance: &mut f64,
    ) -> Result<(), Failure> {
        let mut curve = empty_descriptor(&trimmed(geometry, range)?);
        if parabola {
            curve.set_tolerance(Tools::curve_tolerance(curve.curve(), self.tolerance())?);
        }
        for index in [0, 1] {
            if !self.approximation[index + 1] {
                continue;
            }
            let pcurve = self.project_pcurve(index, geometry, range, tolerance, false)?;
            if pcurve.is_null() {
                return Ok(());
            }
            set_pcurve(&mut curve, index, &trimmed_pcurve(&pcurve, range)?);
        }
        self.curves.push(curve);
        Ok(())
    }
    fn infinite_conic(
        &mut self,
        geometry: &Handle<dyn GeomCurve>,
        range: (f64, f64),
        infinite: (bool, bool),
        domains: &[Rc<dyn TheTopolTool>; 2],
    ) -> Result<(), Failure> {
        let parameter = match infinite {
            (true, false) => range.1 - 100.,
            (false, true) => range.0 + 100.,
            _ => Tools::intermediate_point(-100., 100.),
        };
        let point = geometry.eval_d0(parameter)?;
        let first = self.surfaces[0].get_type()?;
        let second = self.surfaces[1].get_type()?;
        if accepts_infinite(first)
            || accepts_infinite(second)
            || self.inside_domains(point, domains)?
        {
            self.curves.push(empty_descriptor(geometry));
        }
        Ok(())
    }
}
fn accepts_infinite(kind: SurfaceType) -> bool {
    matches!(
        kind,
        SurfaceType::SurfaceOfExtrusion
            | SurfaceType::OffsetSurface
            | SurfaceType::SurfaceOfRevolution
    )
}
