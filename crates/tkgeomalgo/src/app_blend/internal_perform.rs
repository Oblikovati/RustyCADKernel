// Rust translation of ModelingAlgorithms/TKGeomAlgo/AppBlend/AppBlend_AppSurf.gxx (Perform,
// PerformSmoothing, InternalPerform) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AppBlend_AppSurf::Perform(Lin, F, SpApprox)` and `PerformSmoothing`: one
//! multi-line of all the sections, approximated by Bezier pieces (`AppDef_Compute`),
//! one B-spline (`AppDef_BSplineCompute`) or the variational smoothing.
use super::sections::SectionArrays;
use super::{AppBlendAppSurf, TheLine, TheSectionGenerator};
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkgeombase::app_def::{
    BSplineCompute, Compute, MultiLine, Variational, VariationalOptions,
};
use rustycad_tkgeombase::app_par_curves::{Constraint, ConstraintCouple, MultiBSpCurve};
use rustycad_tkgeombase::approx::{ComputeLineOptions, ParametrizationType};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::math::Vector;

/// `std::max(a, b)`: `b` when `a < b`, else `a` (NaN-faithful).
pub(super) fn std_max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}

/// The multi-line of all the sections and its end constraints.
struct SectionLine {
    line: MultiLine,
    first: Constraint,
    last: Constraint,
}

