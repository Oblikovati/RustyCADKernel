// Rust translation of the DrawTrSurf::Set and DrawTrSurf::Get functions of
// Draw/TKDraw/DrawTrSurf/DrawTrSurf.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DrawTrSurf::Set` and `DrawTrSurf::Get`: geometry stored in DRAW variables.
//!
//! The getters return a null handle when the variable does not hold the expected kind
//! of drawable, as the C++ functions do.
use super::bezier_curve::DrawTrSurfBezierCurve;
use super::bezier_curve2d::DrawTrSurfBezierCurve2d;
use super::bezier_surface::DrawTrSurfBezierSurface;
use super::bspline_curve::DrawTrSurfBSplineCurve;
use super::bspline_curve2d::DrawTrSurfBSplineCurve2d;
use super::bspline_surface::DrawTrSurfBSplineSurface;
use super::curve::DrawTrSurfCurve;
use super::curve2d::DrawTrSurfCurve2d;
use super::params::current;
use super::point::DrawTrSurfPoint;
use super::polygon::{DrawTrSurfPolygon2D, DrawTrSurfPolygon3D};
use super::spline_common::{PolesStyle, SplineLook};
use super::style::CurveLook;
use super::surface::{DrawTrSurfSurface, SurfaceLook};
use super::triangulation::DrawTrSurfTriangulation;
use crate::draw::{DrawSession, Drawable3D, DrawableHandle};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCurve};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBSplineSurface, GeomBezierCurve, GeomBezierSurface, GeomCurve,
    GeomGeometry, GeomSurface,
};
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkmath::poly::{Polygon2D, Polygon3D, Triangulation};
use std::rc::Rc;

impl DrawSession {
    /// `DrawTrSurf::Set(name, gp_Pnt)`: a point with the current marker and color.
    pub fn set_point(&self, name: &str, point: Point3) -> Result<(), Failure> {
        let params = current(self);
        let drawable = DrawTrSurfPoint::new(point, params.pnt_marker, params.pnt_color);
        self.set_drawable_auto(name, Some(Rc::new(drawable)))
    }

    /// `DrawTrSurf::Set(name, gp_Pnt2d)`.
    pub fn set_point_2d(&self, name: &str, point: Point2) -> Result<(), Failure> {
        let params = current(self);
        let drawable = DrawTrSurfPoint::new_2d(point, params.pnt_marker, params.pnt_color);
        self.set_drawable_auto(name, Some(Rc::new(drawable)))
    }

    /// `DrawTrSurf::Set(name, Geom_Geometry, isSenseMarker)` (and its template overload
    /// for derived handles: cast them with `Handle::cast`): the drawable that fits the
    /// geometry, with the current parameters; a geometry that is neither a curve nor a
    /// surface (or a null handle) clears the variable.
    pub fn set_geometry(
        &self,
        name: &str,
        geometry: &Handle<dyn GeomGeometry>,
        sense_marker: bool,
    ) -> Result<(), Failure> {
        let drawable = geometry_drawable(self, geometry, sense_marker);
        self.set_drawable_auto(name, drawable)
    }

    /// `DrawTrSurf::Set(name, Geom2d_Curve, isSenseMarker)` (and its template
    /// overload): a Bezier, BSpline or other 2D curve drawable; a null handle clears
    /// the variable.
    pub fn set_curve_2d(
        &self,
        name: &str,
        curve: &Handle<dyn Geom2dCurve>,
        sense_marker: bool,
    ) -> Result<(), Failure> {
        let drawable = curve2d_drawable(self, curve, sense_marker);
        self.set_drawable_auto(name, drawable)
    }

    /// `DrawTrSurf::Set(name, Poly_Triangulation)`.
    pub fn set_triangulation(
        &self,
        name: &str,
        mesh: Handle<Triangulation>,
    ) -> Result<(), Failure> {
        let drawable = DrawTrSurfTriangulation::new(mesh)?;
        self.set_drawable_auto(name, Some(Rc::new(drawable)))
    }

    /// `DrawTrSurf::Set(name, Poly_Polygon3D)`.
    pub fn set_polygon_3d(&self, name: &str, polygon: Handle<Polygon3D>) -> Result<(), Failure> {
        self.set_drawable_auto(name, Some(Rc::new(DrawTrSurfPolygon3D::new(polygon))))
    }

    /// `DrawTrSurf::Set(name, Poly_Polygon2D)`.
    pub fn set_polygon_2d(&self, name: &str, polygon: Handle<Polygon2D>) -> Result<(), Failure> {
        self.set_drawable_auto(name, Some(Rc::new(DrawTrSurfPolygon2D::new(polygon))))
    }

