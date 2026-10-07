// Rust translation of Draw/TKDraw/DrawTrSurf/DrawTrSurf_BezierCurve.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DrawTrSurf_BezierCurve`: a drawable 3D Bezier curve with its control polygon.
use super::curve::DrawTrSurfCurve;
use super::geometry::geometry;
use super::params::current;
use super::spline_common::{PolesStyle, draw_polygon, find_index, scaled_target};
use super::style::CurveLook;
use crate::draw::{
    DrawColor, DrawColorKind, DrawDisplay, DrawIStream, DrawInterpretor, DrawOStream, DrawSession,
    Drawable3D, Drawable3DBase, DrawableHandle, kind_of,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCurve, GeomGeometry};
use rustycad_tkgeombase::geom_tools::CurveSet;
use std::any::{Any, TypeId};
use std::rc::Rc;

/// `DrawTrSurf_BezierCurve`: a [`DrawTrSurfCurve`] on a `Geom_BezierCurve` that also
/// draws its poles.
pub struct DrawTrSurfBezierCurve {
    curve: DrawTrSurfCurve,
    poles: PolesStyle,
}

impl DrawTrSurfBezierCurve {
    /// `DrawTrSurf_BezierCurve(C)`: green, 16 points, deflection 0.05, uniform
    /// deflection, red poles shown.
    pub fn new(curve: Handle<GeomBezierCurve>) -> Self {
        let look = CurveLook::new(DrawColor::new(DrawColorKind::Vert));
        Self {
            curve: DrawTrSurfCurve::with_look(curve.cast::<dyn GeomCurve>(), look, (16, 0.05, 1)),
            poles: PolesStyle::default_shown(),
        }
    }

    /// `DrawTrSurf_BezierCurve(C, CurvColor, PolesColor, ShowPoles, Discret,
    /// Deflection, DrawMode)`; `discretisation` is `(Discret, Deflection, DrawMode)`.
    pub fn with_style(
        curve: Handle<GeomBezierCurve>,
        (curve_color, poles_color): (DrawColor, DrawColor),
        show_poles: bool,
        discretisation: (i32, f64, i32),
    ) -> Self {
        let look = CurveLook::new(curve_color);
        Self {
            curve: DrawTrSurfCurve::with_look(curve.cast::<dyn GeomCurve>(), look, discretisation),
            poles: PolesStyle::new(show_poles, poles_color),
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

    /// The Bezier curve (`occ::down_cast<Geom_BezierCurve>(curv)`).
    fn bezier(&self) -> Handle<GeomBezierCurve> {
        self.curve.curve().cast::<GeomBezierCurve>()
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
        let handle = self.bezier();
        let bezier = geometry(&handle)?;
        let target = scaled_target(display, (x, y), precision);
        find_index(display, target, (index, bezier.nb_poles()), |i| {
            bezier.pole(i)
        })
    }

    /// `RegisterFactory` (the `Draw_Drawable3D_FACTORY` macro).
    pub fn register_factory(session: &DrawSession) {
        session.register_factory("DrawTrSurf_BezierCurve", Self::restore);
    }

    /// `Restore`: a curve read in the compact GeomTools format (no curve when it is
    /// not a Bezier curve), with the settings of `DrawTrSurf::Parameters()`.
    pub fn restore(
        session: &DrawSession,
        stream: &mut DrawIStream,
    ) -> Result<Option<DrawableHandle>, Failure> {
        let params = current(session);
        let curve = stream
            .read_through(CurveSet::read_curve)
            .cast::<GeomBezierCurve>();
        let colors = (params.curv_color, params.poles_color);
        let discretisation = (params.discret, params.deflection, params.draw_mode);
        let restored = Self::with_style(curve, colors, params.is_show_poles, discretisation);
        Ok(Some(Rc::new(restored)))
    }
}

impl Drawable3D for DrawTrSurfBezierCurve {
    fn base(&self) -> &Drawable3DBase {
        self.curve.base()
    }

    fn type_name(&self) -> &'static str {
        "DrawTrSurf_BezierCurve"
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
        self.curve.draw_curve(display)
    }

    /// `Copy`: a copy of the curve with the same look and discretisation.
    fn copy(&self, _this: &DrawableHandle) -> Result<DrawableHandle, Failure> {
        let handle = self.bezier();
        let copied = geometry(&handle)?.copy()?.cast::<GeomBezierCurve>();
        let d = self.curve.drawable();
        let colors = (self.curve.style().color(), self.poles.color());
        let discretisation = (d.discretisation(), d.deflection(), d.draw_mode());
        let copy = Self::with_style(copied, colors, self.poles.shown(), discretisation);
        Ok(Rc::new(copy))
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
