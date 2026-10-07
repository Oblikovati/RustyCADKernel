// Rust translation of Draw/TKDraw/DrawTrSurf/DrawTrSurf_BSplineCurve.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DrawTrSurf_BSplineCurve`: a drawable 3D BSpline curve with its control polygon
//! and knots.
use super::curve::DrawTrSurfCurve;
use super::geometry::geometry;
use super::params::current;
use super::spline_common::{
    KnotsStyle, PolesStyle, SplineLook, draw_polygon, find_index, scaled_target,
};
use super::style::CurveLook;
use crate::draw::{
    DrawColor, DrawColorKind, DrawDisplay, DrawIStream, DrawInterpretor, DrawOStream, DrawSession,
    Drawable3D, Drawable3DBase, DrawableHandle, kind_of,
};
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve, GeomGeometry};
use rustycad_tkgeombase::geom_tools::CurveSet;
use rustycad_tkmath::gp::Point2;
use std::any::{Any, TypeId};
use std::rc::Rc;

/// The minimum number of points of each span of the ranged drawing.
const MIN_SPAN_POINTS: i32 = 30;

/// `DrawTrSurf_BSplineCurve`: a [`DrawTrSurfCurve`] on a `Geom_BSplineCurve` that also
/// draws its poles and marks its knots.
pub struct DrawTrSurfBSplineCurve {
    curve: DrawTrSurfCurve,
    poles: PolesStyle,
    knots: KnotsStyle,
}

impl DrawTrSurfBSplineCurve {
    /// `DrawTrSurf_BSplineCurve(C)`: green, 16 points, deflection 0.05, uniform
    /// deflection, red poles and violet diamond knots shown.
    pub fn new(curve: Handle<GeomBSplineCurve>) -> Self {
        let look = CurveLook::new(DrawColor::new(DrawColorKind::Vert));
        Self {
            curve: DrawTrSurfCurve::with_look(curve.cast::<dyn GeomCurve>(), look, (16, 0.05, 1)),
            poles: PolesStyle::default_shown(),
            knots: KnotsStyle::default_shown(),
        }
    }

    /// `DrawTrSurf_BSplineCurve(C, CurvColor, PolesColor, KnotsColor, KnotsShape,
    /// KnotsSize, ShowPoles, ShowKnots, Discret, Deflection, DrawMode)`.
    pub fn with_style(
        curve: Handle<GeomBSplineCurve>,
        look: &SplineLook,
        discretisation: (i32, f64, i32),
    ) -> Self {
        let curve_look = CurveLook::new(look.curve_color);
        Self {
            curve: DrawTrSurfCurve::with_look(
                curve.cast::<dyn GeomCurve>(),
                curve_look,
                discretisation,
            ),
            poles: look.poles(),
            knots: look.knots(),
        }
    }

    /// The `DrawTrSurf_Curve` part.
    pub fn curve_part(&self) -> &DrawTrSurfCurve {
        &self.curve
    }

    /// `ShowPoles`, `ClearPoles`, `SetPolesColor`, `PolesColor`.
    pub fn poles(&self) -> &PolesStyle {
        &self.poles
    }

    /// `ShowKnots`, `ClearKnots`, `SetKnotsColor`, `KnotsColor`, `SetKnotsShape`,
    /// `KnotsShape`.
    pub fn knots(&self) -> &KnotsStyle {
        &self.knots
    }

    /// The BSpline curve (`occ::down_cast<Geom_BSplineCurve>(curv)`).
    fn bspline(&self) -> Handle<GeomBSplineCurve> {
        self.curve.curve().cast::<GeomBSplineCurve>()
    }

    /// `DrawOn(dis, ShowPoles, ShowKnots)`: as `DrawOn`, with the poles and knots also
    /// switched by the arguments; the polygon is not closed on a periodic curve and
    /// every knot is marked.
    pub fn draw_on_with(
        &self,
        display: &mut DrawDisplay<'_>,
        show_poles: bool,
        show_knots: bool,
    ) -> Result<(), Failure> {
        let handle = self.bspline();
        let bspline = geometry(&handle)?;
        if self.poles.shown() && show_poles {
            draw_polygon(display, self.poles.color(), bspline.poles().as_slice());
        }
        self.curve.draw_curve(display)?;
        if self.knots.shown() && show_knots {
            let knots = bspline.knots();
            self.knots
                .mark(display, knots.iter().map(|u| bspline.value(*u)))?;
        }
        Ok(())
    }

