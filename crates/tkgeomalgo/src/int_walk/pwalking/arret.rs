// Rust translation of TestArret and RepartirOuDiviser of
// ModelingAlgorithms/TKGeomAlgo/IntWalk/IntWalk_PWalking.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TestArret` (the stop test on the natural domain and the closure of the line) and
//! `RepartirOuDiviser` (restart in the other direction, or divide the steps).
use super::IntWalkPWalking;
use crate::int_imp::{IntImpConstIsoparametric as Iso, choix_ref};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, real};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkmath::gp::Xy;

/// The framing of one parameter that left its range: the corrected value and the
/// squared cosine that ranks its isoparametric (-1 when it does not progress).
struct Framing {
    par_c: f64,
    duv: f64,
}

/// The ranking of the isoparametric of a parameter that left its range: `dpc` from
/// the previous point to the parameter, `dpb` to the bound, `dv` the progress of the
/// other parameter of the same surface.
fn iso_rank(dpc: f64, dpb: f64, dv: f64) -> f64 {
    let dv2: f64 = dv * dv;
    if dv2 <= real::real_epsilon() || dv2.is_nan() {
        // No progress in the other direction: the choice of iso does not change.
        return -1.0;
    }
    let mut duv: f64 = dpc * dpb + dv2;
    duv = duv * duv / (dpc * dpc + dv2) / (dpb * dpb + dv2);
    duv
}