impl AppBlendAppSurf {
    /// `Perform(Lin, F, SpApprox)`: approximates the sections of `lin` (None is the
    /// null handle, which leaves the approximation not done); `sp_approx` asks for
    /// one B-spline instead of Bezier pieces;
    /// `approx.perform(Some(&line), &mut generator, false)?`.
    pub fn perform<L, G>(
        &mut self,
        lin: Option<&L>,
        f: &mut G,
        sp_approx: bool,
    ) -> Result<(), Failure>
    where
        L: TheLine,
        G: TheSectionGenerator<Point = L::Point>,
    {
        self.internal_perform(lin, f, sp_approx, false)
    }
    /// `PerformSmoothing(Lin, F)`: the variational smoothing of the sections;
    /// `approx.perform_smoothing(Some(&line), &mut generator)?`.
    pub fn perform_smoothing<L, G>(&mut self, lin: Option<&L>, f: &mut G) -> Result<(), Failure>
    where
        L: TheLine,
        G: TheSectionGenerator<Point = L::Point>,
    {
        self.internal_perform(lin, f, true, true)
    }
    fn internal_perform<L, G>(
        &mut self,
        lin: Option<&L>,
        f: &mut G,
        sp_approx: bool,
        smoothing: bool,
    ) -> Result<(), Failure>
    where
        L: TheLine,
        G: TheSectionGenerator<Point = L::Point>,
    {
        self.done = false;
        let Some(lin) = lin else {
            return Ok(());
        };
        self.seq_poles2d.clear();
        let line = MultiLine::with_nb_multi_points(lin.nb_points())?;
        let mut arrays: SectionArrays = self.section_shape(f)?;
        let sections: SectionLine = section_line(lin, f, (&mut arrays, line), sp_approx)?;
        if lin.nb_points() == 2 {
            self.narrow_two_point_degrees(sections.first, sections.last);
        }
        let Some(multc) = self.approximate(lin, f, &sections, (sp_approx, smoothing))? else {
            return Ok(());
        };
        let reparametrize = self.knownp && !smoothing;
        // The parameters are read only when the knots are reparametrized, after the
        // pole tables are allocated, as natively (a generator can raise there).
        self.store_solution(&multc, &arrays, |knots| {
            if reparametrize {
                let u1: f64 = f.parameter(&lin.point(1))?;
                let u2: f64 = f.parameter(&lin.point(lin.nb_points()))?;
                bsplclib::reparametrize(u1, u2, knots)?;
            }
            Ok(())
        })
    }
    /// `GetShape`, `Knots` and `Mults` of the generator into the U data, and the
    /// working arrays.
    pub(super) fn section_shape<G: TheSectionGenerator>(
        &mut self,
        f: &G,
    ) -> Result<SectionArrays, Failure> {
        let [nb_u_poles, nb_u_knots, udeg, nb_poles2d] = f.get_shape()?;
        self.udeg = udeg;
        self.tab_u_knots = one_based(nb_u_knots)?;
        self.tab_u_mults = one_based(nb_u_knots)?;
        f.knots(&mut self.tab_u_knots)?;
        f.mults(&mut self.tab_u_mults)?;
        SectionArrays::new(nb_u_poles, nb_poles2d)
    }
    /// "IFV 04.06.07 occ13904": two sections give degree 1 at least, and exactly 1
    /// without tangents; the stored degrees change for later computations too.
    pub(super) fn narrow_two_point_degrees(&mut self, first: Constraint, last: Constraint) {
        self.dmin = 1;
        if first == Constraint::PassPoint && last == Constraint::PassPoint {
            self.dmax = 1;
        }
    }
    /// The compute options of this approximation.
    pub(super) fn options(
        &self,
        parametrization: ParametrizationType,
        squares: bool,
    ) -> ComputeLineOptions {
        ComputeLineOptions {
            degree_min: self.dmin,
            degree_max: self.dmax,
            tolerance_3d: self.tol3d,
            tolerance_2d: self.tol2d,
            nb_iterations: self.nbit,
            cutting: true,
            parametrization,
            squares,
        }
    }
    /// The multi-curve of the chosen algorithm; None when the smoothing fails.
    fn approximate<L, G>(
        &mut self,
        lin: &L,
        f: &G,
        sections: &SectionLine,
        (sp_approx, smoothing): (bool, bool),
    ) -> Result<Option<MultiBSpCurve>, Failure>
    where
        L: TheLine,
        G: TheSectionGenerator<Point = L::Point>,
    {
        if !sp_approx {
            return self.bezier_pieces(lin, f, sections).map(Some);
        }
        if !smoothing {
            return self.one_bspline(lin, f, sections).map(Some);
        }
        self.variational(sections)
    }
    /// `AppDef_Compute`: Bezier pieces joined into a B-spline. Quirk kept: the
    /// reached tolerances are computed but not stored.
    fn bezier_pieces<L, G>(
        &self,
        lin: &L,
        f: &G,
        sections: &SectionLine,
    ) -> Result<MultiBSpCurve, Failure>
    where
        L: TheLine,
        G: TheSectionGenerator<Point = L::Point>,
    {
        let mut approx = Compute::new(self.options(self.paramtype, false));
        if self.knownp {
            let params: Vector<'static> = known_parameters(lin, f)?;
            approx = Compute::from_parameters(&params, self.options(self.paramtype, true))?;
        }
        approx.set_constraints(sections.first, sections.last);
        approx.perform(&sections.line)?;
        let (mut mytol3d, mut mytol2d) = (0.0, 0.0);
        for index in 1..=approx.nb_multi_curves() {
            let (tol3d, tol2d) = approx.error(index)?;
            (mytol3d, mytol2d) = (std_max(tol3d, mytol3d), std_max(tol2d, mytol2d));
        }
        let _ = (mytol3d, mytol2d);
        Ok(approx.spline_value()?.clone())
    }
    /// `AppDef_BSplineCompute`: one B-spline. Quirk kept: C0 ends as continuity 3
    /// (the native `if C0 ...; if C1 ... else if C2 ... else 3` chain).
    fn one_bspline<L, G>(
        &mut self,
        lin: &L,
        f: &G,
        sections: &SectionLine,
    ) -> Result<MultiBSpCurve, Failure>
    where
        L: TheLine,
        G: TheSectionGenerator<Point = L::Point>,
    {
        let use_squares: bool = self.nbit == 0;
        let mut approx = BSplineCompute::new(self.options(self.paramtype, use_squares));
        let continuity: i32 = match self.continuity {
            Shape::C1 => 1,
            Shape::C2 => 2,
            _ => 3,
        };
        approx.set_continuity(continuity);
        approx.set_constraints(sections.first, sections.last);
        if self.knownp {
            let params: Vector<'static> = known_parameters(lin, f)?;
            approx.init(self.options(ParametrizationType::IsoParametric, true));
            approx.set_parameters(&params)?;
        }
        approx.perform(&sections.line)?;
        (self.tol3dreached, self.tol2dreached) = approx.error();
        Ok(approx.value().clone())
    }
    /// `AppDef_Variational` with the stored settings; None when it is not created,
    /// over-constrained, raises during `Approximate` or is not done.
    fn variational(&mut self, sections: &SectionLine) -> Result<Option<MultiBSpCurve>, Failure> {
        let nb_point: i32 = sections.line.nb_multi_points()?;
        let mut couples: Array1<'static, ConstraintCouple> = one_based(nb_point)?;
        for i in 1..=nb_point {
            couples[i] = ConstraintCouple::new(i, Constraint::NoConstraint);
        }
        couples[1].set_constraint(sections.first);
        couples[nb_point].set_constraint(sections.last);
        let couples = Handle::new(HArray1::from_owned(couples));
        let mut variation = Variational::new(
            &sections.line,
            1,
            nb_point,
            &couples,
            VariationalOptions::default(),
        )?;
        variation.set_max_degree(self.dmax)?;
        variation.set_continuity(self.continuity)?;
        variation.set_max_segment(1000);
        variation.set_tolerance(self.tol3d)?;
        variation.set_with_min_max(false)?;
        variation.set_with_cutting(true)?;
        variation.set_nb_iterations(self.nbit);
        let [w1, w2, w3] = self.critweights;
        variation.set_criterium_weight(w1, w2, w3)?;
        if !variation.is_created() || variation.is_over_constrained() {
            return Ok(None);
        }
        // The native try/catch: a failed smoothing leaves the approximation not done.
        if variation.approximate().is_err() || !variation.is_done() {
            return Ok(None);
        }
        (self.tol3dreached, self.tol2dreached) = (variation.max_error()?, 0.0);
        variation.value().map(Some)
    }
}

