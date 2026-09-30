// Rust translation of ModelingAlgorithms/TKGeomAlgo/Law/Law_S.hxx/.cxx and
// Law_Interpol.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Law_S` and `Law_Interpol`, the two laws derived from `Law_BSpFunc`. Each embeds
//! its `Law_BSpFunc` parent, reaches its members through `Deref` and forwards the
//! inherited `Law_Function` members, which it does not override.
use super::{LawBSpFunc, LawBSpline, LawFunction, LawInterpolate};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::{at, one_based, put};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Point2;

/// Implements `LawFunction` for a law derived from `Law_BSpFunc` by forwarding every
/// member to its `parent` field.
macro_rules! forward_bsp_func {
    ($law:ty) => {
        impl LawFunction for $law {
            fn continuity(&self) -> Result<Shape, Failure> {
                self.parent.continuity()
            }
            fn nb_intervals(&self, s: Shape) -> Result<i32, Failure> {
                self.parent.nb_intervals(s)
            }
            fn intervals(&self, t: &mut Array1<'_, f64>, s: Shape) -> Result<(), Failure> {
                self.parent.intervals(t, s)
            }
            fn value(&self, x: f64) -> Result<f64, Failure> {
                self.parent.value(x)
            }
            fn d1(&self, x: f64) -> Result<(f64, f64), Failure> {
                self.parent.d1(x)
            }
            fn d2(&self, x: f64) -> Result<(f64, f64, f64), Failure> {
                self.parent.d2(x)
            }
            fn trim(
                &self,
                first: f64,
                last: f64,
                tol: f64,
            ) -> Result<Handle<dyn LawFunction>, Failure> {
                self.parent.trim(first, last, tol)
            }
            fn bounds(&self) -> Result<(f64, f64), Failure> {
                self.parent.bounds()
            }
        }
        impl std::ops::Deref for $law {
            type Target = LawBSpFunc;
            fn deref(&self) -> &LawBSpFunc {
                &self.parent
            }
        }
    };
}

/// A cubic "S" law between two values with given end derivatives (`Law_S`).
///
/// `let law = LawS::new(); law.set(0.0, 1.0, 1.0, 2.0)?; law.value(0.5)?`.
#[derive(Default)]
pub struct LawS {
    parent: LawBSpFunc,
}
impl_transient!(LawS, "Law_S", LawBSpFunc, parent, interfaces: [dyn LawFunction]);
forward_bsp_func!(LawS);

impl LawS {
    /// A law without curve (`Law_S()`); `LawS::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// The S law from `valdeb` at `pdeb` to `valfin` at `pfin` with null end
    /// derivatives (`Set(Pdeb, Valdeb, Pfin, Valfin)`); `law.set(0.0, 1.0, 1.0, 2.0)?`.
    pub fn set(&self, pdeb: f64, valdeb: f64, pfin: f64, valfin: f64) -> Result<(), Failure> {
        self.set_with_derivatives(pdeb, valdeb, 0.0, pfin, valfin, 0.0)
    }

    /// The cubic Bezier law with end values and derivatives
    /// (`Set(Pdeb, Valdeb, Ddeb, Pfin, Valfin, Dfin)`); Standard_ConstructionError when
    /// `pdeb` and `pfin` are too close. `law.set_with_derivatives(0.0, 1.0, 0.5, 1.0, 2.0, 0.0)?`.
    pub fn set_with_derivatives(
        &self,
        pdeb: f64,
        valdeb: f64,
        ddeb: f64,
        pfin: f64,
        valfin: f64,
        dfin: f64,
    ) -> Result<(), Failure> {
        let mut poles: Array1<'static, f64> = one_based(4)?;
        let coe: f64 = (pfin - pdeb) / 3.0;
        put(&mut poles, 1, valdeb)?;
        put(&mut poles, 4, valfin)?;
        put(&mut poles, 2, valdeb + coe * ddeb)?;
        put(&mut poles, 3, valfin - coe * dfin)?;
        let (mut knots, mut mults) = (one_based::<f64>(2)?, one_based::<i32>(2)?);
        put(&mut knots, 1, pdeb)?;
        put(&mut knots, 2, pfin)?;
        mults.init(4);
        let curve = LawBSpline::new(&poles, &knots, &mults, 3, false)?;
        self.parent.set_curve(&Handle::new(curve))
    }
}

/// A law interpolating values given at parameters (`Law_Interpol`).
///
/// `let law = LawInterpol::new(); law.set(&par_and_rad, false)?; law.value(0.5)?`.
#[derive(Default)]
pub struct LawInterpol {
    parent: LawBSpFunc,
}
impl_transient!(LawInterpol, "Law_Interpol", LawBSpFunc, parent, interfaces: [dyn LawFunction]);
forward_bsp_func!(LawInterpol);

