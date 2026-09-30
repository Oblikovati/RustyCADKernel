// Rust translation of ModelingData/TKGeomBase/Approx/Approx_BSplComputeLine.gxx (Compute)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Approx_BSplComputeLine::Compute`: the B-spline fit of the whole line over given
//! knots, for each degree.
use super::BSplComputeLine;
use crate::app_par_curves::arrays::owned_vector;
use crate::app_par_curves::{
    BSpGradient, BSpGradientInput, Constraint, Gradient, LeastSquare, LineTool, MultiBSpCurve,
};
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::Vector;

/// The parameter range a fit may keep: `(-1e-6, 1 + 1e-6)`.
const PARAMETER_MARGIN: f64 = 0.000001;

/// One degree's fit: its curve and errors.
struct DegreeFit {
    curve: MultiBSpCurve,
    tolerances: [f64; 2],
}

impl BSplComputeLine {
    /// `Compute(Line, fpt, lpt, Para, Knots, Mults)`: for each degree, the
    /// multiplicities (unless given), then the least-squares, B-spline gradient or
    /// Bezier gradient fit; interpolates instead when the poles and end constraints
    /// outnumber the points. Keeps the best fit and returns true once one is within
    /// the tolerances.
    pub(super) fn compute<L: LineTool + Clone>(
        &mut self,
        line: &L,
        [fpt, lpt]: [i32; 2],
        para: &Vector<'static>,
        knots: &Array1<'static, f64>,
        mults: &mut Array1<'static, i32>,
    ) -> Result<bool, Failure> {
        let nbp: i32 = lpt - fpt + 1;
        (self.lambda1, self.lambda2) = (0.0, 0.0);
        for deg in self.options.degree_min..=self.options.degree_max {
            let mut params: Vector<'static> = owned_vector(para)?;
            let nbpoles: i32 = self.pole_count(deg, mults)?;
            if nbpoles + self.end_constraint_poles() > nbp {
                self.interpol(line)?;
                self.tolreached = true;
                return Ok(true);
            }
            let knotting = Knotting { knots, mults };
            let fit: Option<DegreeFit> =
                self.fit_degree(line, [fpt, lpt], &mut params, knotting, [deg, nbpoles])?;
            let Some(fit) = fit else {
                continue;
            };
            if self.keep(fit, &params)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    // The pole count of the degree; the multiplicities follow the degree and the
    // continuity unless given (then `Sum(Mults) - deg - 1`).
    fn pole_count(&self, deg: i32, mults: &mut Array1<'static, i32>) -> Result<i32, Failure> {
        if self.has_mults {
            return Ok(mults.iter().fold(-deg - 1, |sum, m| sum + m));
        }
        let (lower, upper) = (mults.lower(), mults.upper());
        let set = |m: &mut Array1<'static, i32>, i: i32, v: i32| {
            m.set_value(i, v)
                .map_err(|e| array_failure(e, "Approx_BSplComputeLine::Compute"))
        };
        set(mults, lower, deg + 1)?;
        set(mults, upper, deg + 1)?;
        let multinter: i32 = if self.continuity == -1 {
            1
        } else {
            (deg - self.continuity).max(1)
        };
        let mut nbpoles: i32 = deg + 1;
        for i in lower + 1..=upper - 1 {
            mults[i] = multinter;
            nbpoles += multinter;
        }
        Ok(nbpoles)
    }

    // One more unknown per tangency or curvature end.
    fn end_constraint_poles(&self) -> i32 {
        [self.real_first_c, self.real_last_c]
            .into_iter()
            .map(|c| i32::from(c == Constraint::TangencyPoint || c == Constraint::CurvaturePoint))
            .sum()
    }

    // The fit of one degree; None when it is not done (the native `continue`).
    fn fit_degree<L: LineTool + Clone>(
        &mut self,
        line: &L,
        range: [i32; 2],
        params: &mut Vector<'static>,
        knotting: Knotting<'_>,
        [deg, nbpoles]: [i32; 2],
    ) -> Result<Option<DegreeFit>, Failure> {
        if self.options.squares {
            return self.least_squares(line, range, params, knotting, nbpoles);
        }
        if nbpoles != deg + 1 {
            return self.bspline_gradient(line, range, params, knotting, [deg, nbpoles]);
        }
        self.bezier_gradient(line, range, params, knotting, deg)
    }

    fn least_squares<L: LineTool + Clone>(
        &self,
        line: &L,
        [fpt, lpt]: [i32; 2],
        params: &Vector<'static>,
        Knotting { knots, mults }: Knotting<'_>,
        nbpoles: i32,
    ) -> Result<Option<DegreeFit>, Failure> {
        let (first_c, last_c) = (self.real_first_c, self.real_last_c);
        let mut sq = LeastSquare::with_knots(
            line, knots, mults, fpt, lpt, first_c, last_c, params, nbpoles,
        )?;
        if !sq.is_done() {
            return Ok(None);
        }
        let curve: MultiBSpCurve = sq.bspline_value()?.clone();
        let (_, tol3d, tol2d) = sq.error()?;
        Ok(Some(DegreeFit {
            curve,
            tolerances: [tol3d, tol2d],
        }))
    }

    // The B-spline gradient; at the lowest degree with tangent ends, the lambdas of
    // a plain fit start it.
    fn bspline_gradient<L: LineTool + Clone>(
        &mut self,
        line: &L,
        [fpt, lpt]: [i32; 2],
        params: &mut Vector<'static>,
        Knotting { knots, mults }: Knotting<'_>,
        [deg, nbpoles]: [i32; 2],
    ) -> Result<Option<DegreeFit>, Failure> {
        let (first_c, last_c) = (self.real_first_c, self.real_last_c);
        let tangent_end: bool =
            first_c >= Constraint::TangencyPoint || last_c >= Constraint::TangencyPoint;
        if deg == self.options.degree_min && tangent_end {
            let fitt = LeastSquare::with_knots(
                line, knots, mults, fpt, lpt, first_c, last_c, params, nbpoles,
            )?;
            self.lambda1 = fitt.first_lambda() * f64::from(deg);
            self.lambda2 = fitt.last_lambda() * f64::from(deg);
        }
        let (l1, l2) = (self.lambda1 / f64::from(deg), self.lambda2 / f64::from(deg));
        let constraints = self.constraints();
        let input = BSpGradientInput {
            ssp: line,
            points: [fpt, lpt],
            constraints: &constraints,
            knots,
            mults,
            deg,
            tolerances: [self.options.tolerance_3d, self.options.tolerance_2d],
            nb_iterations: self.options.nb_iterations,
        };
        let grad: BSpGradient = BSpGradient::with_lambdas(&input, params, l1, l2)?;
        if !grad.is_done() {
            return Ok(None);
        }
        Ok(Some(DegreeFit {
            curve: grad.value(),
            tolerances: [grad.max_error_3d(), grad.max_error_2d()],
        }))
    }

    // One Bezier span (`nbpoles == deg + 1`): the Bezier gradient, over the knots.
    fn bezier_gradient<L: LineTool + Clone>(
        &self,
        line: &L,
        [fpt, lpt]: [i32; 2],
        params: &mut Vector<'static>,
        Knotting { knots, mults }: Knotting<'_>,
        deg: i32,
    ) -> Result<Option<DegreeFit>, Failure> {
        let [tol3d, tol2d] = [self.options.tolerance_3d, self.options.tolerance_2d];
        let iterations: i32 = self.options.nb_iterations;
        let constraints = self.constraints();
        let grad = Gradient::new(
            line,
            fpt,
            lpt,
            &constraints,
            params,
            deg,
            tol3d,
            tol2d,
            iterations,
        )?;
        if !grad.is_done() || grad.value().nb_curves()? == 0 {
            return Ok(None);
        }
        Ok(Some(DegreeFit {
            curve: MultiBSpCurve::from_multi_curve(&grad.value(), knots, mults),
            tolerances: [grad.max_error_3d(), grad.max_error_2d()],
        }))
    }

    // Keeps the fit when it is within the tolerances (true) or better than the kept
    // one; its parameters replace the stored ones when they stay in [0, 1].
    fn keep(&mut self, fit: DegreeFit, params: &Vector<'static>) -> Result<bool, Failure> {
        let save: bool = (params.lower()..=params.upper()).all(|i| {
            let u: f64 = params[i];
            !(u <= -PARAMETER_MARGIN || u >= 1.0 + PARAMETER_MARGIN)
        });
        let [tol3d, tol2d] = fit.tolerances;
        let fits: bool = tol3d <= self.options.tolerance_3d && tol2d <= self.options.tolerance_2d;
        if !fits && !(tol3d <= self.currenttol3d && tol2d <= self.currenttol2d) {
            return Ok(false);
        }
        self.tolreached |= fits;
        self.the_multi_bsp_curve = fit.curve;
        (self.currenttol3d, self.currenttol2d) = (tol3d, tol2d);
        if save {
            self.store_parameters(params)?;
        }
        Ok(fits)
    }

    fn store_parameters(&mut self, params: &Vector<'static>) -> Result<(), Failure> {
        let stored: &mut Array1<'static, f64> = self.my_parameters.as_mut().ok_or_else(|| {
            crate::native_failure::failure(
                "Standard_NullObject",
                "Approx_BSplComputeLine: null myParameters",
            )
        })?;
        for i in params.lower()..=params.upper() {
            stored
                .set_value(i, params[i])
                .map_err(|e| array_failure(e, "Approx_BSplComputeLine::Compute"))?;
        }
        Ok(())
    }
}

/// The knots and multiplicities of a fit.
#[derive(Clone, Copy)]
struct Knotting<'a> {
    knots: &'a Array1<'static, f64>,
    mults: &'a Array1<'static, i32>,
}
