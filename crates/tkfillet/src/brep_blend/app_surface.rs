// Rust translation of BRepBlend_AppSurface.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBlend_AppSurface`: the approximation of a blend surface by a sweep of its
//! sections (`Approx_SweepApproximation`), with the continuity the function can give.
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkgeomalgo::app_blend::{AppBlendApprox, AppBlendCurve2d, AppBlendSurface};
use rustycad_tkgeombase::approx::{
    SweepApproximation, SweepD1, SweepD2, SweepFunction, SweepOptions,
};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};

/// The tolerances and limits of a `BRepBlend_AppSurface` approximation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AppSurfaceOptions {
    /// `Tol3d` (also the boundary tolerance).
    pub tol3d: f64,
    /// `Tol2d`.
    pub tol2d: f64,
    /// `TolAngular`.
    pub tol_angular: f64,
    /// `Continuity`: lowered to what the function can derive.
    pub continuity: Shape,
    /// `Degmax`.
    pub degmax: i32,
    /// `Segmax`.
    pub segmax: i32,
}

/// `BRepBlend_AppSurface`: the sweep approximation of a blend function.
///
/// ```ignore
/// let options = AppSurfaceOptions { tol3d: 1e-4, tol2d: 1e-5, tol_angular: 1e-2,
///     continuity: Shape::C1, degmax: 11, segmax: 50 };
/// let app = BRepBlendAppSurface::new(&sweep_function, first, last, options)?;
/// ```
pub struct BRepBlendAppSurface {
    approx: SweepApproximation,
}

fn filled<T: Clone + Default>(n: i32) -> Result<Array1<'static, T>, Failure> {
    let mut array = Array1::with_bounds(1, n)
        .map_err(|e| Failure::with_message("Standard_RangeError", &format!("{e:?}")))?;
    array.init(T::default());
    Ok(array)
}

/// The continuity the function can give at `first`: C2 when `D2` succeeds, C1 when `D1`
/// does, C0 otherwise (the sections are computed into scratch arrays, natively aliased).
fn derivable_continuity(
    func: &dyn SweepFunction,
    (first, last): (f64, f64),
    continuity: Shape,
) -> Result<Shape, Failure> {
    let mut continuity = continuity;
    let nb2d = func.nb_2d_curves()?;
    if continuity == Shape::C0 {
        return Ok(continuity);
    }
    let nb2d = if nb2d == 0 { 1 } else { nb2d };
    let (nb_pol, _, _) = func.section_shape()?;
    let (mut p, mut v, mut v2, mut w, mut w1, mut w2) = (
        filled::<Point3>(nb_pol)?,
        filled::<Vector3>(nb_pol)?,
        filled::<Vector3>(nb_pol)?,
        filled::<f64>(nb_pol)?,
        filled::<f64>(nb_pol)?,
        filled::<f64>(nb_pol)?,
    );
    let (mut p2d, mut v2d, mut v2d2) = (
        filled::<Point2>(nb2d)?,
        filled::<Vector2>(nb2d)?,
        filled::<Vector2>(nb2d)?,
    );
    if continuity == Shape::C2 {
        let d1 = SweepD1 {
            poles: &mut p,
            d_poles: &mut v,
            poles2d: &mut p2d,
            d_poles2d: &mut v2d,
            weights: &mut w,
            d_weights: &mut w1,
        };
        let ok = func.d2(
            first,
            first,
            last,
            SweepD2 {
                d1,
                d2_poles: &mut v2,
                d2_poles2d: &mut v2d2,
                d2_weights: &mut w2,
            },
        )?;
        if !ok {
            continuity = Shape::C1;
        }
    }
    if continuity == Shape::C1 {
        let ok = func.d1(
            first,
            first,
            last,
            SweepD1 {
                poles: &mut p,
                d_poles: &mut v,
                poles2d: &mut p2d,
                d_poles2d: &mut v2d,
                weights: &mut w,
                d_weights: &mut w1,
            },
        )?;
        if !ok {
            continuity = Shape::C0;
        }
    }
    Ok(continuity)
}