/// A shared array of reals, as `Law_Interpolate` receives them.
type SharedReals = Handle<HArray1<f64>>;

/// The parameters and values of `Law_Interpol::Set*`: the values drop the last point
/// of a periodic law, and `reparam` maps each abscissa to its parameter.
fn split_points(
    par_and_rad: &Array1<'_, Point2>,
    periodic: bool,
    reparam: impl Fn(f64) -> f64,
) -> Result<(SharedReals, SharedReals), Failure> {
    let (l, nbp) = (par_and_rad.lower(), par_and_rad.len() as i32);
    let mut par: Array1<'static, f64> = one_based(nbp)?;
    let mut rad: Array1<'static, f64> = one_based(if periodic { nbp - 1 } else { nbp })?;
    for i in 1..=nbp {
        let point: Point2 = at(par_and_rad, l + i - 1)?;
        put(&mut par, i, reparam(point.x()))?;
        if !periodic || i != nbp {
            put(&mut rad, i, point.y())?;
        }
    }
    Ok((
        Handle::new(HArray1::from_owned(par)),
        Handle::new(HArray1::from_owned(rad)),
    ))
}

/// The abscissa map of `SetInRelative`: `[wd, wf]` onto `[ud, uf]`.
fn relative(
    par_and_rad: &Array1<'_, Point2>,
    ud: f64,
    uf: f64,
) -> Result<impl Fn(f64) -> f64, Failure> {
    let wd: f64 = at(par_and_rad, par_and_rad.lower())?.x();
    let wf: f64 = at(par_and_rad, par_and_rad.upper())?.x();
    Ok(move |x: f64| (uf * (x - wd) + ud * (wf - x)) / (wf - wd))
}

impl LawInterpol {
    /// A law without curve (`Law_Interpol()`); `LawInterpol::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    fn interpolate(
        &self,
        points: (SharedReals, SharedReals),
        periodic: bool,
        tangents: Option<(f64, f64)>,
    ) -> Result<(), Failure> {
        let (par, rad) = points;
        let mut inter =
            LawInterpolate::with_parameters(&rad, &par, periodic, precision::confusion())?;
        if let Some((dd, df)) = tangents {
            inter.load_ends(dd, df)?;
        }
        inter.perform()?;
        self.parent.set_curve(&inter.curve()?)
    }

    /// Interpolates the `(parameter, value)` points (`Set(ParAndRad, Periodic)`; the
    /// native default of `Periodic` is false); a periodic law ignores the last value.
    /// StdFail_NotDone when the interpolation fails. `law.set(&points, false)?`.
    pub fn set(&self, par_and_rad: &Array1<'_, Point2>, periodic: bool) -> Result<(), Failure> {
        self.interpolate(split_points(par_and_rad, periodic, |x| x)?, periodic, None)
    }

    /// As `set`, with the parameters mapped from the first and last abscissae onto
    /// `[ud, uf]` (`SetInRelative(ParAndRad, Ud, Uf, Periodic)`).
    /// `law.set_in_relative(&points, 0.0, 1.0, false)?`.
    pub fn set_in_relative(
        &self,
        par_and_rad: &Array1<'_, Point2>,
        ud: f64,
        uf: f64,
        periodic: bool,
    ) -> Result<(), Failure> {
        let map = relative(par_and_rad, ud, uf)?;
        self.interpolate(split_points(par_and_rad, periodic, map)?, periodic, None)
    }

    /// As `set`, with end derivatives `dd` and `df` (`Set(ParAndRad, Dd, Df, Periodic)`).
    /// `law.set_with_tangents(&points, 0.0, 0.0, false)?`.
    pub fn set_with_tangents(
        &self,
        par_and_rad: &Array1<'_, Point2>,
        dd: f64,
        df: f64,
        periodic: bool,
    ) -> Result<(), Failure> {
        let points = split_points(par_and_rad, periodic, |x| x)?;
        self.interpolate(points, periodic, Some((dd, df)))
    }

    /// As `set_in_relative`, with end derivatives
    /// (`SetInRelative(ParAndRad, Ud, Uf, Dd, Df, Periodic)`).
    /// `law.set_in_relative_with_tangents(&points, 0.0, 1.0, 0.0, 0.0, false)?`.
    pub fn set_in_relative_with_tangents(
        &self,
        par_and_rad: &Array1<'_, Point2>,
        (ud, uf): (f64, f64),
        dd: f64,
        df: f64,
        periodic: bool,
    ) -> Result<(), Failure> {
        let map = relative(par_and_rad, ud, uf)?;
        self.interpolate(
            split_points(par_and_rad, periodic, map)?,
            periodic,
            Some((dd, df)),
        )
    }
}