    /// `DrawTrSurf::Get`: the curve or surface of a DrawTrSurf curve or surface variable.
    pub fn get_geometry(&self, name: &str) -> Handle<dyn GeomGeometry> {
        let Some(drawable) = self.get(name) else {
            return Handle::default();
        };
        if let Some(curve) = drawable.downcast_kind::<DrawTrSurfCurve>() {
            return curve.curve().cast::<dyn GeomGeometry>();
        }
        if let Some(surface) = drawable.downcast_kind::<DrawTrSurfSurface>() {
            return surface.surface().cast::<dyn GeomGeometry>();
        }
        Handle::default()
    }

    /// `DrawTrSurf::GetPoint`: the point of a 3D point variable.
    pub fn get_point(&self, name: &str) -> Option<Point3> {
        let drawable = self.get(name)?;
        let point = drawable.downcast_kind::<DrawTrSurfPoint>()?;
        point.is_3d().then(|| point.point())
    }

    /// `DrawTrSurf::GetPoint2d`: the point of a 2D point variable.
    pub fn get_point_2d(&self, name: &str) -> Option<Point2> {
        let drawable = self.get(name)?;
        let point = drawable.downcast_kind::<DrawTrSurfPoint>()?;
        (!point.is_3d()).then(|| point.point_2d())
    }

    /// `DrawTrSurf::GetCurve`.
    pub fn get_curve(&self, name: &str) -> Handle<dyn GeomCurve> {
        self.kind_of::<DrawTrSurfCurve, _>(name, |d| d.curve())
    }

    /// `DrawTrSurf::GetBezierCurve`: the curve of a Bezier curve variable.
    pub fn get_bezier_curve(&self, name: &str) -> Handle<GeomBezierCurve> {
        self.kind_of::<DrawTrSurfBezierCurve, _>(name, |d| d.curve_part().curve())
            .cast::<GeomBezierCurve>()
    }

    /// `DrawTrSurf::GetBSplineCurve`: the curve of a BSpline curve variable.
    pub fn get_bspline_curve(&self, name: &str) -> Handle<GeomBSplineCurve> {
        self.kind_of::<DrawTrSurfBSplineCurve, _>(name, |d| d.curve_part().curve())
            .cast::<GeomBSplineCurve>()
    }

    /// `DrawTrSurf::GetCurve2d`.
    pub fn get_curve_2d(&self, name: &str) -> Handle<dyn Geom2dCurve> {
        self.kind_of::<DrawTrSurfCurve2d, _>(name, |d| d.curve())
    }

    /// `DrawTrSurf::GetBezierCurve2d`: the curve of any 2D curve variable when it is a
    /// Bezier curve (the C++ code casts the curve, not the drawable).
    pub fn get_bezier_curve_2d(&self, name: &str) -> Handle<Geom2dBezierCurve> {
        self.get_curve_2d(name).cast::<Geom2dBezierCurve>()
    }

    /// `DrawTrSurf::GetBSplineCurve2d`: the curve of any 2D curve variable when it is a
    /// BSpline curve.
    pub fn get_bspline_curve_2d(&self, name: &str) -> Handle<Geom2dBSplineCurve> {
        self.get_curve_2d(name).cast::<Geom2dBSplineCurve>()
    }

    /// `DrawTrSurf::GetSurface`.
    pub fn get_surface(&self, name: &str) -> Handle<dyn GeomSurface> {
        self.kind_of::<DrawTrSurfSurface, _>(name, |d| d.surface())
    }

    /// `DrawTrSurf::GetBezierSurface`: the surface of a Bezier surface variable.
    pub fn get_bezier_surface(&self, name: &str) -> Handle<GeomBezierSurface> {
        self.kind_of::<DrawTrSurfBezierSurface, _>(name, |d| d.surface_part().surface())
            .cast::<GeomBezierSurface>()
    }

    /// `DrawTrSurf::GetBSplineSurface`: the surface of a BSpline surface variable.
    pub fn get_bspline_surface(&self, name: &str) -> Handle<GeomBSplineSurface> {
        self.kind_of::<DrawTrSurfBSplineSurface, _>(name, |d| d.surface_part().surface())
            .cast::<GeomBSplineSurface>()
    }

    /// `DrawTrSurf::GetTriangulation`.
    pub fn get_triangulation(&self, name: &str) -> Handle<Triangulation> {
        self.kind_of::<DrawTrSurfTriangulation, _>(name, |d| d.triangulation())
    }

    /// `DrawTrSurf::GetPolygon3D`.
    pub fn get_polygon_3d(&self, name: &str) -> Handle<Polygon3D> {
        self.kind_of::<DrawTrSurfPolygon3D, _>(name, |d| d.polygon())
    }

