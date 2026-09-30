// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_BezierCurves.hxx/.cxx/.lxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_BezierCurves`: the Bezier surface filled between two, three or four
//! Bezier boundary curves.
use super::curves_filling::{
    CurvedPair, FourBoundaries, SchemeGrids, chained, closing_poles, curved_pair, curved_two,
    fill_scheme, live, ruled_grids, scheme_grids, take_open,
};
use super::{GeomFillFilling, GeomFillFillingStyle};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::at;
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomBezierSurface, GeomBoundedCurve};
use rustycad_tkmath::gp::Point3;

/// The Bezier surface of two to four Bezier boundaries (`GeomFill_BezierCurves`),
/// filled in the `GeomFill_FillingStyle` asked for.
///
/// `let fill = GeomFillBezierCurves::with_four(&c1, &c2, &c3, &c4, GeomFillFillingStyle::CoonsStyle)?; fill.surface()`
#[derive(Clone, Default)]
pub struct GeomFillBezierCurves {
    my_surface: Handle<GeomBezierSurface>,
}

type Curve = Handle<GeomBezierCurve>;

impl GeomFillBezierCurves {
    /// An empty filling with a null surface (`GeomFill_BezierCurves()`);
    /// `GeomFillBezierCurves::new().surface().is_null()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `GeomFill_BezierCurves(C1, C2, C3, C4, Type)`: see `init_four`.
    /// `GeomFillBezierCurves::with_four(&c1, &c2, &c3, &c4, GeomFillFillingStyle::StretchStyle)?`.
    pub fn with_four(
        c1: &Curve,
        c2: &Curve,
        c3: &Curve,
        c4: &Curve,
        style: GeomFillFillingStyle,
    ) -> Result<Self, Failure> {
        let mut filling = Self::new();
        filling.init_four(c1, c2, c3, c4, style)?;
        Ok(filling)
    }
    /// `GeomFill_BezierCurves(C1, C2, C3, Type)`: see `init_three`.
    /// `GeomFillBezierCurves::with_three(&c1, &c2, &c3, GeomFillFillingStyle::CurvedStyle)?`.
    pub fn with_three(
        c1: &Curve,
        c2: &Curve,
        c3: &Curve,
        style: GeomFillFillingStyle,
    ) -> Result<Self, Failure> {
        let mut filling = Self::new();
        filling.init_three(c1, c2, c3, style)?;
        Ok(filling)
    }
    /// `GeomFill_BezierCurves(C1, C2, Type)`: see `init_two`.
    /// `GeomFillBezierCurves::with_two(&c1, &c2, GeomFillFillingStyle::StretchStyle)?`.
    pub fn with_two(c1: &Curve, c2: &Curve, style: GeomFillFillingStyle) -> Result<Self, Failure> {
        let mut filling = Self::new();
        filling.init_two(c1, c2, style)?;
        Ok(filling)
    }
    /// `Init(C1, C2, C3, C4, Type)`: the copies of the curves are chained end to start
    /// within Precision::Confusion (reversing some), Standard_ConstructionError when
    /// they do not join; they are raised to common degrees (at least 3 for the Coons
    /// style), rational weights are made compatible, and the scheme of `style` fills
    /// the poles; `fill.init_four(&c1, &c2, &c3, &c4, GeomFillFillingStyle::CoonsStyle)?`.
    pub fn init_four(
        &mut self,
        c1: &Curve,
        c2: &Curve,
        c3: &Curve,
        c4: &Curve,
        style: GeomFillFillingStyle,
    ) -> Result<(), Failure> {
        let Some(cc) = arrange([c1, c2, c3, c4], precision::confusion())? else {
            return Err(errors::construction(
                " GeomFill_BezierCurves: Courbes non jointives; expected four joined curves",
            ));
        };
        let deg = |k: usize| live(&cc[k]).map(|c| c.degree());
        let mut deg_u: i32 = deg(0)?.max(deg(2)?);
        let mut deg_v: i32 = deg(1)?.max(deg(3)?);
        if style == GeomFillFillingStyle::CoonsStyle {
            (deg_u, deg_v) = (deg_u.max(3), deg_v.max(3));
        }
        for (k, target) in [deg_u, deg_v, deg_u, deg_v].into_iter().enumerate() {
            let curve: &GeomBezierCurve = live(&cc[k])?;
            if curve.degree() < target {
                curve.increase(target)?;
            }
        }
        let caro: GeomFillFilling = self.four_scheme(&cc, style)?;
        self.my_surface = Handle::new(surface_of(&caro)?);
        Ok(())
    }
    /// The scheme of `style` on the arranged curves, with `SetSameWeights` on copies
    /// of their weights when one is rational.
    fn four_scheme(
        &self,
        cc: &[Curve; 4],
        style: GeomFillFillingStyle,
    ) -> Result<GeomFillFilling, Failure> {
        let curves: Vec<&GeomBezierCurve> = cc.iter().map(live).collect::<Result<_, _>>()?;
        let poles: Vec<Array1<'static, Point3>> = curves.iter().map(|c| c.poles()).collect();
        let is_rat: bool = curves.iter().any(|c| c.is_rational());
        let boundaries = FourBoundaries {
            poles: [&poles[0], &poles[1], &poles[2], &poles[3]],
            weights: None,
        };
        if !is_rat {
            return fill_scheme(&boundaries, style);
        }
        let mut weights: Vec<Array1<'static, f64>> = curves
            .iter()
            .map(|c| c.weights_array(|w| w.clone()))
            .collect();
        set_same_weights(&mut weights)?;
        let weighted = FourBoundaries {
            weights: Some([&weights[0], &weights[1], &weights[2], &weights[3]]),
            ..boundaries
        };
        fill_scheme(&weighted, style)
    }
    /// `Init(C1, C2, C3, Type)`: the fourth boundary is the segment joining the ends
    /// of `c1` and `c3` that do not touch `c2` (within the square of
    /// Precision::Confusion); `fill.init_three(&c1, &c2, &c3, GeomFillFillingStyle::CoonsStyle)?`.
    pub fn init_three(
        &mut self,
        c1: &Curve,
        c2: &Curve,
        c3: &Curve,
        style: GeomFillFillingStyle,
    ) -> Result<(), Failure> {
        let poles: Array1<'static, Point3> =
            closing_poles([live(c1)?, live(c2)?, live(c3)?].map(|c| c as &dyn GeomBoundedCurve))?;
        let c4: Curve = Handle::new(GeomBezierCurve::new(&poles)?);
        self.init_four(c1, c2, c3, &c4, style)
    }
    /// `Init(C1, C2, Type)`: the ruled surface between copies of the curves raised
    /// to one degree, or, in the curved style, the translation surface of the first
    /// curve along the second after joining their ends (Standard_OutOfRange when
    /// they do not touch). Quirk kept: when the end of `c1` meets the start of `c2`,
    /// the caller's `c1` is reversed, not its copy;
    /// `fill.init_two(&c1, &c2, GeomFillFillingStyle::StretchStyle)?`.
    pub fn init_two(
        &mut self,
        c1: &Curve,
        c2: &Curve,
        style: GeomFillFillingStyle,
    ) -> Result<(), Failure> {
        let cc1: Curve = Handle::new(GeomBezierCurve::copy_of(live(c1)?)?);
        let cc2: Curve = Handle::new(GeomBezierCurve::copy_of(live(c2)?)?);
        let (a, b) = (live(&cc1)?, live(&cc2)?);
        let is_rat: bool = a.is_rational() || b.is_rational();
        if style != GeomFillFillingStyle::CurvedStyle {
            self.my_surface = Handle::new(ruled(a, b, is_rat)?);
            return Ok(());
        }
        let pair = CurvedPair {
            copies: [a as &dyn GeomBoundedCurve, b],
            caller_first: live(c1)?,
            message: "GeomFill_BezierCurves: Courbes non jointives; expected two touching curves",
        };
        curved_pair(&pair)?;
        let rows = [a.poles(), b.poles()];
        let weights = is_rat.then(|| [a, b].map(|c| c.weights_array(|w| w.clone())));
        let caro = curved_two(&rows, weights.as_ref())?;
        self.my_surface = Handle::new(surface_of(&caro)?);
        Ok(())
    }
    /// `Surface()`: the surface of the last `Init`, null before; `fill.surface()`.
    pub fn surface(&self) -> Handle<GeomBezierSurface> {
        self.my_surface.clone()
    }
}

