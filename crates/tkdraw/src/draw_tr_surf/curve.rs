// Rust translation of Draw/TKDraw/DrawTrSurf/DrawTrSurf_Curve.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DrawTrSurf_Curve`: a drawable 3D curve.
use super::curve_common::{CurvatureDisplay, draw_orientation_arrow, finite_range};
use super::drawable::{DrawTrSurfDrawable, interval_bounds};
use super::geometry::geometry;
use super::params::current;
use super::style::{CurveLook, CurveStyle};
use crate::draw::{
    DrawColor, DrawColorKind, DrawDisplay, DrawIStream, DrawInterpretor, DrawOStream, DrawSession,
    Drawable3D, Drawable3DBase, DrawableHandle,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeombase::geom_lprop::CLProps;
use rustycad_tkgeombase::geom_tools::CurveSet;
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Point3, Vector2};
use std::any::{Any, TypeId};
use std::rc::Rc;

/// `DrawTrSurf_Curve`: a `Geom_Curve` with its color, orientation arrow and optional
/// curvature radii.
///
/// ```
/// use rustycad_tkdraw::draw_tr_surf::DrawTrSurfCurve;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
/// use rustycad_tkmath::gp::{Axis1, Direction3, Point3};
/// let axis = Axis1::new(Point3::new(0.0, 0.0, 0.0), Direction3::new(1.0, 0.0, 0.0).unwrap());
/// let line = GeomLine::new(axis);
/// let curve = DrawTrSurfCurve::new(Handle::new(line).cast::<dyn GeomCurve>(), true);
/// assert_eq!(curve.drawable().discretisation(), 16);
/// ```
pub struct DrawTrSurfCurve {
    drawable: DrawTrSurfDrawable,
    curve: Handle<dyn GeomCurve>,
    style: CurveStyle,
}

impl DrawTrSurfCurve {
    /// `DrawTrSurf_Curve(C, DispOrigin)`: green, 16 points per interval, deflection
    /// 0.01, uniform-deflection mode.
    pub fn new(curve: Handle<dyn GeomCurve>, display_origin: bool) -> Self {
        let look = CurveLook {
            display_origin,
            ..CurveLook::new(DrawColor::new(DrawColorKind::Vert))
        };
        Self::with_look(curve, look, (16, 0.01, 1))
    }

    /// `DrawTrSurf_Curve(C, aColor, Discret, Deflection, DrawMode, DispOrigin,
    /// DispCurvRadius, RadiusMax, RatioOfRadius)`; `discretisation` is
    /// `(Discret, Deflection, DrawMode)`.
    pub fn with_look(
        curve: Handle<dyn GeomCurve>,
        look: CurveLook,
        (discret, deflection, draw_mode): (i32, f64, i32),
    ) -> Self {
        Self {
            drawable: DrawTrSurfDrawable::new(discret, deflection, draw_mode),
            curve,
            style: CurveStyle::new(look),
        }
    }

    /// The `DrawTrSurf_Drawable` part.
    pub fn drawable(&self) -> &DrawTrSurfDrawable {
        &self.drawable
    }

    /// The color, orientation arrow and curvature settings.
    pub fn style(&self) -> &CurveStyle {
        &self.style
    }

    /// `GetCurve`.
    pub fn curve(&self) -> Handle<dyn GeomCurve> {
        self.curve.clone()
    }

    /// `RegisterFactory` (the `Draw_Drawable3D_FACTORY` macro).
    pub fn register_factory(session: &DrawSession) {
        session.register_factory("DrawTrSurf_Curve", Self::restore);
    }

    /// `Restore`: a curve read in the compact GeomTools format, with the colors and
    /// discretisation of `DrawTrSurf::Parameters()`. A record that cannot be read
    /// gives a drawable without curve, as upstream (drawing it fails).
    pub fn restore(session: &DrawSession, stream: &mut DrawIStream) -> Option<DrawableHandle> {
        let params = current(session);
        let curve = stream.read_through(CurveSet::read_curve);
        let look = CurveLook::new(params.curv_color);
        let discretisation = (params.discret, params.deflection, params.draw_mode);
        Some(Rc::new(Self::with_look(curve, look, discretisation)))
    }

