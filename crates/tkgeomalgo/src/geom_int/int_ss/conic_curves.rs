//! The curves of the conic lines: the parts of a line, parabola or hyperbola (an
//! infinite part is kept when a point of it is inside both domains), and of a circle or
//! an ellipse (a whole turn is kept when one of 18 sample points is inside).
use super::GeomIntIntSS;
use super::internal_perform::CurveInputs;
use crate::geom_int::line_constructor::helpers::quadric_parameters;
use crate::geom_int::line_tool::line_as;
use crate::int_patch::{IntPatchGLine, IntPatchIType, IntPatchLine};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::real_epsilon;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{
    GeomCircle, GeomCurve, GeomEllipse, GeomHyperbola, GeomLine, GeomParabola, GeomTrimmedCurve,
};
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point2, Point3};
use std::f64::consts::PI;

/// `Parameters(HS1, HS2, Ptref, U1, V1, U2, V2)` of `GeomInt_IntSS_1.cxx`: the
/// parameters on both quadrics (`GetQuadric` raises Standard_ConstructionError for
/// another surface).
fn parameters(ss: &GeomIntIntSS, ptref: Point3) -> Result<[f64; 4], Failure> {
    const MEMBER: &str = "GeomInt_IntSS::MakeCurve";
    let (u1, v1) = quadric_parameters(&ss.my_hs1, ptref, MEMBER)?;
    let (u2, v2) = quadric_parameters(&ss.my_hs2, ptref, MEMBER)?;
    Ok([u1, v1, u2, v2])
}

/// Whether the point of the curve is not OUT of both domains (with
/// `Precision::Confusion()`).
fn inside_domains(
    ss: &GeomIntIntSS,
    inputs: &CurveInputs<'_>,
    ptref: Point3,
) -> Result<bool, Failure> {
    let uv: [f64; 4] = parameters(ss, ptref)?;
    let tol_x: f64 = precision::confusion();
    if inputs
        .dom1
        .classify(Point2::new(uv[0], uv[1]), tol_x, true)?
        == State::Out
    {
        return Ok(false);
    }
    Ok(inputs
        .dom2
        .classify(Point2::new(uv[2], uv[3]), tol_x, true)?
        != State::Out)
}