    /// `DrawOn(dis, U1, U2, Pindex, ShowPoles, ShowKnots)`: the curve between `u1` and
    /// `u2` only, the polygon around pole `pole_index` (all poles for 0), and the
    /// knots inside the range.
    pub fn draw_on_range(
        &self,
        display: &mut DrawDisplay<'_>,
        (u1, u2): (f64, f64),
        pole_index: i32,
        (show_poles, show_knots): (bool, bool),
    ) -> Result<(), Failure> {
        let handle = self.bspline();
        let bspline = geometry(&handle)?;
        let (i1, j1) = bspline.locate_u(u1, epsilon(u1).abs(), false)?;
        let (i2, j2) = bspline.locate_u(u2, epsilon(u2).abs(), false)?;
        if self.poles.shown() && show_poles {
            display.set_color(self.poles.color());
            draw_pole_neighbourhood(display, bspline, pole_index)?;
        }
        display.set_color(self.curve.style().color());
        if bspline.degree() == 1 {
            display.move_to_3d(bspline.value(u1)?);
            display.draw_to_3d(bspline.value(u2)?);
        } else {
            let span = RangeSpans {
                bspline,
                discret: self.curve.drawable().discretisation(),
            };
            span.draw(display, (u1, u2), (i1, j2))?;
        }
        if self.knots.shown() && show_knots {
            let marked = (j1..=i2).map(|i| bspline.value(bspline.knot(i)?));
            self.knots.mark(display, marked)?;
        }
        Ok(())
    }

    /// `FindPole`: the first pole after `index` whose projection is within `precision`
    /// pixels of the pick point `(x, y)`; 0 when none is.
    pub fn find_pole(
        &self,
        (x, y): (f64, f64),
        display: &DrawDisplay<'_>,
        precision: f64,
        index: i32,
    ) -> Result<i32, Failure> {
        let handle = self.bspline();
        let bspline = geometry(&handle)?;
        let target = scaled_target(display, (x, y), precision);
        find_index(display, target, (index, bspline.nb_poles()), |i| {
            bspline.pole(i)
        })
    }

    /// `FindKnot`: the first knot after `index` whose curve point projects within
    /// `precision` of `(x, y)` (both in model units, unlike `FindPole`); 0 when none.
    pub fn find_knot(
        &self,
        (x, y): (f64, f64),
        display: &DrawDisplay<'_>,
        precision: f64,
        index: i32,
    ) -> Result<i32, Failure> {
        let handle = self.bspline();
        let bspline = geometry(&handle)?;
        let target = (Point2::new(x, y), precision);
        find_index(display, target, (index, bspline.nb_knots()), |i| {
            bspline.value(bspline.knot(i)?)
        })
    }

    /// `RegisterFactory` (the `Draw_Drawable3D_FACTORY` macro).
    pub fn register_factory(session: &DrawSession) {
        session.register_factory("DrawTrSurf_BSplineCurve", Self::restore);
    }

    /// `Restore`: a curve read in the compact GeomTools format (no curve when it is
    /// not a BSpline curve), with the settings of `DrawTrSurf::Parameters()`.
    pub fn restore(session: &DrawSession, stream: &mut DrawIStream) -> Option<DrawableHandle> {
        let params = current(session);
        let curve = stream
            .read_through(CurveSet::read_curve)
            .cast::<GeomBSplineCurve>();
        let discretisation = (params.discret, params.deflection, params.draw_mode);
        let look = SplineLook::from_params(&params);
        Some(Rc::new(Self::with_style(curve, &look, discretisation)))
    }

    /// The current settings as constructor arguments.
    fn look(&self) -> SplineLook {
        SplineLook::of(self.curve.style().color(), &self.poles, &self.knots)
    }
}

/// The control polygon around pole `index`: every pole for 0, the first or last leg
/// at the ends, the two legs through the pole otherwise.
fn draw_pole_neighbourhood(
    display: &mut DrawDisplay<'_>,
    bspline: &GeomBSplineCurve,
    index: i32,
) -> Result<(), Failure> {
    let count = bspline.nb_poles();
    let legs = match index {
        0 => 1..=count,
        1 => 1..=2,
        i if i == count => count - 1..=count,
        i => i - 1..=i + 1,
    };
    let mut first = true;
    for i in legs {
        let pole = bspline.pole(i)?;
        if first {
            display.move_to_3d(pole);
            first = false;
        } else {
            display.draw_to_3d(pole);
        }
    }
    Ok(())
}

/// The curve part of the ranged drawing: the parts before the first and after the
/// last knot of the range, then the knot spans, each with a number of points
/// proportional to its length (at least [`MIN_SPAN_POINTS`]).
struct RangeSpans<'a> {
    bspline: &'a GeomBSplineCurve,
    discret: i32,
}