impl BRepBlendAppSurface {
    /// `BRepBlend_AppSurface(Funct, First, Last, Tol3d, Tol2d, TolAngular, Continuity,
    /// Degmax, Segmax)`: the approximation, at the continuity the function can give.
    pub fn new(
        func: &Handle<dyn SweepFunction>,
        first: f64,
        last: f64,
        options: AppSurfaceOptions,
    ) -> Result<Self, Failure> {
        let mut approx = SweepApproximation::new(func);
        let live = func.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "BRepBlend_AppSurface: null sweep function",
            )
        })?;
        let continuity = derivable_continuity(live, (first, last), options.continuity)?;
        let sweep = SweepOptions {
            tol_3d: options.tol3d,
            bound_tol: options.tol3d,
            tol_2d: options.tol2d,
            tol_angular: options.tol_angular,
            continuity,
            degmax: options.degmax,
            segmax: options.segmax,
        };
        approx.perform(first, last, sweep)?;
        Ok(Self { approx })
    }

    /// `MaxErrorOnSurf()`.
    pub fn max_error_on_surf(&self) -> Result<f64, Failure> {
        self.approx.max_error_on_surf()
    }

    /// `Max2dError(Index)`.
    pub fn max_2d_error(&self, index: i32) -> Result<f64, Failure> {
        self.approx.max_2d_error(index)
    }

    /// `TolCurveOnSurf(Index)` with its failure.
    pub fn tol_curve_on_surf_checked(&self, index: i32) -> Result<f64, Failure> {
        self.approx.tol_curve_on_surf(index)
    }

    /// `Dump(o)`.
    pub fn dump(&self, stream: &mut dyn std::fmt::Write) -> Result<(), Failure> {
        self.approx.dump(stream)
    }
}

impl AppBlendApprox for BRepBlendAppSurface {
    fn is_done(&self) -> bool {
        self.approx.is_done()
    }
    fn surf_shape(&self) -> Result<[i32; 6], Failure> {
        let s = self.approx.surf_shape()?;
        Ok([
            s.u_degree,
            s.v_degree,
            s.nb_u_poles,
            s.nb_v_poles,
            s.nb_u_knots,
            s.nb_v_knots,
        ])
    }
    fn surface(&self) -> Result<AppBlendSurface, Failure> {
        let s = self.approx.surface()?;
        Ok(AppBlendSurface {
            poles: s.poles,
            weights: s.weights,
            u_knots: s.u_knots,
            v_knots: s.v_knots,
            u_mults: s.u_mults,
            v_mults: s.v_mults,
        })
    }
    fn u_degree(&self) -> Result<i32, Failure> {
        self.approx.u_degree()
    }
    fn v_degree(&self) -> Result<i32, Failure> {
        self.approx.v_degree()
    }
    fn surf_poles(&self) -> Result<&Array2<'static, Point3>, Failure> {
        self.approx.surf_poles()
    }
    fn surf_weights(&self) -> Result<&Array2<'static, f64>, Failure> {
        self.approx.surf_weights()
    }
    fn surf_u_knots(&self) -> Result<&Array1<'static, f64>, Failure> {
        self.approx.surf_u_knots()
    }
    fn surf_v_knots(&self) -> Result<&Array1<'static, f64>, Failure> {
        self.approx.surf_v_knots()
    }
    fn surf_u_mults(&self) -> Result<&Array1<'static, i32>, Failure> {
        self.approx.surf_u_mults()
    }
    fn surf_v_mults(&self) -> Result<&Array1<'static, i32>, Failure> {
        self.approx.surf_v_mults()
    }
    fn nb_curves2d(&self) -> Result<i32, Failure> {
        self.approx.nb_curves_2d()
    }
    fn curves2d_shape(&self) -> Result<[i32; 3], Failure> {
        let (degree, nb_poles, nb_knots) = self.approx.curves_2d_shape()?;
        Ok([degree, nb_poles, nb_knots])
    }
    fn curve2d(&self, index: i32) -> Result<AppBlendCurve2d, Failure> {
        let c = self.approx.curve_2d(index)?;
        Ok(AppBlendCurve2d {
            poles: c.poles,
            knots: c.knots,
            mults: c.mults,
        })
    }
    fn curves2d_degree(&self) -> Result<i32, Failure> {
        self.approx.curves_2d_degree()
    }
    fn curve2d_poles(&self, index: i32) -> Result<&Array1<'static, Point2>, Failure> {
        self.approx.curve_2d_poles(index)
    }
    fn curves2d_knots(&self) -> Result<&Array1<'static, f64>, Failure> {
        self.approx.curves_2d_knots()
    }
    fn curves2d_mults(&self) -> Result<&Array1<'static, i32>, Failure> {
        self.approx.curves_2d_mults()
    }
    /// `TolReached(Tol3d, Tol2d)`: the surface error and the largest 2d error.
    fn tol_reached(&self) -> Result<(f64, f64), Failure> {
        let tol3d = self.approx.max_error_on_surf()?;
        let mut tol2d = 0.;
        for ii in 1..=self.approx.nb_curves_2d()? {
            tol2d = std_max(tol2d, self.approx.max_2d_error(ii)?);
        }
        Ok((tol3d, tol2d))
    }
    /// `TolCurveOnSurf(Index)`; the trait has no failure, so a native exception (no 2d
    /// curve, an index out of range) reads NaN here; `tol_curve_on_surf_checked` returns it.
    fn tol_curve_on_surf(&self, index: i32) -> f64 {
        self.approx.tol_curve_on_surf(index).unwrap_or(f64::NAN)
    }
}