impl GeomIntIntSS {
    /// The 2d curve on the surface `k` (1 or 2) of `newc` over `first..last` when
    /// `wanted` (else null); the reached 2d tolerance is updated.
    pub(super) fn conic_pcurve(
        &mut self,
        (k, wanted): (i32, bool),
        (first, last): (f64, f64),
        tolpc: &mut f64,
        newc: &Handle<dyn GeomCurve>,
    ) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        if !wanted {
            return Ok(Handle::default());
        }
        let surface = if k == 1 {
            self.my_hs1.surface()
        } else {
            self.my_hs2.surface()
        };
        let mut c2d: Handle<dyn Geom2dCurve> = Handle::default();
        Self::build_pcurves(first, last, tolpc, &surface, newc, &mut c2d)?;
        if *tolpc > self.my_tol_reached2d || self.my_tol_reached2d == 0.0 {
            self.my_tol_reached2d = *tolpc;
        }
        Ok(c2d)
    }

    /// The curve of a line, parabola or hyperbola line.
    fn open_conic(l: &Handle<dyn IntPatchLine>) -> Result<Handle<dyn GeomCurve>, Failure> {
        let gline: Handle<IntPatchGLine> = line_as::<IntPatchGLine>(l, "GeomInt_IntSS::MakeCurve")?;
        Ok(match l.arc_type() {
            IntPatchIType::Lin => Handle::new(GeomLine::from_lin(gline.line()?)).cast(),
            IntPatchIType::Parabola => {
                Handle::new(GeomParabola::from_parab(gline.parabola()?)).cast()
            }
            _ => Handle::new(GeomHyperbola::from_hypr(gline.hyperbola()?)).cast(),
        })
    }

    /// The curves of the parts of a line, parabola or hyperbola line.
    pub(super) fn open_conic_curves(
        &mut self,
        l: &Handle<dyn IntPatchLine>,
        inputs: &CurveInputs<'_>,
        tolpc: &mut f64,
    ) -> Result<(), Failure> {
        let newc: Handle<dyn GeomCurve> = Self::open_conic(l)?;
        let nb_parts: i32 = self.my_lconstruct.nb_parts()?;
        for i in 1..=nb_parts {
            let (fprm, lprm) = self.my_lconstruct.part(i)?;
            if !precision::is_negative_infinite(fprm) && !precision::is_positive_infinite(lprm) {
                self.finite_part(&newc, (fprm, lprm), inputs, tolpc)?;
            } else {
                self.infinite_part(&newc, (fprm, lprm), inputs)?;
            }
        }
        Ok(())
    }

    /// A finite part: the trimmed curve and the trimmed 2d curves.
    fn finite_part(
        &mut self,
        newc: &Handle<dyn GeomCurve>,
        (fprm, lprm): (f64, f64),
        inputs: &CurveInputs<'_>,
        tolpc: &mut f64,
    ) -> Result<(), Failure> {
        let ct3d = GeomTrimmedCurve::new(newc, fprm, lprm, true, true)?;
        self.sline.push(Handle::new(ct3d).cast());
        for (k, wanted) in [(1, inputs.approx.approx_s1), (2, inputs.approx.approx_s2)] {
            let c2d = self.conic_pcurve((k, wanted), (fprm, lprm), tolpc, newc)?;
            let trimmed: Handle<dyn Geom2dCurve> = if wanted {
                Handle::new(Geom2dTrimmedCurve::new(&c2d, fprm, lprm, true, true)?).cast()
            } else {
                c2d
            };
            if k == 1 {
                self.sline_s1.push(trimmed);
            } else {
                self.sline_s2.push(trimmed);
            }
        }
        Ok(())
    }

    /// An infinite part: kept whole (without 2d curves) on a swept or offset surface,
    /// else when its point at 100 from its finite end (or at 0) is inside both domains.
    fn infinite_part(
        &mut self,
        newc: &Handle<dyn GeomCurve>,
        (fprm, lprm): (f64, f64),
        inputs: &CurveInputs<'_>,
    ) -> Result<(), Failure> {
        let special = |t: SurfaceType| {
            matches!(
                t,
                SurfaceType::SurfaceOfExtrusion
                    | SurfaceType::OffsetSurface
                    | SurfaceType::SurfaceOfRevolution
            )
        };
        let keep: bool = if special(self.my_hs1.get_type()?) || special(self.my_hs2.get_type()?) {
            true
        } else {
            let dt: f64 = 100.0;
            let b_fn_it: bool = precision::is_negative_infinite(fprm);
            let b_lp_it: bool = precision::is_positive_infinite(lprm);
            let test_prm: f64 = match (b_fn_it, b_lp_it) {
                (true, false) => lprm - dt,
                (false, true) => fprm + dt,
                _ => 0.0,
            };
            inside_domains(self, inputs, newc.value(test_prm)?)?
        };
        if keep {
            self.push_curve_only(newc.clone());
        }
        Ok(())
    }

    /// Appends a curve without 2d curves.
    pub(super) fn push_curve_only(&mut self, c: Handle<dyn GeomCurve>) {
        self.sline.push(c);
        self.sline_s1.push(Handle::default());
        self.sline_s2.push(Handle::default());
    }

    /// The curve of a circle or ellipse line.
    fn closed_conic(l: &Handle<dyn IntPatchLine>) -> Result<Handle<dyn GeomCurve>, Failure> {
        let gline: Handle<IntPatchGLine> = line_as::<IntPatchGLine>(l, "GeomInt_IntSS::MakeCurve")?;
        Ok(if l.arc_type() == IntPatchIType::Circle {
            Handle::new(GeomCircle::from_circ(gline.circle()?)).cast()
        } else {
            Handle::new(GeomEllipse::from_elips(gline.ellipse()?)).cast()
        })
    }

    /// The curves of the parts of a circle or ellipse line: a part other than the whole
    /// turn is trimmed; the whole turn of a single part is trimmed too; otherwise the
    /// whole curve is kept when one of 18 sample points is inside.
    pub(super) fn closed_conic_curves(
        &mut self,
        l: &Handle<dyn IntPatchLine>,
        inputs: &CurveInputs<'_>,
        tolpc: &mut f64,
    ) -> Result<(), Failure> {
        let newc: Handle<dyn GeomCurve> = Self::closed_conic(l)?;
        let real_eps: f64 = real_epsilon();
        let period: f64 = PI + PI;
        let nb_parts: i32 = self.my_lconstruct.nb_parts()?;
        for i in 1..=nb_parts {
            let (fprm, lprm) = self.my_lconstruct.part(i)?;
            if fprm.abs() > real_eps || (lprm - period).abs() > real_eps {
                self.trimmed_closed_part(&newc, (fprm, lprm), inputs, tolpc)?;
                continue;
            }
            // Whether it is kept.
            if nb_parts == 1
                && fprm.abs() < real_epsilon()
                && (lprm - 2.0 * PI).abs() < real_epsilon()
            {
                self.trimmed_closed_part(&newc, (fprm, lprm), inputs, tolpc)?;
                break;
            }
            self.sampled_closed_part(&newc, (fprm, lprm), inputs, tolpc)?;
        }
        Ok(())
    }

    /// A trimmed part of a closed conic, with its (untrimmed) 2d curves over the bounds
    /// of the trimmed curve.
    fn trimmed_closed_part(
        &mut self,
        newc: &Handle<dyn GeomCurve>,
        (fprm, lprm): (f64, f64),
        inputs: &CurveInputs<'_>,
        tolpc: &mut f64,
    ) -> Result<(), Failure> {
        let tc3d = GeomTrimmedCurve::new(newc, fprm, lprm, true, true)?;
        let range: (f64, f64) = (tc3d.first_parameter(), tc3d.last_parameter());
        self.sline.push(Handle::new(tc3d).cast());
        self.push_pcurves(newc, range, inputs, tolpc)
    }

    /// The 2d curves of `newc` over `range` on both surfaces (null when not wanted).
    fn push_pcurves(
        &mut self,
        newc: &Handle<dyn GeomCurve>,
        range: (f64, f64),
        inputs: &CurveInputs<'_>,
        tolpc: &mut f64,
    ) -> Result<(), Failure> {
        let c2d1 = self.conic_pcurve((1, inputs.approx.approx_s1), range, tolpc, newc)?;
        self.sline_s1.push(c2d1);
        let c2d2 = self.conic_pcurve((2, inputs.approx.approx_s2), range, tolpc, newc)?;
        self.sline_s2.push(c2d2);
        Ok(())
    }

    /// The whole closed conic when one of its points at `j * 2 Pi / 17` (`j = 0..=17`)
    /// is inside both domains.
    fn sampled_closed_part(
        &mut self,
        newc: &Handle<dyn GeomCurve>,
        range: (f64, f64),
        inputs: &CurveInputs<'_>,
        tolpc: &mut f64,
    ) -> Result<(), Failure> {
        let two_pi_div17: f64 = 2.0 * PI / 17.0;
        for j in 0..=17 {
            let ptref: Point3 = newc.value(f64::from(j) * two_pi_div17)?;
            if inside_domains(self, inputs, ptref)? {
                self.sline.push(newc.clone());
                return self.push_pcurves(newc, range, inputs, tolpc);
            }
        }
        Ok(())
    }
}
