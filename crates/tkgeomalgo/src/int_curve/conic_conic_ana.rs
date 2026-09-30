// Rust translation of IntCurve_IntConicConic.cxx (the Perform overloads of the pairs
// with a parabola or a hyperbola) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The IntCurve_IntConicConic overloads whose parametric conic is a parabola or a
//! hyperbola: its range is bounded by the analytic intersections of two offset
//! copies of one conic with the other before IntImpConicParConic runs.
use super::conic_conic_bounds::{
    AnaBounds, CLIP_EARLY, ClipMode, ConicPair, PARAM_MAX_ON_HYPERBOLA, PARAM_MAX_ON_PARABOLA,
    TOL_EXACT_INTER, bounded_domain,
};
use super::{IConicTool, IntConicConic, PConic};
use crate::int_res2d::Domain;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana2d::{AnaIntersection, Conic};
use rustycad_tkmath::gp::{Circle2, Ellipse2, Hyperbola2, Line2, Parabola2, Vector2};
use std::cmp::Ordering;

/// `maxtol` of most overloads: `100 max(Tol, TolConf)`, at least 1e-6.
fn scaled_max_tolerance(tol_conf: f64, tol: f64) -> f64 {
    let mut maxtol: f64 = if tol > tol_conf { tol } else { tol_conf };
    maxtol *= 100.0;
    if maxtol < 0.000001 {
        maxtol = 0.000001;
    }
    maxtol
}

/// The analytic range of a hyperbola `h` against `other`, from its two copies
/// translated by `maxtol` along its X axis.
fn hyperbola_range(h: Hyperbola2, other: &Conic, maxtol: f64) -> Result<AnaBounds, Failure> {
    let direction = h.x_axis().direction();
    let mut offset = Vector2::new(maxtol * direction.x(), maxtol * direction.y());
    let mut bounds: AnaBounds = AnaBounds::new();
    let mut ana: AnaIntersection = AnaIntersection::new();
    ana.perform_hyperbola_conic(h.translated(offset), other)?;
    bounds.widen(&ana, &h, maxtol, PARAM_MAX_ON_HYPERBOLA)?;
    offset.reverse();
    ana.perform_hyperbola_conic(h.translated(offset), other)?;
    bounds.widen(&ana, &h, maxtol, PARAM_MAX_ON_HYPERBOLA)?;
    Ok(bounds)
}

/// A circle or ellipse domain closed on `[First, First + 2PI]` unless it already is.
pub(super) fn closed_domain(domain: &Domain) -> Result<Domain, Failure> {
    let mut closed: Domain = *domain;
    if !domain.is_closed() {
        let first: f64 = domain.first_parameter()?;
        closed.set_equivalent_parameters(
            first,
            first + std::f64::consts::PI + std::f64::consts::PI,
        )?;
    }
    Ok(closed)
}

impl IntConicConic {
    /// `Perform(gp_Lin2d, DL, gp_Hypr2d, DH, TolConf, Tol)`.
    /// `inter.perform_line_hyperbola(l, &dl, h, &dh, 1e-9, 1e-9)?`.
    pub fn perform_line_hyperbola(
        &mut self,
        l: Line2,
        dl: &Domain,
        h: Hyperbola2,
        dh: &Domain,
        tol_conf: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        self.base.reset_fields();
        let itool: IConicTool = IConicTool::from_line(l)?;
        let pcurve: PConic = accurate(PConic::from_hyperbola(h));
        self.inter
            .set_reversed_parameters(self.base.reversed_parameters());
        let maxtol: f64 = scaled_max_tolerance(tol_conf, tol);
        let bounds: AnaBounds = hyperbola_range(h, &Conic::from_line(l), maxtol)?;
        let pair = ConicPair {
            itool: &itool,
            imp_domain: dl,
            pcurve: &pcurve,
            par_domain: dh,
        };
        self.perform_on_bounds(&pair, bounds, (tol_conf, tol), CLIP_EARLY)
    }

