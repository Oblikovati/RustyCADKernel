// Rust translation of Draw/TKDraw/DrawTrSurf/DrawTrSurf_Curve2d.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DrawTrSurf_Curve2d`: a drawable 2D curve.
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
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeombase::geom_lprop::CLProps2d;
use rustycad_tkgeombase::geom_tools::Curve2dSet;
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::Point2;
use std::any::{Any, TypeId};
use std::rc::Rc;

/// The deflection and draw mode of a 2D curve: the defaults of the
/// `DrawTrSurf_Drawable(discret)` constructor the 2D curve calls.
const DEFLECTION: f64 = 0.01;

/// `DrawTrSurf_Curve2d`: a `Geom2d_Curve` with its color, orientation arrow and
/// optional curvature radii; a 2D object (`Is3D` is false).
///
/// ```
/// use rustycad_tkdraw::draw::Drawable3D;
/// use rustycad_tkdraw::draw_tr_surf::DrawTrSurfCurve2d;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
/// use rustycad_tkmath::gp::{Axis2d, Direction2, Point2};
/// let axis = Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0).unwrap());
/// let line = Handle::new(Geom2dLine::new(axis)).cast::<dyn Geom2dCurve>();
/// let curve = DrawTrSurfCurve2d::new(line, true);
/// assert!(!curve.is_3d());
/// assert_eq!(curve.drawable().discretisation(), 50);
/// ```
pub struct DrawTrSurfCurve2d {
    drawable: DrawTrSurfDrawable,
    curve: Handle<dyn Geom2dCurve>,
    style: CurveStyle,
}

impl DrawTrSurfCurve2d {
    /// `DrawTrSurf_Curve2d(C, DispOrigin)`: green, 50 points per interval.
    pub fn new(curve: Handle<dyn Geom2dCurve>, display_origin: bool) -> Self {
        let look = CurveLook {
            display_origin,
            ..CurveLook::new(DrawColor::new(DrawColorKind::Vert))
        };
        Self::with_look(curve, look, 50)
    }

    /// `DrawTrSurf_Curve2d(C, aColor, Discret, DispOrigin, DispCurvRadius, RadiusMax,
    /// RatioOfRadius)`.
    pub fn with_look(curve: Handle<dyn Geom2dCurve>, look: CurveLook, discret: i32) -> Self {
        Self {
            drawable: DrawTrSurfDrawable::new(discret, DEFLECTION, 0),
            curve,
            style: CurveStyle::new(look),
        }
    }

    /// The `DrawTrSurf_Drawable` part.
    pub fn drawable(&self) -> &DrawTrSurfDrawable {
        &self.drawable
    }

    /// `GetCurve`.
    pub fn curve(&self) -> Handle<dyn Geom2dCurve> {
        self.curve.clone()
    }

    /// The color, orientation arrow and curvature settings.
    pub fn style(&self) -> &CurveStyle {
        &self.style
    }

    /// `RegisterFactory` (the `Draw_Drawable3D_FACTORY` macro).
    pub fn register_factory(session: &DrawSession) {
        session.register_factory("DrawTrSurf_Curve2d", Self::restore);
    }

    /// `Restore`: a curve read in the compact GeomTools format, with the curve color
    /// and discretisation of `DrawTrSurf::Parameters()`.
    pub fn restore(
        session: &DrawSession,
        stream: &mut DrawIStream,
    ) -> Result<Option<DrawableHandle>, Failure> {
        let params = current(session);
        let curve = stream.read_through(Curve2dSet::read_curve2d);
        let look = CurveLook::new(params.curv_color);
        Ok(Some(Rc::new(Self::with_look(curve, look, params.discret))))
    }

    /// The orientation arrow at the end of the drawn range.
    fn draw_origin(
        &self,
        adaptor: &Geom2dAdaptorCurve,
        last: f64,
        display: &mut DrawDisplay<'_>,
    ) -> Result<(), Failure> {
        let (point, vector) = adaptor.d1(last)?;
        draw_orientation_arrow(display, point, vector)
    }

    /// The curvature radii of a non-linear curve.
    fn draw_curvature(
        &self,
        adaptor: &Geom2dAdaptorCurve,
        display: &mut DrawDisplay<'_>,
    ) -> Result<(), Failure> {
        let bounds = interval_bounds(adaptor.nb_intervals(Shape::CN)?, |b| {
            adaptor.intervals(b, Shape::CN)
        })?;
        let radii = CurvatureDisplay::of(self.drawable.discretisation(), &self.style);
        let mut props = CLProps2d::new(self.curve.clone(), 2, radii.resolution())?;
        let curve = geometry(&self.curve)?;
        radii.draw(display, &bounds, &mut props, |t| curve.d0(t))
    }
}

impl Drawable3D for DrawTrSurfCurve2d {
    fn base(&self) -> &Drawable3DBase {
        self.drawable.base()
    }

    fn type_name(&self) -> &'static str {
        "DrawTrSurf_Curve2d"
    }

    fn ancestor(&self, type_id: TypeId) -> Option<&dyn Any> {
        self.drawable.as_kind(type_id)
    }

    /// `DrawOn`: the curve on its range (made finite for infinite curves), the
    /// orientation arrow and the curvature radii when enabled.
    fn draw_on(&self, display: &mut DrawDisplay<'_>) -> Result<(), Failure> {
        let curve = geometry(&self.curve)?;
        let range = (curve.first_parameter(), curve.last_parameter());
        let (first, last) = finite_range::<Point2>(range, |t| curve.d0(t))?;
        display.set_color(self.style.color());
        let adaptor = Geom2dAdaptorCurve::with_range(&self.curve, first, last)?;
        self.drawable.draw_curve2d_on(&adaptor, display)?;
        if self.style.display_origin() {
            self.draw_origin(&adaptor, last, display)?;
        }
        if self.style.curvature_shown() && adaptor.get_type()? != CurveType::Line {
            self.draw_curvature(&adaptor, display)?;
        }
        Ok(())
    }

    /// `Copy`: a copy of the curve with the color and the number of points (the other
    /// options return to their defaults, as upstream).
    fn copy(&self, _this: &DrawableHandle) -> Result<DrawableHandle, Failure> {
        let copied = geometry(&self.curve)?.copy()?.cast::<dyn Geom2dCurve>();
        let look = CurveLook::new(self.style.color());
        Ok(Rc::new(Self::with_look(
            copied,
            look,
            self.drawable.discretisation(),
        )))
    }

    /// `Dump`: `GeomTools_Curve2dSet::PrintCurve2d` in the readable layout.
    fn dump(&self, stream: &mut DrawOStream) -> Result<(), Failure> {
        stream.write_through(|io| Curve2dSet::print_curve2d(&self.curve, io, false))
    }

    /// `Save`: `GeomTools_Curve2dSet::PrintCurve2d` in the compact layout.
    fn save(&self, stream: &mut DrawOStream) -> Result<(), Failure> {
        stream.write_through(|io| Curve2dSet::print_curve2d(&self.curve, io, true))
    }

    fn whatis(&self, di: &DrawInterpretor) -> Result<(), Failure> {
        di.append("2d curve")?;
        Ok(())
    }

    fn is_3d(&self) -> bool {
        false
    }
}