/// The ruled Bezier surface of `Init(C1, C2)` outside the curved style: the copies
/// raised to the higher degree, their poles and weights as the two columns.
fn ruled(
    a: &GeomBezierCurve,
    b: &GeomBezierCurve,
    is_rat: bool,
) -> Result<GeomBezierSurface, Failure> {
    let deg_u: i32 = a.degree().max(b.degree());
    for curve in [a, b] {
        if curve.degree() < deg_u {
            curve.increase(deg_u)?;
        }
    }
    let (p1, p2) = (a.poles(), b.poles());
    let weights = is_rat.then(|| [a, b].map(|c| c.weights_array(|w| w.clone())));
    let grids = ruled_grids(
        [&p1, &p2],
        weights.as_ref().map(|[w1, w2]| [w1, w2]),
        deg_u + 1,
    )?;
    bezier_surface(grids)
}

/// The Bezier surface of a filling scheme's poles (and weights when rational).
fn surface_of(caro: &GeomFillFilling) -> Result<GeomBezierSurface, Failure> {
    bezier_surface(scheme_grids(caro)?)
}

/// `new Geom_BezierSurface(Poles[, Weights])`.
fn bezier_surface((poles, weights): SchemeGrids) -> Result<GeomBezierSurface, Failure> {
    match weights {
        Some(weights) => GeomBezierSurface::with_weights(&poles, &weights),
        None => GeomBezierSurface::new(&poles),
    }
}