    /// `Perform(gp_Circ2d, DC, gp_Parab2d, DP, TolConf, Tol)`: the range from two
    /// circles of radius `R +/- R/10` (the smaller one only when positive).
    /// `inter.perform_circle_parabola(c, &dc, p, &dp, 1e-9, 1e-9)?`.
    pub fn perform_circle_parabola(
        &mut self,
        c: Circle2,
        dc: &Domain,
        p: Parabola2,
        dp: &Domain,
        tol_conf: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        self.base.reset_fields();
        let itool: IConicTool = IConicTool::from_circle(c);
        let pcurve: PConic = accurate(PConic::from_parabola(p));
        self.inter
            .set_reversed_parameters(self.base.reversed_parameters());
        let d: Domain = closed_domain(dc)?;
        let maxtol: f64 = c.radius() / 10.0;
        let mut cp: Circle2 = c;
        cp.set_radius(c.radius() + maxtol)?;
        let mut bounds: AnaBounds = AnaBounds::new();
        let mut ana: AnaIntersection = AnaIntersection::new();
        ana.perform_parabola_conic(p, &Conic::from_circle(cp))?;
        bounds.widen(&ana, &p, maxtol, PARAM_MAX_ON_PARABOLA)?;
        if c.radius() > maxtol {
            cp.set_radius(c.radius() - maxtol)?;
            ana.perform_parabola_conic(p, &Conic::from_circle(cp))?;
            bounds.widen(&ana, &p, maxtol, PARAM_MAX_ON_PARABOLA)?;
        }
        let pair = ConicPair {
            itool: &itool,
            imp_domain: &d,
            pcurve: &pcurve,
            par_domain: dp,
        };
        self.perform_on_bounds(&pair, bounds, (tol_conf, tol), CLIP_EARLY)
    }

    /// `Perform(gp_Circ2d, DC, gp_Hypr2d, DH, TolConf, Tol)`: offsets of `R/10`.
    /// `inter.perform_circle_hyperbola(c, &dc, h, &dh, 1e-9, 1e-9)?`.
    pub fn perform_circle_hyperbola(
        &mut self,
        c: Circle2,
        dc: &Domain,
        h: Hyperbola2,
        dh: &Domain,
        tol_conf: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        self.base.reset_fields();
        let itool: IConicTool = IConicTool::from_circle(c);
        let pcurve: PConic = accurate(PConic::from_hyperbola(h));
        self.inter
            .set_reversed_parameters(self.base.reversed_parameters());
        let d: Domain = closed_domain(dc)?;
        let bounds: AnaBounds = hyperbola_range(h, &Conic::from_circle(c), c.radius() / 10.0)?;
        let pair = ConicPair {
            itool: &itool,
            imp_domain: &d,
            pcurve: &pcurve,
            par_domain: dh,
        };
        self.perform_on_bounds(&pair, bounds, (tol_conf, tol), CLIP_EARLY)
    }

    /// `Perform(gp_Parab2d, DP1, gp_Parab2d, DP2, TolConf, Tol)`: the second parabola
    /// offset along its mirror axis. `inter.perform_parabolas(p1, &d1, p2, &d2, 1e-9, 1e-9)?`.
    pub fn perform_parabolas(
        &mut self,
        p1: Parabola2,
        dp1: &Domain,
        p2: Parabola2,
        dp2: &Domain,
        tol_conf: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        self.base.reset_fields();
        let itool: IConicTool = IConicTool::from_parabola(p1);
        let pcurve: PConic = accurate(PConic::from_parabola(p2));
        self.inter
            .set_reversed_parameters(self.base.reversed_parameters());
        let maxtol: f64 = scaled_max_tolerance(tol_conf, tol);
        let direction = p2.axis_of_symmetry().direction();
        let mut offset = Vector2::new(maxtol * direction.x(), maxtol * direction.y());
        let other: Conic = Conic::from_parabola(p1);
        let mut bounds: AnaBounds = AnaBounds::new();
        let mut ana: AnaIntersection = AnaIntersection::new();
        ana.perform_parabola_conic(p2.translated(offset), &other)?;
        bounds.widen(&ana, &p2, maxtol, PARAM_MAX_ON_PARABOLA)?;
        offset.reverse();
        ana.perform_parabola_conic(p2.translated(offset), &other)?;
        bounds.widen(&ana, &p2, maxtol, PARAM_MAX_ON_PARABOLA)?;
        let pair = ConicPair {
            itool: &itool,
            imp_domain: dp1,
            pcurve: &pcurve,
            par_domain: dp2,
        };
        self.perform_on_bounds(&pair, bounds, (tol_conf, tol), CLIP_EARLY)
    }