impl RangeSpans<'_> {
    /// Draws `[u1, u2]`; `(i1, j2)` are the `LocateU` indices of `u1` and `u2`.
    fn draw(
        &self,
        display: &mut DrawDisplay<'_>,
        (u1, u2): (f64, f64),
        (i1, j2): (i32, i32),
    ) -> Result<(), Failure> {
        let (mut ka, mut kb) = (
            self.bspline.first_u_knot_index()?,
            self.bspline.last_u_knot_index()?,
        );
        let total = (self.bspline.knot(ka)?, self.bspline.knot(kb)?);
        let uk1 = if i1 > ka {
            ka = i1;
            u1
        } else {
            self.plot(display, (u1, total.0), total)?;
            total.0
        };
        let uk2 = if j2 < kb {
            kb = j2;
            u2
        } else {
            self.plot(display, (total.1, u2), total)?;
            total.1
        };
        for k in ka..kb {
            let ua = if k == ka { uk1 } else { self.bspline.knot(k)? };
            let ub = if k == ka {
                self.bspline.knot(k + 1)?
            } else if k == kb - 1 {
                uk2
            } else {
                self.bspline.knot(k + 1)?
            };
            self.plot(display, (ua, ub), total)?;
        }
        Ok(())
    }

    /// One span `[ua, ub]`: `|Discret * (ua - ub) / (Ustart - Uend)|` points (at least
    /// 30), a move to `ua`, `NbPoints - 2` steps, then a line to `ub`.
    fn plot(
        &self,
        display: &mut DrawDisplay<'_>,
        (ua, ub): (f64, f64),
        (u_start, u_end): (f64, f64),
    ) -> Result<(), Failure> {
        let ratio = f64::from(self.discret) * (ua - ub) / (u_start - u_end);
        let nb_points = (ratio.abs() as i32).max(MIN_SPAN_POINTS);
        let du = (ub - ua) / f64::from(nb_points);
        let mut u = ua;
        display.move_to_3d(self.bspline.value(u)?);
        for _ in 1..=nb_points - 2 {
            u += du;
            display.draw_to_3d(self.bspline.value(u)?);
        }
        display.draw_to_3d(self.bspline.value(ub)?);
        Ok(())
    }
}

impl Drawable3D for DrawTrSurfBSplineCurve {
    fn base(&self) -> &Drawable3DBase {
        self.curve.base()
    }

    fn type_name(&self) -> &'static str {
        "DrawTrSurf_BSplineCurve"
    }

    fn ancestor(&self, type_id: TypeId) -> Option<&dyn Any> {
        kind_of(&self.curve, type_id)
    }

    /// `DrawOn`: the control polygon (closed on a periodic curve), the curve, then the
    /// knots from the first to the last used knot.
    fn draw_on(&self, display: &mut DrawDisplay<'_>) -> Result<(), Failure> {
        let handle = self.bspline();
        let bspline = geometry(&handle)?;
        if self.poles.shown() {
            let poles = bspline.poles();
            draw_polygon(display, self.poles.color(), poles.as_slice());
            if bspline.is_periodic() {
                display.draw_to_3d(bspline.pole(1)?);
            }
        }
        self.curve.draw_curve(display)?;
        if self.knots.shown() {
            let first = bspline.first_u_knot_index()?;
            let last = bspline.last_u_knot_index()?;
            let marked = (first..=last).map(|i| bspline.value(bspline.knot(i)?));
            self.knots.mark(display, marked)?;
        }
        Ok(())
    }

    /// `Copy`: a copy of the curve with the same look and discretisation.
    fn copy(&self, _this: &DrawableHandle) -> Result<DrawableHandle, Failure> {
        let handle = self.bspline();
        let copied = geometry(&handle)?.copy()?.cast::<GeomBSplineCurve>();
        let d = self.curve.drawable();
        let discretisation = (d.discretisation(), d.deflection(), d.draw_mode());
        Ok(Rc::new(Self::with_style(
            copied,
            &self.look(),
            discretisation,
        )))
    }

    fn dump(&self, stream: &mut DrawOStream) -> Result<(), Failure> {
        self.curve.dump(stream)
    }

    fn save(&self, stream: &mut DrawOStream) -> Result<(), Failure> {
        self.curve.save(stream)
    }

    fn whatis(&self, di: &DrawInterpretor) -> Result<(), Failure> {
        self.curve.whatis(di)
    }
}
