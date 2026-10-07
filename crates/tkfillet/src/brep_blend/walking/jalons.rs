// Rust translation of the singular points (jalons) of BRepBlend_Walking::Recadre
// (BRepBlend_Walking.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The reframing from a singular point computed before the walking (a jalon) when the
//! step crosses it.
use super::recadre::Arc;
use super::{BRepBlendWalking, Sol, live, math_vector};
use crate::blend::{BlendFuncInv, BlendPoint};
use crate::brep_blend::{BRepBlendBlendTool, BRepBlendHCurve2dTool};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point2;

impl BRepBlendWalking {
    /// The index (0-based) of the singular point between the previous parameter and the
    /// current one (or at the current one): "only works when leaving simultaneously".
    pub(super) fn crossed_jalon(&self, last_param: f64) -> Option<usize> {
        let (t1, t2) = if last_param < self.param {
            (last_param, self.param)
        } else {
            (self.param, last_param)
        };
        for (ii, jalon) in self.jalons.iter().enumerate() {
            let t = jalon.parameter();
            if (t1 < t && t2 > t) || t == self.param {
                return Some(ii);
            }
            if t < t2 {
                continue;
            }
            return None; // On s'arrete si t>=t2
        }
        None
    }

    /// The start from the singular point `jalon` (periodic reframing of its other
    /// surface parameters, projection on the arc), and whether it is a solution.
    pub(super) fn jalon_start(
        &self,
        func_inv: &mut dyn BlendFuncInv,
        on_first: bool,
        jalon: usize,
        pmin: f64,
        thecur: &Arc,
        solrst: &mut Sol,
    ) -> Result<bool, Failure> {
        let mon_jalon: BlendPoint = self.jalons[jalon];
        solrst[1] = mon_jalon.parameter();
        let (u, v) = if on_first {
            mon_jalon.parameters_on_s2()?
        } else {
            mon_jalon.parameters_on_s1()?
        };
        (solrst[2], solrst[3]) = (u, v);
        self.reframe_periodic_jalon(on_first, pmin, thecur, solrst)?;
        // Pour le parametre sur arc il faut projeter...
        solrst[0] = self.jalon_arc_parameter(on_first, pmin, thecur, solrst)?;
        // On verifie le jalon
        func_inv.is_solution(&math_vector(solrst)?, self.tolpoint3d)
    }

    /// Recadrage eventuel pour le cas periodique: the parameters of the singular point
    /// on the other surface brought within 0.6 period of the arc point.
    fn reframe_periodic_jalon(
        &self,
        on_first: bool,
        pmin: f64,
        thecur: &Arc,
        solrst: &mut Sol,
    ) -> Result<(), Failure> {
        let surf = live(self.surface(!on_first))?;
        let (uper, vper) = (surf.is_u_periodic()?, surf.is_v_periodic()?);
        if !uper && !vper {
            return Ok(());
        }
        let lastpt2d = BRepBlendHCurve2dTool::value(thecur, pmin)?;
        if uper {
            solrst[2] = within_period(solrst[2], lastpt2d.x(), surf.u_period()?);
        }
        if surf.is_v_periodic()? {
            solrst[3] = within_period(solrst[3], lastpt2d.y(), surf.v_period()?);
        }
        Ok(())
    }

    /// The parameter on the arc of the singular point: the nearest end, or the
    /// projection when it is nearer, or else `pmin`.
    fn jalon_arc_parameter(
        &self,
        on_first: bool,
        pmin: f64,
        thecur: &Arc,
        solrst: &Sol,
    ) -> Result<f64, Failure> {
        let pt2d = Point2::new(solrst[2], solrst[3]);
        let (ufirst, ulast) = BRepBlendBlendTool::bounds(thecur)?;
        let mut dist = pt2d.distance(BRepBlendHCurve2dTool::value(thecur, ufirst)?);
        let mut parameter = ufirst;
        let distaux = pt2d.distance(BRepBlendHCurve2dTool::value(thecur, ulast)?);
        if distaux < dist {
            (parameter, dist) = (ulast, distaux);
        }
        if dist > precision::p_confusion() {
            let (prm, _) = BRepBlendBlendTool::project(pt2d, self.surface(on_first), thecur)?;
            let nearer = pt2d.distance(BRepBlendHCurve2dTool::value(thecur, prm)?) < dist;
            parameter = if nearer { prm } else { pmin };
        }
        Ok(parameter)
    }
}

/// The parameter `value` moved by one period towards `reference` when it is more than
/// 0.6 period away.
fn within_period(value: f64, reference: f64, period: f64) -> f64 {
    let mut value = value;
    if value - reference > period * 0.6 {
        value -= period;
    }
    if value - reference < -period * 0.6 {
        value += period;
    }
    value
}