    /// `DrawTrSurf::GetPolygon2D`.
    pub fn get_polygon_2d(&self, name: &str) -> Handle<Polygon2D> {
        self.kind_of::<DrawTrSurfPolygon2D, _>(name, |d| d.polygon())
    }

    /// `occ::down_cast<D>(Draw::Get(name))` then `read`; a null handle otherwise.
    fn kind_of<D: 'static, T: ?Sized + rustycad_tkernel::standard::RuntimeTyped>(
        &self,
        name: &str,
        read: impl Fn(&D) -> Handle<T>,
    ) -> Handle<T> {
        self.get(name)
            .and_then(|drawable| drawable.downcast_kind::<D>().map(&read))
            .unwrap_or_default()
    }
}

/// The drawable `DrawTrSurf::Set(name, Geom_Geometry)` makes: Bezier and BSpline
/// curves and surfaces get their own drawables, other curves and surfaces the generic
/// ones.
fn geometry_drawable(
    session: &DrawSession,
    geometry: &Handle<dyn GeomGeometry>,
    sense_marker: bool,
) -> Option<DrawableHandle> {
    let params = current(session);
    let discretisation = (params.discret, params.deflection, params.draw_mode);
    let spline = SplineLook::from_params(&params);
    let bezier = geometry.cast::<GeomBezierCurve>();
    if bezier.is_some() {
        let colors = (params.curv_color, params.poles_color);
        let curve =
            DrawTrSurfBezierCurve::with_style(bezier, colors, params.is_show_poles, discretisation);
        return Some(Rc::new(curve));
    }
    let bspline = geometry.cast::<GeomBSplineCurve>();
    if bspline.is_some() {
        return Some(Rc::new(DrawTrSurfBSplineCurve::with_style(
            bspline,
            &spline,
            discretisation,
        )));
    }
    let curve = geometry.cast::<dyn GeomCurve>();
    if curve.is_some() {
        let look = CurveLook {
            display_origin: sense_marker,
            ..CurveLook::new(params.curv_color)
        };
        return Some(Rc::new(DrawTrSurfCurve::with_look(
            curve,
            look,
            discretisation,
        )));
    }
    surface_drawable(session, geometry)
}

/// The surface part of [`geometry_drawable`].
fn surface_drawable(
    session: &DrawSession,
    geometry: &Handle<dyn GeomGeometry>,
) -> Option<DrawableHandle> {
    let params = current(session);
    let discretisation = (params.discret, params.deflection, params.draw_mode);
    let look = SurfaceLook {
        nb_u_isos: params.nb_u_isos,
        nb_v_isos: params.nb_v_isos,
        bounds_color: params.bounds_color,
        isos_color: params.isos_color,
    };
    let bezier = geometry.cast::<GeomBezierSurface>();
    if bezier.is_some() {
        let poles = PolesStyle::new(params.is_show_poles, params.poles_color);
        return Some(Rc::new(DrawTrSurfBezierSurface::with_style(
            bezier,
            look,
            poles,
            discretisation,
        )));
    }
    let bspline = geometry.cast::<GeomBSplineSurface>();
    if bspline.is_some() {
        return Some(Rc::new(DrawTrSurfBSplineSurface::from_params(
            bspline, &params,
        )));
    }
    let surface = geometry.cast::<dyn GeomSurface>();
    if surface.is_some() {
        return Some(Rc::new(DrawTrSurfSurface::with_look(
            surface,
            look,
            discretisation,
        )));
    }
    None
}

/// The drawable `DrawTrSurf::Set(name, Geom2d_Curve)` makes.
fn curve2d_drawable(
    session: &DrawSession,
    curve: &Handle<dyn Geom2dCurve>,
    sense_marker: bool,
) -> Option<DrawableHandle> {
    let params = current(session);
    let bezier = curve.cast::<Geom2dBezierCurve>();
    if bezier.is_some() {
        let colors = (params.curv_color, params.poles_color);
        let drawable = DrawTrSurfBezierCurve2d::with_style(
            bezier,
            colors,
            params.is_show_poles,
            params.discret,
        );
        return Some(Rc::new(drawable));
    }
    let bspline = curve.cast::<Geom2dBSplineCurve>();
    if bspline.is_some() {
        let spline = SplineLook::from_params(&params);
        return Some(Rc::new(DrawTrSurfBSplineCurve2d::with_style(
            bspline,
            &spline,
            params.discret,
        )));
    }
    if curve.is_null() {
        return None;
    }
    let look = CurveLook {
        display_origin: sense_marker,
        ..CurveLook::new(params.curv_color)
    };
    Some(Rc::new(DrawTrSurfCurve2d::with_look(
        curve.clone(),
        look,
        params.discret,
    )))
}