impl IntWalkPWalking {
    /// `TestArret(DejaReparti, Param, ChoixIso)`: when a parameter (or the current
    /// solution) leaves the natural domain, frames `param` on the bounds, chooses the
    /// isoparametric of the best progress and returns true; otherwise, before the
    /// restart, returns whether the line closed (the first point between the previous
    /// and the current point on both surfaces).
    /// `let stop = walk.test_arret(false, &mut param, &mut iso)?;`.
    pub fn test_arret(
        &mut self,
        deja_reparti: bool,
        param: &mut Array1<'_, f64>,
        choix_iso: &mut Iso,
    ) -> Result<bool, Failure> {
        let mut p: [f64; 4] = [0.0; 4];
        for (i, value) in p.iter_mut().enumerate() {
            *value = *param.value(i as i32 + 1).map_err(errors::array_bounds)?;
        }
        let (a, b, c, d) = self.previous_point.parameters();
        let uvp: [f64; 4] = [a, b, c, d];
        let (a, b, c, d) = self.my_intersection_on2s.point()?.parameters();
        let sol: [f64; 4] = [a, b, c, d];
        let framings: [Option<Framing>; 4] =
            [0, 1, 2, 3].map(|i| self.framing(i, &p, &uvp, sol[i]));
        if framings.iter().all(Option::is_none) {
            return self.test_closure(deja_reparti);
        }
        let par_c: [f64; 4] = [0, 1, 2, 3].map(|i| framings[i].as_ref().map_or(p[i], |f| f.par_c));
        let duv: [f64; 4] = [0, 1, 2, 3].map(|i| framings[i].as_ref().map_or(-1., |f| f.duv));
        let (mut ddv, mut k) = (-1.0, -1);
        for i in 0..4 {
            *param
                .change_value(i as i32 + 1)
                .map_err(errors::array_bounds)? = par_c[i];
            if duv[i] > ddv {
                (ddv, k) = (duv[i], i as i32);
            }
        }
        if k != -1 {
            *choix_iso = choix_ref(k)?;
        } else if let Some(iso) = self.border_iso(&par_c) {
            *choix_iso = iso;
        }
        self.close = false;
        Ok(true)
    }
    /// The framing of parameter `i` (0-based) when it or the solution `sol` is beyond
    /// a bound by more than the resolution.
    fn framing(&self, i: usize, p: &[f64; 4], uvp: &[f64; 4], sol: f64) -> Option<Framing> {
        let other: usize = [1, 0, 3, 2][i];
        let (low, high, eps) = (self.lower[i], self.upper[i], self.reso[i]);
        let dv: f64 = p[other] - uvp[other];
        if p[i] < (low - eps) || sol < (low - eps) {
            return Some(Framing {
                par_c: low,
                duv: iso_rank(uvp[i] - p[i], uvp[i] - low, dv),
            });
        }
        if p[i] > (high + eps) || sol > (high + eps) {
            return Some(Framing {
                par_c: high,
                duv: iso_rank(p[i] - uvp[i], high - uvp[i], dv),
            });
        }
        None
    }
    /// Without progress anywhere, the isoparametric of the first parameter on its
    /// border (within the resolution).
    fn border_iso(&self, par_c: &[f64; 4]) -> Option<Iso> {
        const ISOS: [Iso; 4] = [
            Iso::UIsoparametricOnCaro1,
            Iso::VIsoparametricOnCaro1,
            Iso::UIsoparametricOnCaro2,
            Iso::VIsoparametricOnCaro2,
        ];
        (0..4)
            .find(|&i| {
                par_c[i] <= self.lower[i] + self.reso[i] || par_c[i] >= self.upper[i] - self.reso[i]
            })
            .map(|i| ISOS[i])
    }
    /// The closure test of `TestArret` before the restart: the first point lies
    /// between the previous and the current point on both surfaces.
    fn test_closure(&mut self, deja_reparti: bool) -> Result<bool, Failure> {
        if deja_reparti {
            return Ok(false);
        }
        let first = self.line_ref()?.value(1)?;
        let current = *self.my_intersection_on2s.point()?;
        let between = |on_first: bool| -> bool {
            let p1: Xy = first.value_on_surface(on_first).xy();
            let prev = self.previous_point.value_on_surface(on_first).xy();
            let inter = current.value_on_surface(on_first).xy();
            (p1 - prev).dot(p1 - inter) < 0.0
        };
        self.close = between(true) && between(false);
        Ok(self.close)
    }
    /// `RepartirOuDiviser(DejaReparti, ChoixIso, Arrive)`: an arrived line restarts
    /// from its first point in the other direction when it did not yet; a failed step
    /// divides the steps, or, when they reach the resolutions, restarts (or arrives).
    /// `walk.repartir_ou_diviser(&mut deja_reparti, &mut iso, &mut arrive)?`.
    pub fn repartir_ou_diviser(
        &mut self,
        deja_reparti: &mut bool,
        choix_iso: &mut Iso,
        arrive: &mut bool,
    ) -> Result<(), Failure> {
        if *arrive {
            if !*deja_reparti {
                *arrive = false;
                *deja_reparti = true;
                self.restart_other_direction(choix_iso)?;
            }
            return Ok(());
        }
        if !(0..4).all(|i| self.pasuv[i] * 0.5 < self.reso[i]) {
            self.pasuv = self.pasuv.map(|pas| pas * 0.5);
            return Ok(());
        }
        if !self.previoustg {
            self.tglast = true; // IS IT ENOUGH ????
        }
        if *deja_reparti {
            *arrive = true;
            return Ok(());
        }
        *deja_reparti = true;
        self.restart_other_direction(choix_iso)
    }
    /// The restart from the first point in the other direction: the line is reversed,
    /// the first tangents come back reversed, and the steps become the last step.
    fn restart_other_direction(&mut self, choix_iso: &mut Iso) -> Result<(), Failure> {
        let line = self.line_handle()?;
        self.previous_point = line.value(1)?;
        self.previoustg = false;
        (self.previousd1, self.previousd2, self.previousd) =
            (self.firstd1, self.firstd2, self.tgdir);
        self.my_tangent_idx = line.nb_points();
        self.tgdir.reverse();
        line.reverse();
        self.sens_cheminement = -1;
        (self.tgfirst, self.tglast) = (self.tglast, false);
        *choix_iso = self.choix_iso_sav;
        let nn: i32 = line.nb_points();
        if nn > 2 {
            let (u1, v1, u2, v2) = line.value(nn)?.parameters();
            let (pu1, pv1, pu2, pv2) = line.value(nn - 1)?.parameters();
            self.pasuv = [
                (u1 - pu1).abs(),
                (v1 - pv1).abs(),
                (u2 - pu2).abs(),
                (v2 - pv2).abs(),
            ];
        }
        Ok(())
    }
}