/// The section parameters of the line points mapped onto [0, 1].
fn known_parameters<L, G>(lin: &L, f: &G) -> Result<Vector<'static>, Failure>
where
    L: TheLine,
    G: TheSectionGenerator<Point = L::Point>,
{
    let nb_point: i32 = lin.nb_points();
    let mut params: Vector<'static> = Vector::filled(1, nb_point, 0.0)?;
    params[1] = 0.0;
    params[nb_point] = 1.0;
    let uf: f64 = f.parameter(&lin.point(1))?;
    let ul: f64 = f.parameter(&lin.point(nb_point))? - uf;
    for i in 2..nb_point {
        params[i] = (f.parameter(&lin.point(i))? - uf) / ul;
    }
    Ok(params)
}

/// The multi-line of every section: tangency constraints at the ends when the
/// generator gives derivatives there, and the inner sections without derivatives
/// for the B-spline approximations.
fn section_line<L, G>(
    lin: &L,
    f: &mut G,
    (arrays, line): (&mut SectionArrays, MultiLine),
    sp_approx: bool,
) -> Result<SectionLine, Failure>
where
    L: TheLine,
    G: TheSectionGenerator<Point = L::Point>,
{
    let nb_point: i32 = lin.nb_points();
    let end_constraint = |withderiv: bool| {
        if withderiv {
            Constraint::TangencyPoint
        } else {
            Constraint::PassPoint
        }
    };
    let withderiv: bool = arrays.load_d1(f, &lin.point(1))?;
    arrays.weighted(withderiv, None);
    line.set_value(1, &arrays.constraint(withderiv)?)?;
    let first: Constraint = end_constraint(withderiv);
    for i in 2..nb_point {
        let point = lin.point(i);
        let withderiv: bool = if sp_approx {
            f.section(&point, &mut arrays.section)?;
            false
        } else {
            arrays.load_d1(f, &point)?
        };
        arrays.weighted(withderiv, None);
        line.set_value(i, &arrays.constraint(withderiv)?)?;
    }
    let withderiv: bool = arrays.load_d1(f, &lin.point(nb_point))?;
    arrays.weighted(withderiv, None);
    line.set_value(nb_point, &arrays.constraint(withderiv)?)?;
    Ok(SectionLine {
        line,
        first,
        last: end_constraint(withderiv),
    })
}