/// `SetSameWeights`: rescales the weight rows `W2`, `W3`, `W4` so that the corner
/// weights agree, and, when the two corner cross-ratios differ by more than
/// Precision::Confusion, spreads a geometric factor over `W4`.
fn set_same_weights(w: &mut [Array1<'static, f64>]) -> Result<(), Failure> {
    let eps: f64 = precision::confusion();
    let (nu, nv) = (w[0].len() as i32, w[1].len() as i32);
    let a: f64 = (at(&w[0], 1)? * at(&w[1], 1)?) / (at(&w[0], nu)? * at(&w[1], nv)?);
    let b: f64 = (at(&w[2], 1)? * at(&w[3], 1)?) / (at(&w[2], nu)? * at(&w[3], nv)?);
    let alfa: f64 = at(&w[0], nu)? / at(&w[1], 1)?;
    scale(&mut w[1], alfa);
    let beta: f64 = at(&w[1], nv)? / at(&w[2], nu)?;
    scale(&mut w[2], beta);
    let gamma: f64 = at(&w[2], 1)? / at(&w[3], nv)?;
    scale(&mut w[3], gamma);
    if (a - b).abs() > eps {
        let step: f64 = (at(&w[0], 1)? / at(&w[3], 1)?).powf(1. / f64::from(nv - 1));
        let mut x: f64 = step;
        for i in (1..nv).rev() {
            w[3][i] *= x;
            x *= step;
        }
    }
    Ok(())
}

fn scale(row: &mut Array1<'static, f64>, factor: f64) {
    row.iter_mut().for_each(|w| *w *= factor);
}

/// The static `Arrange`: copies of the four curves chained end to start (a curve
/// that joins by its end is reversed), the last two reversed; `None` when a curve
/// joins no free end.
fn arrange(curves: [&Curve; 4], tol: f64) -> Result<Option<[Curve; 4]>, Failure> {
    let mut gc: Vec<Curve> = Vec::with_capacity(4);
    for curve in curves {
        gc.push(Handle::new(GeomBezierCurve::copy_of(live(curve)?)?));
    }
    for i in 1..=3 {
        let end: Point3 = live(&gc[i - 1])?.end_point()?;
        if !take_open(&mut gc, i, end, tol)? {
            return Ok(None);
        }
    }
    chained(&gc).map(Some)
}
