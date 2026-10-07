// Rust translation of Draw/TKDraw/DrawTrSurf/DrawTrSurf_BSplineSurface.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DrawTrSurf_BSplineSurface`: a drawable BSpline surface with its pole net, knots and
//! knot isos.
use super::geometry::geometry;
use super::params::{DrawTrSurfParams, current};
use super::spline_common::{
    KnotsStyle, PolesStyle, draw_pole_net, find_index, find_net_index, scaled_target,
};
use super::surface::{DrawTrSurfSurface, SurfaceLook};
use crate::draw::{
    DrawColor, DrawColorKind, DrawDisplay, DrawIStream, DrawInterpretor, DrawOStream, DrawSession,
    Drawable3D, Drawable3DBase, DrawableHandle, kind_of,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dIsoCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomGeometry, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeombase::geom_tools::SurfaceSet;
use rustycad_tkmath::geom_abs::IsoType;
use rustycad_tkmath::gp::Point2;
use std::any::{Any, TypeId};
use std::rc::Rc;

/// The pole and knot settings of a spline surface.
type ControlStyles = (PolesStyle, KnotsStyle);

/// `DrawTrSurf_BSplineSurface`: a [`DrawTrSurfSurface`] on a `Geom_BSplineSurface`
/// that also draws its poles, marks its knots, and by default draws its inner knot
/// isos instead of evenly spaced isos.
pub struct DrawTrSurfBSplineSurface {
    surface: DrawTrSurfSurface,
    poles: PolesStyle,
    knots: KnotsStyle,
}

impl DrawTrSurfBSplineSurface {
    /// `DrawTrSurf_BSplineSurface(S)`: knot isos, yellow bounds, blue isos, red poles
    /// and violet diamond knots shown, 30 points, deflection 0.05, discrete mode.
    pub fn new(surface: Handle<GeomBSplineSurface>) -> Self {
        let colors = (
            DrawColor::new(DrawColorKind::Jaune),
            DrawColor::new(DrawColorKind::Bleu),
        );
        let styles = (PolesStyle::default_shown(), KnotsStyle::default_shown());
        Self::with_knot_isos(surface, colors, styles, (30, 0.05, 0))
    }

    /// `DrawTrSurf_BSplineSurface(S, BoundsColor, IsosColor, PolesColor, KnotsColor,
    /// KnotsShape, KnotsSize, ShowPoles, ShowKnots, Discret, Deflection, DrawMode)`:
    /// the inner knot isos are drawn. Without a surface (an unreadable record) the
    /// iso counts are 0 where the C++ code would dereference the null surface.
    pub fn with_knot_isos(
        surface: Handle<GeomBSplineSurface>,
        (bounds_color, isos_color): (DrawColor, DrawColor),
        (poles, knots): ControlStyles,
        discretisation: (i32, f64, i32),
    ) -> Self {
        let (nb_u_isos, nb_v_isos) = inner_knot_counts(&surface);
        let look = SurfaceLook {
            nb_u_isos,
            nb_v_isos,
            bounds_color,
            isos_color,
        };
        let drawable = Self::with_isos(surface, look, (poles, knots), discretisation);
        drawable.surface.set_knots_isos(drawable.surface.nb_isos());
        drawable
    }

    /// `DrawTrSurf_BSplineSurface(S, NbUIsos, NbVIsos, BoundsColor, IsosColor,
    /// PolesColor, KnotsColor, KnotsShape, KnotsSize, ShowPoles, ShowKnots, Discret,
    /// Deflection, DrawMode)`: evenly spaced isos.
    pub fn with_isos(
        surface: Handle<GeomBSplineSurface>,
        look: SurfaceLook,
        (poles, knots): ControlStyles,
        discretisation: (i32, f64, i32),
    ) -> Self {
        let geometry = surface.cast::<dyn GeomSurface>();
        Self {
            surface: DrawTrSurfSurface::with_look(geometry, look, discretisation),
            poles,
            knots,
        }
    }

    /// The `DrawTrSurf_Surface` part (`ShowIsos`, `ClearIsos`, colors, `NbIsos`).
    pub fn surface_part(&self) -> &DrawTrSurfSurface {
        &self.surface
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

    /// `ShowKnotsIsos`: draw the inner knot isos again.
    pub fn show_knots_isos(&self) -> Result<(), Failure> {
        let handle = self.bspline();
        let bspline = geometry(&handle)?;
        let counts = (bspline.nb_u_knots() - 2, bspline.nb_v_knots() - 2);
        self.surface.set_knots_isos(counts);
        Ok(())
    }

    /// The BSpline surface (`occ::down_cast<Geom_BSplineSurface>(surf)`).
    fn bspline(&self) -> Handle<GeomBSplineSurface> {
        self.surface.surface().cast::<GeomBSplineSurface>()
    }

    /// `FindPole`: the first pole after `(u_index, v_index)` whose projection is
    /// within `precision` pixels of the pick point `(x, y)`; `(0, 0)` when none is.
    pub fn find_pole(
        &self,
        (x, y): (f64, f64),
        display: &DrawDisplay<'_>,
        precision: f64,
        indices: (i32, i32),
    ) -> Result<(i32, i32), Failure> {
        let handle = self.bspline();
        let bspline = geometry(&handle)?;
        let target = scaled_target(display, (x, y), precision);
        let counts = (bspline.nb_u_poles(), bspline.nb_v_poles());
        find_net_index(display, target, indices, counts, |u, v| bspline.pole(u, v))
    }

    /// `FindUKnot`: the first U knot after `index` whose point on the first V bound
    /// projects within `precision` of `(x, y)` (model units); 0 when none does.
    pub fn find_u_knot(
        &self,
        (x, y): (f64, f64),
        display: &DrawDisplay<'_>,
        precision: f64,
        index: i32,
    ) -> Result<i32, Failure> {
        let handle = self.bspline();
        let bspline = geometry(&handle)?;
        let (_, _, v1, _) = bspline.bounds();
        let target = (Point2::new(x, y), precision);
        find_index(display, target, (index, bspline.nb_u_knots()), |i| {
            bspline.value(bspline.u_knot(i)?, v1)
        })
    }

    /// `FindVKnot`: the first V knot after `index` whose point on the first U bound
    /// projects within `precision` of `(x, y)` (model units); 0 when none does.
    pub fn find_v_knot(
        &self,
        (x, y): (f64, f64),
        display: &DrawDisplay<'_>,
        precision: f64,
        index: i32,
    ) -> Result<i32, Failure> {
        let handle = self.bspline();
        let bspline = geometry(&handle)?;
        let (u1, _, _, _) = bspline.bounds();
        let target = (Point2::new(x, y), precision);
        find_index(display, target, (index, bspline.nb_v_knots()), |j| {
            bspline.value(u1, bspline.v_knot(j)?)
        })
    }

    /// `RegisterFactory` (the `Draw_Drawable3D_FACTORY` macro).
    pub fn register_factory(session: &DrawSession) {
        session.register_factory("DrawTrSurf_BSplineSurface", Self::restore);
    }

    /// `Restore`: a surface read in the compact GeomTools format (no surface when it
    /// is not a BSpline surface), with the settings of `DrawTrSurf::Parameters()`:
    /// knot isos when `NeedKnotsIsos`, else `NbUIsos` x `NbVIsos` isos.
    pub fn restore(
        session: &DrawSession,
        stream: &mut DrawIStream,
    ) -> Result<Option<DrawableHandle>, Failure> {
        let params = current(session);
        let surface = stream
            .read_through(SurfaceSet::read_surface)
            .cast::<GeomBSplineSurface>();
        Ok(Some(Rc::new(Self::from_params(surface, &params))))
    }

    /// The drawable `DrawTrSurf::Set` and `Restore` make with the parameters.
    pub(crate) fn from_params(
        surface: Handle<GeomBSplineSurface>,
        params: &DrawTrSurfParams,
    ) -> Self {
        let styles = (
            PolesStyle::new(params.is_show_poles, params.poles_color),
            KnotsStyle::new(
                params.is_show_knots,
                params.knots_marker,
                params.knots_color,
                params.knots_size,
            ),
        );
        let discretisation = (params.discret, params.deflection, params.draw_mode);
        let colors = (params.bounds_color, params.isos_color);
        if params.need_knots_isos {
            return Self::with_knot_isos(surface, colors, styles, discretisation);
        }
        let look = SurfaceLook {
            nb_u_isos: params.nb_u_isos,
            nb_v_isos: params.nb_v_isos,
            bounds_color: colors.0,
            isos_color: colors.1,
        };
        Self::with_isos(surface, look, styles, discretisation)
    }

    /// The inner knot isos: U isos at the knots strictly inside the first and last
    /// used U knots over the V range, then the V isos.
    fn draw_knot_isos(
        &self,
        bspline: &GeomBSplineSurface,
        display: &mut DrawDisplay<'_>,
    ) -> Result<(), Failure> {
        let (u1, u2, v1, v2) = bspline.bounds();
        let adaptor = GeomAdaptorSurface::default();
        adaptor.load(&self.surface.surface())?;
        let iso =
            Adaptor3dIsoCurve::with_surface(&Handle::new(adaptor).cast::<dyn Adaptor3dSurface>());
        display.set_color(self.surface.isos_color());
        let drawable = self.surface.drawable();
        for i in bspline.first_u_knot_index()? + 1..bspline.last_u_knot_index()? {
            let u = bspline.u_knot(i)?;
            drawable.draw_iso_curve_on(&iso, IsoType::IsoU, u, (v1, v2), display)?;
        }
        for j in bspline.first_v_knot_index()? + 1..bspline.last_v_knot_index()? {
            let v = bspline.v_knot(j)?;
            drawable.draw_iso_curve_on(&iso, IsoType::IsoV, v, (u1, u2), display)?;
        }
        Ok(())
    }

    /// The knot markers: the used U knots on the first V bound, then the used V knots
    /// on the first U bound.
    fn mark_knots(
        &self,
        bspline: &GeomBSplineSurface,
        display: &mut DrawDisplay<'_>,
    ) -> Result<(), Failure> {
        let (u1, _, v1, _) = bspline.bounds();
        let u_range = bspline.first_u_knot_index()?..=bspline.last_u_knot_index()?;
        let on_v1 = u_range.map(|i| bspline.value(bspline.u_knot(i)?, v1));
        self.knots.mark(display, on_v1)?;
        let v_range = bspline.first_v_knot_index()?..=bspline.last_v_knot_index()?;
        let on_u1 = v_range.map(|j| bspline.value(u1, bspline.v_knot(j)?));
        self.knots.mark(display, on_u1)
    }
}

/// `S->NbUKnots() - 2` and `S->NbVKnots() - 2`; 0 without a surface.
fn inner_knot_counts(surface: &Handle<GeomBSplineSurface>) -> (i32, i32) {
    surface
        .get()
        .map_or((0, 0), |s| (s.nb_u_knots() - 2, s.nb_v_knots() - 2))
}

impl Drawable3D for DrawTrSurfBSplineSurface {
    fn base(&self) -> &Drawable3DBase {
        self.surface.base()
    }

    fn type_name(&self) -> &'static str {
        "DrawTrSurf_BSplineSurface"
    }

    fn ancestor(&self, type_id: TypeId) -> Option<&dyn Any> {
        kind_of(&self.surface, type_id)
    }

    /// `DrawOn`: the pole net (closed along periodic directions), the knot isos when
    /// enabled, the surface (with its own isos otherwise), then the knot markers.
    fn draw_on(&self, display: &mut DrawDisplay<'_>) -> Result<(), Failure> {
        let handle = self.bspline();
        let bspline = geometry(&handle)?;
        if self.poles.shown() {
            let counts = (bspline.nb_u_poles(), bspline.nb_v_poles());
            let periodic = (bspline.is_u_periodic(), bspline.is_v_periodic());
            draw_pole_net(display, self.poles.color(), counts, periodic, |u, v| {
                bspline.pole(u, v)
            })?;
        }
        let knots_isos = self.surface.knots_isos();
        if knots_isos {
            self.draw_knot_isos(bspline, display)?;
        }
        self.surface.draw_on_with_isos(display, !knots_isos)?;
        if self.knots.shown() {
            self.mark_knots(bspline, display)?;
        }
        Ok(())
    }

    /// `Copy`: a copy of the surface with the same look, with knot isos or with the
    /// current iso counts.
    fn copy(&self, _this: &DrawableHandle) -> Result<DrawableHandle, Failure> {
        let handle = self.bspline();
        let copied = geometry(&handle)?.copy()?.cast::<GeomBSplineSurface>();
        let d = self.surface.drawable();
        let discretisation = (d.discretisation(), d.deflection(), d.draw_mode());
        let styles = (self.poles.clone(), self.knots.clone());
        let look = self.surface.look();
        let copy = if self.surface.knots_isos() {
            let colors = (look.bounds_color, look.isos_color);
            Self::with_knot_isos(copied, colors, styles, discretisation)
        } else {
            Self::with_isos(copied, look, styles, discretisation)
        };
        Ok(Rc::new(copy))
    }

    fn dump(&self, stream: &mut DrawOStream) -> Result<(), Failure> {
        self.surface.dump(stream)
    }

    fn save(&self, stream: &mut DrawOStream) -> Result<(), Failure> {
        self.surface.save(stream)
    }

    fn whatis(&self, di: &DrawInterpretor) -> Result<(), Failure> {
        self.surface.whatis(di)
    }
}
