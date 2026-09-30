// Rust translation of TestDeflection of ModelingAlgorithms/TKGeomAlgo/IntWalk/IntWalk_PWalking.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TestDeflection`: the status of a new point after the previous one (tangent,
//! inflexion risk, confused, angles in the parameters, deflection of the chord and of
//! the osculating circle) and the next steps.
use super::IntWalkPWalking;
use crate::int_imp::IntImpConstIsoparametric as Iso;
use crate::int_surf::IntSurfPntOn2S;
use crate::int_walk::IntWalkStatusDeflection as Status;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg3d::adaptor3d::h_surface_tool as tool;
use rustycad_tkmath::gp::{self, Direction3, Point3};
use std::f64::consts::PI;

/// `d`: the bound of the exponent of the tolerance coefficients.
const D_MAX: f64 = 7.0;

/// The parameter steps from the previous to the current point (`Du1, Dv1, Du2, Dv2`).
struct Steps {
    d: [f64; 4],
    abs: [f64; 4],
}

impl IntWalkPWalking {
    /// `TestDeflection(ChoixIso, theStatus)`: the status of the current point of the
    /// intersection after the previous point, adjusting the steps (OCC431, OCC26717).
    /// `let status = walk.test_deflection(iso, IntWalkStatusDeflection::OK)?;`.
    pub fn test_deflection(
        &mut self,
        choix_iso: Iso,
        the_status: Status,
    ) -> Result<Status, Failure> {
        if self.line_ref()?.nb_points() == 1 {
            (
                self.static_blocage_sur_pas_trop_grand,
                self.static_precedent_inflexion,
            ) = (0, 0);
        }
        let current: IntSurfPntOn2S = *self.my_intersection_on2s.point()?;
        if self.my_intersection_on2s.is_tangent()? {
            return Ok(Status::ArretSurPoint);
        }
        let tg_courante: Direction3 = self.my_intersection_on2s.direction()?;
        let cos_between_tangent: f64 = tg_courante.dot(self.previousd);
        if let Some(early) = self.inflexion_test(cos_between_tangent) {
            return Ok(early);
        }
        let sq_dist: f64 = self
            .previous_point
            .value()
            .square_distance(&current.value());
        let mut status: Status = Status::OK;
        if sq_dist < precision::square_confusion() {
            self.confused_steps(choix_iso)?;
            status = Status::PointConfondu;
        }
        let steps: Steps = self.parameter_steps(&current);
        if (0..4).all(|k| steps.abs[k] < self.reso[k]) {
            self.pasuv = self.reso;
            return Ok(Status::ArretSurPointPrecedent);
        }
        let tol_coeff: [f64; 2] = self.tolerance_coefficients(&steps);
        if status != Status::PointConfondu
            && let Some(early) = self.side_test(&steps, tol_coeff)?
        {
            return Ok(early);
        }
        let fleche_courante: f64 = ((self.previousd.xyz() - tg_courante.xyz()).square_modulus()
            * sq_dist)
            .abs()
            .sqrt()
            / 8.;
        let ratio: f64 = match self.fleche_test(fleche_courante, &steps, status) {
            Ok(ratio) => ratio,
            Err(early) => return Ok(early),
        };
        if status != Status::PointConfondu
            && let Some(early) = self.arc_test(sq_dist, cos_between_tangent, the_status, &steps)
        {
            return Ok(early);
        }
        for k in 0..4 {
            self.pasuv[k] = std_max(
                self.my_step_min[k],
                std_min(
                    std_min(ratio * steps.abs[k], self.pasuv[k]),
                    self.pas_init[k],
                ),
            );
        }
        if status == Status::OK {
            self.static_blocage_sur_pas_trop_grand = 0;
        }
        Ok(status)
    }
    /// Halves the steps; `near` when all are then under the resolutions.
    fn halve_steps(&mut self, near: Status, far: Status) -> Status {
        self.pasuv = self.pasuv.map(|pas| pas * 0.5);
        if (0..4).all(|k| self.pasuv[k] < self.reso[k]) {
            near
        } else {
            far
        }
    }
    /// The risk of an inflexion point (tangents against each other): halve the steps
    /// and let the next three tests pass; one pending pass is taken here.
    fn inflexion_test(&mut self, cos_between_tangent: f64) -> Option<Status> {
        if cos_between_tangent < 0.0 {
            self.pasuv = self.pasuv.map(|pas| pas * 0.5);
            self.static_precedent_inflexion += 3;
            let under: bool = (0..4).all(|k| self.pasuv[k] < self.reso[k]);
            return Some(if under {
                Status::ArretSurPointPrecedent
            } else {
                Status::PasTropGrand
            });
        }
        if self.static_precedent_inflexion > 0 {
            self.static_precedent_inflexion -= 1;
            return Some(Status::OK);
        }
        None
    }
    /// Confused points: the initial steps grow to five resolutions, the steps by half
    /// (up to them), and the step of the chosen isoparametric at its initial value
    /// goes to twice the local resolution of `tolconf` (OCC26717).
    fn confused_steps(&mut self, choix_iso: Iso) -> Result<(), Failure> {
        for k in 0..4 {
            self.pas_init[k] = std_max(self.pas_init[k], 5.0 * self.reso[k]);
        }
        for k in 0..4 {
            self.pasuv[k] = std_max(
                self.pasuv[k],
                std_min(1.5 * self.pasuv[k], self.pas_init[k]),
            );
        }
        let c: usize = choix_iso as usize;
        if (self.pasuv[c] - self.pas_init[c]).abs() > precision::confusion() {
            return Ok(());
        }
        let ref_dist: f64 = self.offset_distance(choix_iso)?;
        let mut local_resol: f64 = 0.;
        if ref_dist > gp::resolution() {
            local_resol = self.pasuv[c] * self.tolconf / ref_dist;
        }
        if self.pasuv[c] < 2. * local_resol {
            self.pasuv[c] = 2. * local_resol;
            self.pas_init[c] = self.pasuv[c];
        }
        Ok(())
    }
    /// The distance from the previous point to the point one step away along the
    /// chosen isoparametric, on its surface.
    fn offset_distance(&mut self, choix_iso: Iso) -> Result<f64, Failure> {
        let (caro1, caro2) = self.surfaces();
        let on_first: bool = matches!(
            choix_iso,
            Iso::UIsoparametricOnCaro1 | Iso::VIsoparametricOnCaro1
        );
        let (cur_u, cur_v) = self.previous_point.parameters_on_surface(on_first);
        let caro = if on_first { &caro1 } else { &caro2 };
        let cur_pnt: Point3 = tool::value(caro, cur_u, cur_v)?;
        let sens: f64 = f64::from(self.sens_cheminement);
        let offset_pnt: Point3 = match choix_iso {
            Iso::UIsoparametricOnCaro1 => tool::value(&caro1, cur_u + sens * self.pasuv[0], cur_v)?,
            Iso::VIsoparametricOnCaro1 => tool::value(&caro1, cur_u, cur_v + sens * self.pasuv[1])?,
            Iso::UIsoparametricOnCaro2 => tool::value(&caro2, cur_u + sens * self.pasuv[2], cur_v)?,
            Iso::VIsoparametricOnCaro2 => tool::value(&caro2, cur_u, cur_v + sens * self.pasuv[3])?,
        };
        Ok(cur_pnt.distance(&offset_pnt))
    }
    /// The parameter steps between the previous and the current point.
    fn parameter_steps(&self, current: &IntSurfPntOn2S) -> Steps {
        let (up1, vp1, up2, vp2) = self.previous_point.parameters();
        let (uc1, vc1, uc2, vc2) = current.parameters();
        let d: [f64; 4] = [uc1 - up1, vc1 - vp1, uc2 - up2, vc2 - vp2];
        Steps {
            d,
            abs: d.map(f64::abs),
        }
    }
    /// `exp(d1)`, `exp(d2)`: the coefficients that widen the angle tolerances when a
    /// step is near the resolutions (NIZNHY-PKV 2002).
    fn tolerance_coefficients(&self, steps: &Steps) -> [f64; 2] {
        let mut tol_area: f64 = 100.0;
        if self
            .reso
            .iter()
            .any(|&reso| reso < precision::p_confusion())
        {
            tol_area *= 2.0;
        }
        let min_div2: f64 = precision::confusion() * precision::confusion();
        [0, 2].map(|k| {
            let duv: f64 = steps.d[k] * steps.d[k] + steps.d[k + 1] * steps.d[k + 1];
            let reso_uv: f64 = self.reso[k] * self.reso[k] + self.reso[k + 1] * self.reso[k + 1];
            let mut d: f64 = D_MAX;
            if duv > min_div2 {
                d = (reso_uv / duv).abs();
                d = std_min(d.sqrt() * tol_area, D_MAX);
            }
            d.exp()
        })
    }
    /// The angles in the parameters: the step against the previous tangents, then
    /// against the new tangents (and the angle between the tangents).
    fn side_test(&mut self, steps: &Steps, tol_coeff: [f64; 2]) -> Result<Option<Status>, Failure> {
        let cos_ref_2d: f64 = (PI / 9.0).cos();
        let cos_ref: [f64; 2] = [cos_ref_2d / tol_coeff[0], cos_ref_2d / tol_coeff[1]];
        let duv: [f64; 2] =
            [0, 2].map(|k| steps.d[k] * steps.d[k] + steps.d[k + 1] * steps.d[k + 1]);
        let cosi1: f64 = steps.d[0] * self.previousd1.x() + steps.d[1] * self.previousd1.y();
        let cosi2: f64 = steps.d[2] * self.previousd2.x() + steps.d[3] * self.previousd2.y();
        if cosi1 * cosi1 < cos_ref[0] * duv[0] || cosi2 * cosi2 < cos_ref[1] * duv[1] {
            if self.halve_steps(Status::ArretSurPointPrecedent, Status::PasTropGrand)
                == Status::ArretSurPointPrecedent
            {
                return Ok(Some(Status::ArretSurPointPrecedent));
            }
            self.pasuv = self.pasuv.map(|pas| pas * 0.5);
            return Ok(Some(Status::PasTropGrand));
        }
        let tg1 = self.my_intersection_on2s.direction_on_s1()?;
        let tg2 = self.my_intersection_on2s.direction_on_s2()?;
        let cosi1: f64 = steps.d[0] * tg1.x() + steps.d[1] * tg1.y();
        let cosi2: f64 = steps.d[2] * tg2.x() + steps.d[3] * tg2.y();
        let ang1: f64 = self.previousd1.angle(tg1).abs();
        let ang2: f64 = self.previousd2.angle(tg2).abs();
        let ang_ref: [f64; 2] = [PI / 2.0 * tol_coeff[0], PI / 2.0 * tol_coeff[1]];
        if cosi1 * cosi1 < cos_ref[0] * duv[0]
            || cosi2 * cosi2 < cos_ref[1] * duv[1]
            || ang1 > ang_ref[0]
            || ang2 > ang_ref[1]
        {
            return Ok(Some(
                self.halve_steps(Status::ArretSurPoint, Status::PasTropGrand),
            ));
        }
        Ok(None)
    }
    /// The deflection of the chord estimated with the two tangents: a step too small
    /// grows (by at most the initial steps, and the growth is blocked after five
    /// changes), a step too great shrinks; returns the ratio of the final step, or the
    /// status when the test ends here.
    fn fleche_test(
        &mut self,
        fleche_courante: f64,
        steps: &Steps,
        status: Status,
    ) -> Result<f64, Status> {
        if fleche_courante <= self.fleche * 0.5 {
            return Err(self.grow_small_step(fleche_courante, steps, status));
        }
        if fleche_courante > self.fleche {
            let ratio: f64 = self.fleche / fleche_courante;
            self.pasuv = self.pasuv.map(|pas| ratio * pas);
            return Err(Status::PasTropGrand);
        }
        Ok(0.75 * (self.fleche / fleche_courante))
    }
    /// The current step is too small: the steps grow to the last steps (at least the
    /// resolutions) times the deflection ratio, bounded by the initial steps.
    fn grow_small_step(&mut self, fleche_courante: f64, steps: &Steps, status: Status) -> Status {
        let mut ratio: f64 = if fleche_courante > 1e-16 {
            0.5 * (self.fleche / fleche_courante)
        } else {
            10.0
        };
        let saved: [f64; 4] = self.pasuv;
        for k in 0..4 {
            if self.pasuv[k] < steps.abs[k] {
                self.pasuv[k] = steps.abs[k];
            }
        }
        for k in 0..4 {
            if self.pasuv[k] < self.reso[k] {
                self.pasuv[k] = self.reso[k];
            }
        }
        let mut r: f64 = self.pas_init[0] / self.pasuv[0];
        for k in 1..4 {
            let r1: f64 = self.pas_init[k] / self.pasuv[k];
            if r1 < r {
                r = r1;
            }
        }
        if ratio > r {
            ratio = r;
        }
        for k in 0..4 {
            self.pasuv[k] = std_min(ratio * self.pasuv[k], self.pas_init[k]);
        }
        if self.pasuv != saved {
            self.static_blocage_sur_pas_trop_grand += 1;
            if self.static_blocage_sur_pas_trop_grand > 5 {
                self.static_blocage_sur_pas_trop_grand = 0;
                return Status::PasTropGrand;
            }
        }
        if status == Status::OK {
            self.static_blocage_sur_pas_trop_grand = 0;
        }
        status
    }
    /// The deflection of the osculating circle through both points with both
    /// tangents against `tolconf`: greater, the step is too great; under half of it
    /// (unless the previous step was too great), too small.
    fn arc_test(
        &mut self,
        sq_dist: f64,
        cos_between_tangent: f64,
        the_status: Status,
        steps: &Steps,
    ) -> Option<Status> {
        let inv_sq_max: f64 = 0.25 * sq_dist / (self.tolconf * self.tolconf);
        let sin_b2_max: f64 = 1.0 - 2.0 / (1.0 + inv_sq_max);
        let mut status: Status = Status::OK;
        if sin_b2_max >= 0.0 && (cos_between_tangent <= 2.0 * sin_b2_max * sin_b2_max - 1.0) {
            status = Status::PasTropGrand;
        } else {
            let inv_sq_min: f64 = 4.0 * inv_sq_max;
            let sin_b2_min: f64 = 1.0 - 2.0 / (1.0 + inv_sq_min);
            if the_status != Status::PasTropGrand
                && ((sin_b2_min < 0.0)
                    || (cos_between_tangent >= 2.0 * sin_b2_min * sin_b2_min - 1.0))
            {
                status = Status::StepTooSmall;
            }
        }
        match status {
            Status::PasTropGrand => {
                self.pasuv = self.pasuv.map(|pas| pas * 0.5);
                Some(status)
            }
            Status::StepTooSmall => {
                for k in 0..4 {
                    self.pasuv[k] = std_max(self.pasuv[k], steps.abs[k]);
                }
                for k in 0..4 {
                    self.pas_init[k] = std_max(self.pas_init[k], steps.abs[k]);
                }
                Some(status)
            }
            _ => None,
        }
    }
}
