// Rust translation of Draw/TKDraw/DrawTrSurf/DrawTrSurf_BezierCurve2d.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DrawTrSurf_BezierCurve2d`: a drawable 2D Bezier curve with its control polygon.
use super::curve2d::DrawTrSurfCurve2d;
use super::geometry::geometry;
use super::params::current;
use super::spline_common::{PolesStyle, draw_polygon, find_index, scaled_target};
use super::style::CurveLook;
use crate::draw::{
    DrawColor, DrawColorKind, DrawDisplay, DrawIStream, DrawInterpretor, DrawOStream, DrawSession,
    Drawable3D, Drawable3DBase, DrawableHandle, kind_of,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve, Geom2dGeometry};
use rustycad_tkgeombase::geom_tools::Curve2dSet;
use std::any::{Any, TypeId};
use std::rc::Rc;

/// `DrawTrSurf_BezierCurve2d`: a [`DrawTrSurfCurve2d`] on a `Geom2d_BezierCurve` that
/// also draws its poles.
pub struct DrawTrSurfBezierCurve2d {
    curve: DrawTrSurfCurve2d,
    poles: PolesStyle,
}

impl DrawTrSurfBezierCurve2d {
    /// `DrawTrSurf_BezierCurve2d(C)`: green, 50 points, red poles shown.
    pub fn new(curve: Handle<Geom2dBezierCurve>) -> Self {
        Self::with_style(
            curve,
            (
                DrawColor::new(DrawColorKind::Vert),
                DrawColor::new(DrawColorKind::Rouge),
            ),
            true,
            50,
        )
    }

    /// `DrawTrSurf_BezierCurve2d(C, CurvColor, PolesColor, ShowPoles, Discret)`.
    pub fn with_style(
        curve: Handle<Geom2dBezierCurve>,
        (curve_color, poles_color): (DrawColor, DrawColor),
        show_poles: bool,
        discret: i32,
    ) -> Self {
        let look = CurveLook::new(curve_color);
        Self {
            curve: DrawTrSurfCurve2d::with_look(curve.cast::<dyn Geom2dCurve>(), look, discret),
            poles: PolesStyle::new(show_poles, poles_color),
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

    /// The Bezier curve (`occ::down_cast<Geom2d_BezierCurve>(curv)`).
    fn bezier(&self) -> Handle<Geom2dBezierCurve> {
        self.curve.curve().cast::<Geom2dBezierCurve>()
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
        let handle = self.bezier();
        let bezier = geometry(&handle)?;
        let target = scaled_target(display, (x, y), precision);
        find_index(display, target, (index, bezier.nb_poles()), |i| {
            bezier.pole(i)
        })
    }

    /// `RegisterFactory` (the `Draw_Drawable3D_FACTORY` macro).
    pub fn register_factory(session: &DrawSession) {
        session.register_factory("DrawTrSurf_BezierCurve2d", Self::restore);
    }

    /// `Restore`: a curve read in the compact GeomTools format (no curve when it is
    /// not a Bezier curve), with the settings of `DrawTrSurf::Parameters()`.
    pub fn restore(
        session: &DrawSession,
        stream: &mut DrawIStream,
    ) -> Result<Option<DrawableHandle>, Failure> {
        let params = current(session);
        let curve = stream
            .read_through(Curve2dSet::read_curve2d)
            .cast::<Geom2dBezierCurve>();
        let colors = (params.curv_color, params.poles_color);
        let restored = Self::with_style(curve, colors, params.is_show_poles, params.discret);
        Ok(Some(Rc::new(restored)))
    }
}

impl Drawable3D for DrawTrSurfBezierCurve2d {
    fn base(&self) -> &Drawable3DBase {
        self.curve.base()
    }

    fn type_name(&self) -> &'static str {
        "DrawTrSurf_BezierCurve2d"
    }

    fn ancestor(&self, type_id: TypeId) -> Option<&dyn Any> {
        kind_of(&self.curve, type_id)
    }

    /// `DrawOn`: the control polygon when shown, then the curve.
    fn draw_on(&self, display: &mut DrawDisplay<'_>) -> Result<(), Failure> {
        if self.poles.shown() {
            let handle = self.bezier();
            let poles = geometry(&handle)?.poles();
            draw_polygon(display, self.poles.color(), poles.as_slice());
        }
        self.curve.draw_on(display)
    }

    /// `Copy`: a copy of the curve with the same look and number of points.
    fn copy(&self, _this: &DrawableHandle) -> Result<DrawableHandle, Failure> {
        let handle = self.bezier();
        let copied = geometry(&handle)?.copy()?.cast::<Geom2dBezierCurve>();
        let colors = (self.curve.style().color(), self.poles.color());
        let discret = self.curve.drawable().discretisation();
        Ok(Rc::new(Self::with_style(
            copied,
            colors,
            self.poles.shown(),
            discret,
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

    fn is_3d(&self) -> bool {
        false
    }
}