    /// The curve drawing of `DrawOn`, shared with the Bezier and BSpline curves.
    pub(crate) fn draw_curve(&self, display: &mut DrawDisplay<'_>) -> Result<(), Failure> {
        let curve = geometry(&self.curve)?;
        let range = (curve.first_parameter(), curve.last_parameter());
        let (first, last) = finite_range::<Point3>(range, |t| curve.d0(t))?;
        display.set_color(self.style.color());
        let adaptor = GeomAdaptorCurve::with_range(&self.curve, first, last)?;
        self.drawable.draw_curve_on(&adaptor, display)?;
        if self.style.display_origin() {
            draw_origin(&adaptor, last, display)?;
        }
        if self.style.curvature_shown() && adaptor.get_type()? != CurveType::Line {
            self.draw_curvature(&adaptor, display)?;
        }
        Ok(())
    }

    /// The curvature radii of a non-linear curve.
    fn draw_curvature(
        &self,
        adaptor: &GeomAdaptorCurve,
        display: &mut DrawDisplay<'_>,
    ) -> Result<(), Failure> {
        let bounds = interval_bounds(adaptor.nb_intervals(Shape::CN)?, |b| {
            adaptor.intervals(b, Shape::CN)
        })?;
        let radii = CurvatureDisplay::of(self.drawable.discretisation(), &self.style);
        let mut props = CLProps::new(self.curve.clone(), 2, radii.resolution())?;
        let curve = geometry(&self.curve)?;
        radii.draw(display, &bounds, &mut props, |t| curve.d0(t))
    }
}

/// The orientation arrow at the end of the drawn range, from the projection of the
/// end point and of the end point moved by the first derivative.
fn draw_origin(
    adaptor: &GeomAdaptorCurve,
    last: f64,
    display: &mut DrawDisplay<'_>,
) -> Result<(), Failure> {
    let (point, vector) = adaptor.d1(last)?;
    let p1 = display.project(point);
    let p2 = display.project(point.translated(vector));
    draw_orientation_arrow(display, p1, Vector2::between(p1, p2))
}

impl Drawable3D for DrawTrSurfCurve {
    fn base(&self) -> &Drawable3DBase {
        self.drawable.base()
    }

    fn type_name(&self) -> &'static str {
        "DrawTrSurf_Curve"
    }

    fn ancestor(&self, type_id: TypeId) -> Option<&dyn Any> {
        self.drawable.as_kind(type_id)
    }

    /// `DrawOn`: the curve on its range (made finite for infinite curves), the
    /// orientation arrow and the curvature radii when enabled.
    fn draw_on(&self, display: &mut DrawDisplay<'_>) -> Result<(), Failure> {
        self.draw_curve(display)
    }

    /// `Copy`: a copy of the curve with the color and discretisation (the other
    /// options return to their defaults, as upstream).
    fn copy(&self, _this: &DrawableHandle) -> Result<DrawableHandle, Failure> {
        let copied = geometry(&self.curve)?.copy()?.cast::<dyn GeomCurve>();
        let look = CurveLook::new(self.style.color());
        let d = &self.drawable;
        let discretisation = (d.discretisation(), d.deflection(), d.draw_mode());
        Ok(Rc::new(Self::with_look(copied, look, discretisation)))
    }

    /// `Dump`: `GeomTools_CurveSet::PrintCurve` in the readable layout.
    fn dump(&self, stream: &mut DrawOStream) -> Result<(), Failure> {
        stream.write_through(|io| CurveSet::print_curve(&self.curve, io, false))
    }

    /// `Save`: `GeomTools_CurveSet::PrintCurve` in the compact layout.
    fn save(&self, stream: &mut DrawOStream) -> Result<(), Failure> {
        stream.write_through(|io| CurveSet::print_curve(&self.curve, io, true))
    }

    fn whatis(&self, di: &DrawInterpretor) -> Result<(), Failure> {
        di.append(" a 3d curve")?;
        Ok(())
    }
}