    /// `Perform(gp_Elips2d, DE, gp_Parab2d, DP, TolConf, Tol)`: the range from two
    /// ellipses with both radii moved by `MinorRadius/10` (the smaller one only when
    /// the minor radius is larger). `inter.perform_ellipse_parabola(e, &de, p, &dp, 1e-9, 1e-9)?`.
    pub fn perform_ellipse_parabola(
        &mut self,
        e: Ellipse2,
        de: &Domain,
        p: Parabola2,
        dp: &Domain,
        tol_conf: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        self.base.reset_fields();
        let itool: IConicTool = IConicTool::from_ellipse(e);
        let pcurve: PConic = accurate(PConic::from_parabola(p));
        self.inter
            .set_reversed_parameters(self.base.reversed_parameters());
        let d: Domain = closed_domain(de)?;
        // The native code computes max(Tol, TolConf) first and overwrites it.
        let maxtol: f64 = e.minor_radius() / 10.0;
        let mut ep: Ellipse2 = e;
        ep.set_major_radius(e.major_radius() + maxtol)?;
        ep.set_minor_radius(e.minor_radius() + maxtol)?;
        let mut bounds: AnaBounds = AnaBounds::new();
        let mut ana: AnaIntersection = AnaIntersection::new();
        ana.perform_parabola_conic(p, &Conic::from_ellipse(ep))?;
        bounds.widen(&ana, &p, maxtol, PARAM_MAX_ON_PARABOLA)?;
        if e.minor_radius() > maxtol {
            ep.set_minor_radius(e.minor_radius() - maxtol)?;
            ep.set_major_radius(e.major_radius() - maxtol)?;
            ana.perform_parabola_conic(p, &Conic::from_ellipse(ep))?;
            bounds.widen(&ana, &p, maxtol, PARAM_MAX_ON_PARABOLA)?;
        }
        let pair = ConicPair {
            itool: &itool,
            imp_domain: &d,
            pcurve: &pcurve,
            par_domain: dp,
        };
        self.perform_on_bounds(&pair, bounds, (tol_conf, tol), CLIP_EARLY)
    }

    /// `Perform(gp_Parab2d, DP, gp_Hypr2d, DH, TolConf, Tol)`: the hyperbola keeps
    /// its 50 samples (this overload does not set the accuracy to 20).
    /// `inter.perform_parabola_hyperbola(p, &dp, h, &dh, 1e-9, 1e-9)?`.
    pub fn perform_parabola_hyperbola(
        &mut self,
        p: Parabola2,
        dp: &Domain,
        h: Hyperbola2,
        dh: &Domain,
        tol_conf: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        self.base.reset_fields();
        let itool: IConicTool = IConicTool::from_parabola(p);
        let pcurve: PConic = PConic::from_hyperbola(h);
        self.inter
            .set_reversed_parameters(self.base.reversed_parameters());
        let maxtol: f64 = scaled_max_tolerance(tol_conf, tol);
        let bounds: AnaBounds = hyperbola_range(h, &Conic::from_parabola(p), maxtol)?;
        let pair = ConicPair {
            itool: &itool,
            imp_domain: dp,
            pcurve: &pcurve,
            par_domain: dh,
        };
        self.perform_on_bounds(&pair, bounds, (tol_conf, tol), CLIP_EARLY)
    }

