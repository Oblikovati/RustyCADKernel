//! Native MakeCurve dispatch; a failed pcurve validation restarts the same line.
use super::*;
use rustycad_tkgeomalgo::{
    int_patch::{IntPatchIType, IntPatchLine, IntPatchPointLine, IntPatchWLine},
    int_start::TheTopolTool,
};
use std::rc::Rc;
#[derive(Default)]
pub(super) struct ApproximationRetry {
    pub(super) reject_surface: bool,
    pub(super) reapproximate: bool,
    pub(super) pcurves_ok: bool,
}
impl FaceFace {
    pub(super) fn make_curve(
        &mut self,
        index: i32,
        domains: &[Rc<dyn TheTopolTool>; 2],
        tolerance: f64,
    ) -> Result<(), Failure> {
        let mut retry = ApproximationRetry {
            pcurves_ok: true,
            ..Default::default()
        };
        loop {
            let line = self.kernel.intersection.line(index)?;
            if !self.construct_parts(&line)? {
                return Ok(());
            }
            if !self.dispatch_curve(&line, domains, tolerance, &mut retry)? {
                return Ok(());
            }
        }
    }
    fn dispatch_curve(
        &mut self,
        line: &Handle<dyn IntPatchLine>,
        domains: &[Rc<dyn TheTopolTool>; 2],
        tolerance: f64,
        retry: &mut ApproximationRetry,
    ) -> Result<bool, Failure> {
        let mut tolpc = self.approximation_tolerance;
        Ok(match line.arc_type() {
            IntPatchIType::Lin | IntPatchIType::Parabola | IntPatchIType::Hyperbola => {
                self.open_conics(line, domains, &mut tolpc)?;
                false
            }
            IntPatchIType::Circle | IntPatchIType::Ellipse => {
                self.closed_conics(line, domains, &mut tolpc)?;
                false
            }
            IntPatchIType::Walking => self.walking_curves(&line.cast(), retry)?,
            IntPatchIType::Restriction => {
                self.restriction_curves(&line.cast(), tolerance)?;
                false
            }
            _ => false,
        })
    }
    fn construct_parts(&mut self, line: &Handle<dyn IntPatchLine>) -> Result<bool, Failure> {
        if line.arc_type() == IntPatchIType::Walking {
            let walking = line.cast::<IntPatchWLine>();
            if walking.is_null() {
                return Ok(false);
            }
            // Native still evaluates endpoints even though both branches retain false.
            let first = walking.point(1)?.value();
            let last = walking.point(walking.nb_pnts()?)?.value();
            let _closed = first.square_distance(&last) < 1e-14;
        }
        if line.arc_type() == IntPatchIType::Restriction {
            return Ok(true);
        }
        self.kernel.constructor.perform(line)?;
        Ok(self.kernel.constructor.is_done() && self.kernel.constructor.nb_parts()? > 0)
    }
}
