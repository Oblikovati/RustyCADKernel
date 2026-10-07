// Rust translation of Draw/TKDraw/DrawTrSurf/DrawTrSurf_BSplineCurve2d.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DrawTrSurf_BSplineCurve2d`: a drawable 2D BSpline curve with its control polygon
//! and knots.
use super::curve2d::DrawTrSurfCurve2d;
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
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve, Geom2dGeometry};
use rustycad_tkgeombase::geom_tools::Curve2dSet;
use rustycad_tkmath::gp::Point2;
use std::any::{Any, TypeId};
use std::rc::Rc;

/// `DrawTrSurf_BSplineCurve2d`: a [`DrawTrSurfCurve2d`] on a `Geom2d_BSplineCurve`
/// that also draws its poles and marks its knots.
pub struct DrawTrSurfBSplineCurve2d {
    curve: DrawTrSurfCurve2d,
    poles: PolesStyle,
    knots: KnotsStyle,
}

impl DrawTrSurfBSplineCurve2d {
    /// `DrawTrSurf_BSplineCurve2d(C)`: green, 100 points, red poles and violet diamond
    /// knots shown.
    pub fn new(curve: Handle<Geom2dBSplineCurve>) -> Self {
        let look = CurveLook::new(DrawColor::new(DrawColorKind::Vert));
        Self {
            curve: DrawTrSurfCurve2d::with_look(curve.cast::<dyn Geom2dCurve>(), look, 100),
            poles: PolesStyle::default_shown(),
            knots: KnotsStyle::default_shown(),
        }
    }

    /// `DrawTrSurf_BSplineCurve2d(C, CurvColor, PolesColor, KnotsColor, KnotsShape,
    /// KnotsSize, ShowPoles, ShowKnots, Discret)`.
    pub fn with_style(curve: Handle<Geom2dBSplineCurve>, look: &SplineLook, discret: i32) -> Self {
        let curve_look = CurveLook::new(look.curve_color);
        Self {
            curve: DrawTrSurfCurve2d::with_look(
                curve.cast::<dyn Geom2dCurve>(),
                curve_look,
                discret,
            ),
            poles: look.poles(),
            knots: look.knots(),
        }
    }

    /// The `DrawTrSurf_Curve2d` part.
    pub fn curve_part(&self) -> &DrawTrSurfCurve2d {
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

    /// The BSpline curve (`occ::down_cast<Geom2d_BSplineCurve>(curv)`).
    fn bspline(&self) -> Handle<Geom2dBSplineCurve> {
        self.curve.curve().cast::<Geom2dBSplineCurve>()
    }

    /// `FindPole`: the first pole after `index` whose projection (as `(x, y, 0)`) is
    /// within `precision` pixels of the pick point `(x, y)`; 0 when none is.
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
        session.register_factory("DrawTrSurf_BSplineCurve2d", Self::restore);
    }

    /// `Restore`: a curve read in the compact GeomTools format (no curve when it is
    /// not a BSpline curve), with the settings of `DrawTrSurf::Parameters()`.
    pub fn restore(session: &DrawSession, stream: &mut DrawIStream) -> Option<DrawableHandle> {
        let params = current(session);
        let curve = stream
            .read_through(Curve2dSet::read_curve2d)
            .cast::<Geom2dBSplineCurve>();
        let look = SplineLook::from_params(&params);
        Some(Rc::new(Self::with_style(curve, &look, params.discret)))
    }
}

impl Drawable3D for DrawTrSurfBSplineCurve2d {
    fn base(&self) -> &Drawable3DBase {
        self.curve.base()
    }

    fn type_name(&self) -> &'static str {
        "DrawTrSurf_BSplineCurve2d"
    }

    fn ancestor(&self, type_id: TypeId) -> Option<&dyn Any> {
        kind_of(&self.curve, type_id)
    }

    /// `DrawOn`: the control polygon (closed on a periodic curve), the curve, then
    /// every knot.
    fn draw_on(&self, display: &mut DrawDisplay<'_>) -> Result<(), Failure> {
        let handle = self.bspline();
        let bspline = geometry(&handle)?;
        if self.poles.shown() {
            draw_polygon(display, self.poles.color(), bspline.poles().as_slice());
            if bspline.is_periodic() {
                display.draw_to(bspline.pole(1)?);
            }
        }
        self.curve.draw_on(display)?;
        if self.knots.shown() {
            let knots = bspline.knots();
            self.knots
                .mark(display, knots.iter().map(|u| bspline.value(*u)))?;
        }
        Ok(())
    }

    /// `Copy`: a copy of the curve with the same look and number of points.
    fn copy(&self, _this: &DrawableHandle) -> Result<DrawableHandle, Failure> {
        let handle = self.bspline();
        let copied = geometry(&handle)?.copy()?.cast::<Geom2dBSplineCurve>();
        let look = SplineLook::of(self.curve.style().color(), &self.poles, &self.knots);
        let discret = self.curve.drawable().discretisation();
        Ok(Rc::new(Self::with_style(copied, &look, discret)))
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

    fn is_3d(&self) -> bool {
        false
    }
}