    /// `Perform(gp_Elips2d, DE, gp_Hypr2d, DH, TolConf, Tol)`: offsets of
    /// `MinorRadius/10`; a range clipped to nothing is an empty result.
    /// `inter.perform_ellipse_hyperbola(e, &de, h, &dh, 1e-9, 1e-9)?`.
    pub fn perform_ellipse_hyperbola(
        &mut self,
        e: Ellipse2,
        de: &Domain,
        h: Hyperbola2,
        dh: &Domain,
        tol_conf: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        self.base.reset_fields();
        let itool: IConicTool = IConicTool::from_ellipse(e);
        let pcurve: PConic = accurate(PConic::from_hyperbola(h));
        self.inter
            .set_reversed_parameters(self.base.reversed_parameters());
        let de_modif: Domain = closed_domain(de)?;
        let bounds: AnaBounds =
            hyperbola_range(h, &Conic::from_ellipse(e), e.minor_radius() / 10.0)?;
        let pair = ConicPair {
            itool: &itool,
            imp_domain: &de_modif,
            pcurve: &pcurve,
            par_domain: dh,
        };
        let mode = ClipMode {
            early: true,
            ordered: true,
        };
        self.perform_on_bounds(&pair, bounds, (tol_conf, tol), mode)
    }

    /// `Perform(gp_Hypr2d, DH1, gp_Hypr2d, DH2, TolConf, Tol)`: the clipping by a
    /// bounded domain has no early exit, only the final check that the range is not
    /// empty. `inter.perform_hyperbolas(h1, &d1, h2, &d2, 1e-9, 1e-9)?`.
    pub fn perform_hyperbolas(
        &mut self,
        h1: Hyperbola2,
        dh1: &Domain,
        h2: Hyperbola2,
        dh2: &Domain,
        tol_conf: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        self.base.reset_fields();
        let itool: IConicTool = IConicTool::from_hyperbola(h1);
        let pcurve: PConic = accurate(PConic::from_hyperbola(h2));
        self.inter
            .set_reversed_parameters(self.base.reversed_parameters());
        let maxtol: f64 = scaled_max_tolerance(tol_conf, tol);
        let bounds: AnaBounds = hyperbola_range(h2, &Conic::from_hyperbola(h1), maxtol)?;
        let pair = ConicPair {
            itool: &itool,
            imp_domain: dh1,
            pcurve: &pcurve,
            par_domain: dh2,
        };
        let mode = ClipMode {
            early: false,
            ordered: true,
        };
        self.perform_on_bounds(&pair, bounds, (tol_conf, tol), mode)
    }

    /// `Perform(gp_Circ2d, DC, gp_Elips2d, DE, TolConf, Tol)`: both domains closed,
    /// no analytic range. `inter.perform_circle_ellipse(c, &dc, e, &de, 1e-9, 1e-9)?`.
    pub fn perform_circle_ellipse(
        &mut self,
        c: Circle2,
        dc: &Domain,
        e: Ellipse2,
        de: &Domain,
        tol_conf: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        self.base.reset_fields();
        self.perform_closed(
            IConicTool::from_circle(c),
            dc,
            PConic::from_ellipse(e),
            de,
            (tol_conf, tol),
        )
    }

    /// `Perform(gp_Elips2d, DE1, gp_Elips2d, DE2, TolConf, Tol)`: both domains
    /// closed. `inter.perform_ellipses(e1, &d1, e2, &d2, 1e-9, 1e-9)?`.
    pub fn perform_ellipses(
        &mut self,
        e1: Ellipse2,
        de1: &Domain,
        e2: Ellipse2,
        de2: &Domain,
        tol_conf: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        self.base.reset_fields();
        self.perform_closed(
            IConicTool::from_ellipse(e1),
            de1,
            PConic::from_ellipse(e2),
            de2,
            (tol_conf, tol),
        )
    }

    // IntImpConicParConic on two closed conics (the first domain closed before the
    // second one, as natively).
    fn perform_closed(
        &mut self,
        itool: IConicTool,
        d1: &Domain,
        pcurve: PConic,
        d2: &Domain,
        (tol_conf, tol): (f64, f64),
    ) -> Result<(), Failure> {
        let pcurve: PConic = accurate(pcurve);
        self.inter
            .set_reversed_parameters(self.base.reversed_parameters());
        let closed1: Domain = closed_domain(d1)?;
        let closed2: Domain = closed_domain(d2)?;
        self.inter
            .perform(&itool, &closed1, &pcurve, &closed2, tol_conf, tol)?;
        self.base.set_values(&self.inter);
        Ok(())
    }
}

/// `PCurve.SetAccuracy(20)`.
pub(super) fn accurate(mut pcurve: PConic) -> PConic {
    pcurve.set_accuracy(20);
    pcurve
}

impl IntConicConic {
    /// `Perform(gp_Lin2d, DL, gp_Parab2d, DP, TolConf, Tol)`: the lines offset by
    /// `100 max(Tol, TolConf, 1e-7)` along `(dy, dx)` of the line direction (not its
    /// normal, as natively). On a bounded parabola domain the search first runs with
    /// the exact tolerance 1e-7, and again with the given ones when that finds no
    /// point. `inter.perform_line_parabola(l, &dl, p, &dp, 1e-9, 1e-9)?`.
    pub fn perform_line_parabola(
        &mut self,
        l: Line2,
        dl: &Domain,
        p: Parabola2,
        dp: &Domain,
        tol_conf: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        self.base.reset_fields();
        let itool: IConicTool = IConicTool::from_line(l)?;
        let pcurve: PConic = accurate(PConic::from_parabola(p));
        self.inter
            .set_reversed_parameters(self.base.reversed_parameters());
        let mut maxtol: f64 = if tol > tol_conf { tol } else { tol_conf };
        if maxtol < 1.0e-7 {
            maxtol = 1.0e-7;
        }
        maxtol *= 100.0;
        let direction = l.direction();
        let mut offset = Vector2::new(maxtol * direction.y(), maxtol * direction.x());
        let mut bounds: AnaBounds = AnaBounds::new();
        let mut ana: AnaIntersection = AnaIntersection::new();
        ana.perform_parabola_conic(p, &Conic::from_line(l.translated(offset)))?;
        bounds.widen(&ana, &p, maxtol, PARAM_MAX_ON_PARABOLA)?;
        offset.reverse();
        ana.perform_parabola_conic(p, &Conic::from_line(l.translated(offset)))?;
        bounds.widen(&ana, &p, maxtol, PARAM_MAX_ON_PARABOLA)?;
        let pair = ConicPair {
            itool: &itool,
            imp_domain: dl,
            pcurve: &pcurve,
            par_domain: dp,
        };
        let ordered: bool = matches!(
            bounds.binf.partial_cmp(&bounds.bsup),
            Some(Ordering::Less | Ordering::Equal)
        );
        if !bounded_domain(dp) || !ordered {
            return self.perform_on_bounds(&pair, bounds, (tol_conf, tol), CLIP_EARLY);
        }
        let Some(dp_modif) = bounds.clip(dp, true, false)? else {
            self.base.done = true;
            return Ok(());
        };
        self.inter.perform(
            &itool,
            dl,
            &pcurve,
            &dp_modif,
            TOL_EXACT_INTER,
            TOL_EXACT_INTER,
        )?;
        self.base.set_values(&self.inter);
        if self.base.is_done() && self.base.nb_points()? == 0 {
            self.base.reset_fields();
            self.inter
                .perform(&itool, dl, &pcurve, &dp_modif, tol_conf, tol)?;
            self.base.set_values(&self.inter);
        }
        Ok(())
    }
}
