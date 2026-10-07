// Rust translation of Draw/TKDraw/DrawTrSurf/DrawTrSurf_Surface.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DrawTrSurf_Surface`: a drawable surface (bounds, isos and an orientation mark).
use super::curve_common::finite_range;
use super::drawable::DrawTrSurfDrawable;
use super::geometry::geometry;
use super::params::current;
use crate::draw::{
    DrawColor, DrawColorKind, DrawDisplay, DrawIStream, DrawInterpretor, DrawOStream, DrawSession,
    Drawable3D, Drawable3DBase, DrawableHandle,
};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dIsoCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomSurface, SurfaceBounds};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeombase::geom_tools::SurfaceSet;
use rustycad_tkmath::geom_abs::IsoType;
use rustycad_tkmath::gp::Point3;
use std::any::{Any, TypeId};
use std::cell::Cell;
use std::rc::Rc;

/// The colors and iso counts of a surface drawable (the arguments of the second
/// `DrawTrSurf_Surface` constructor besides the discretisation).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceLook {
    /// `Nu`: the number of U isos (its absolute value is kept).
    pub nb_u_isos: i32,
    /// `Nv`: the number of V isos.
    pub nb_v_isos: i32,
    /// `BoundsColor`.
    pub bounds_color: DrawColor,
    /// `IsosColor`.
    pub isos_color: DrawColor,
}

/// `DrawTrSurf_Surface`: a `Geom_Surface` drawn as its bounds, `nbUIsos` x `nbVIsos`
/// isos and a small iso near the origin corner that shows the orientation.
///
/// The Bezier and BSpline surfaces embed this part; the BSpline flag "draw the knot
/// isos instead" lives here so that `ShowIsos` and `ClearIsos`, virtual in C++, clear
/// it whatever the concrete class (it is never set for other surfaces).
pub struct DrawTrSurfSurface {
    drawable: DrawTrSurfDrawable,
    surface: Handle<dyn GeomSurface>,
    bounds_look: Cell<DrawColor>,
    isos_look: Cell<DrawColor>,
    nb_u_isos: Cell<i32>,
    nb_v_isos: Cell<i32>,
    knots_isos: Cell<bool>,
}

impl DrawTrSurfSurface {
    /// `DrawTrSurf_Surface(S)`: yellow bounds, blue isos, one iso each way, 16 points,
    /// deflection 0.01, uniform deflection.
    pub fn new(surface: Handle<dyn GeomSurface>) -> Self {
        let look = SurfaceLook {
            nb_u_isos: 1,
            nb_v_isos: 1,
            bounds_color: DrawColor::new(DrawColorKind::Jaune),
            isos_color: DrawColor::new(DrawColorKind::Bleu),
        };
        Self::with_look(surface, look, (16, 0.01, 1))
    }

    /// `DrawTrSurf_Surface(S, Nu, Nv, BoundsColor, IsosColor, Discret, Deflection,
    /// DrawMode)`; `discretisation` is `(Discret, Deflection, DrawMode)`.
    pub fn with_look(
        surface: Handle<dyn GeomSurface>,
        look: SurfaceLook,
        (discret, deflection, draw_mode): (i32, f64, i32),
    ) -> Self {
        Self {
            drawable: DrawTrSurfDrawable::new(discret, deflection, draw_mode),
            surface,
            bounds_look: Cell::new(look.bounds_color),
            isos_look: Cell::new(look.isos_color),
            nb_u_isos: Cell::new(look.nb_u_isos.abs()),
            nb_v_isos: Cell::new(look.nb_v_isos.abs()),
            knots_isos: Cell::new(false),
        }
    }

    /// The `DrawTrSurf_Drawable` part.
    pub fn drawable(&self) -> &DrawTrSurfDrawable {
        &self.drawable
    }

    /// `GetSurface`.
    pub fn surface(&self) -> Handle<dyn GeomSurface> {
        self.surface.clone()
    }

    /// `BoundsColor`.
    pub fn bounds_color(&self) -> DrawColor {
        self.bounds_look.get()
    }

    /// `SetBoundsColor`.
    pub fn set_bounds_color(&self, color: DrawColor) {
        self.bounds_look.set(color);
    }

    /// `IsosColor`.
    pub fn isos_color(&self) -> DrawColor {
        self.isos_look.get()
    }

    /// `SetIsosColor`.
    pub fn set_isos_color(&self, color: DrawColor) {
        self.isos_look.set(color);
    }

    /// `NbIsos`: `(nbUIsos, nbVIsos)`.
    pub fn nb_isos(&self) -> (i32, i32) {
        (self.nb_u_isos.get(), self.nb_v_isos.get())
    }

    /// `ShowIsos(Nu, Nv)` (virtual): `|Nu|` by `|Nv|` isos, and a BSpline surface stops
    /// drawing its knot isos.
    pub fn show_isos(&self, nb_u: i32, nb_v: i32) {
        self.knots_isos.set(false);
        self.nb_u_isos.set(nb_u.abs());
        self.nb_v_isos.set(nb_v.abs());
    }

    /// `ClearIsos` (virtual): no isos (and no knot isos on a BSpline surface).
    pub fn clear_isos(&self) {
        self.knots_isos.set(false);
        self.nb_u_isos.set(0);
        self.nb_v_isos.set(0);
    }

    /// `knotsIsos` of `DrawTrSurf_BSplineSurface`.
    pub(crate) fn knots_isos(&self) -> bool {
        self.knots_isos.get()
    }

    /// `ShowKnotsIsos` of `DrawTrSurf_BSplineSurface`: the knot isos, with the iso
    /// counts set to the inner knot counts.
    pub(crate) fn set_knots_isos(&self, (nb_u, nb_v): (i32, i32)) {
        self.knots_isos.set(true);
        self.nb_u_isos.set(nb_u);
        self.nb_v_isos.set(nb_v);
    }

    /// The current look (for `Copy`).
    pub(crate) fn look(&self) -> SurfaceLook {
        let (nb_u_isos, nb_v_isos) = self.nb_isos();
        SurfaceLook {
            nb_u_isos,
            nb_v_isos,
            bounds_color: self.bounds_color(),
            isos_color: self.isos_color(),
        }
    }

    /// `RegisterFactory` (the `Draw_Drawable3D_FACTORY` macro).
    pub fn register_factory(session: &DrawSession) {
        session.register_factory("DrawTrSurf_Surface", Self::restore);
    }

    /// `Restore`: a surface read in the compact GeomTools format, with the settings of
    /// `DrawTrSurf::Parameters()`.
    pub fn restore(
        session: &DrawSession,
        stream: &mut DrawIStream,
    ) -> Result<Option<DrawableHandle>, Failure> {
        let params = current(session);
        let surface = stream.read_through(SurfaceSet::read_surface);
        let look = SurfaceLook {
            nb_u_isos: params.nb_u_isos,
            nb_v_isos: params.nb_v_isos,
            bounds_color: params.bounds_color,
            isos_color: params.isos_color,
        };
        let discretisation = (params.discret, params.deflection, params.draw_mode);
        Ok(Some(Rc::new(Self::with_look(
            surface,
            look,
            discretisation,
        ))))
    }

    /// `DrawOn(dis, Iso)`: the isos when `with_isos`, the finite bounds, then the
    /// orientation mark (a tenth of the first U iso from the corner).
    pub fn draw_on_with_isos(
        &self,
        display: &mut DrawDisplay<'_>,
        with_isos: bool,
    ) -> Result<(), Failure> {
        let surface = geometry(&self.surface)?;
        let declared = surface.bounds();
        let bounds = finite_bounds(surface, declared)?;
        let adaptor = GeomAdaptorSurface::default();
        adaptor.load_bounds(&self.surface, bounds)?;
        let iso =
            Adaptor3dIsoCurve::with_surface(&Handle::new(adaptor).cast::<dyn Adaptor3dSurface>());
        let (u1, u2, v1, v2) = bounds;
        if with_isos {
            display.set_color(self.isos_color());
            self.draw_isos(&iso, bounds, display)?;
        }
        display.set_color(self.bounds_color());
        let (u1_inf, u2_inf, v1_inf, v2_inf) = infinite_sides(declared);
        let sides = [
            (u1_inf, IsoType::IsoU, u1, (v1, v2)),
            (u2_inf, IsoType::IsoU, u2, (v1, v2)),
            (v1_inf, IsoType::IsoV, v1, (u1, u2)),
            (v2_inf, IsoType::IsoV, v2, (u1, u2)),
        ];
        for (infinite, kind, parameter, range) in sides {
            if !infinite {
                self.drawable
                    .draw_iso_curve_on(&iso, kind, parameter, range, display)?;
            }
        }
        let mark = (v1, v1 + (v2 - v1) / 10.);
        self.drawable
            .draw_iso_curve_on(&iso, IsoType::IsoU, u1 + (u2 - u1) / 10., mark, display)
    }

    /// `nbUIsos` U isos and `nbVIsos` V isos, evenly spaced inside the bounds.
    fn draw_isos(
        &self,
        iso: &Adaptor3dIsoCurve,
        (u1, u2, v1, v2): SurfaceBounds,
        display: &mut DrawDisplay<'_>,
    ) -> Result<(), Failure> {
        let (nb_u, nb_v) = self.nb_isos();
        let du = (u2 - u1) / f64::from(nb_u + 1);
        let mut u = u1;
        for _ in 1..=nb_u {
            u += du;
            self.drawable
                .draw_iso_curve_on(iso, IsoType::IsoU, u, (v1, v2), display)?;
        }
        let dv = (v2 - v1) / f64::from(nb_v + 1);
        let mut v = v1;
        for _ in 1..=nb_v {
            v += dv;
            self.drawable
                .draw_iso_curve_on(iso, IsoType::IsoV, v, (u1, u2), display)?;
        }
        Ok(())
    }
}

/// Which of `(U1, U2, V1, V2)` are infinite.
fn infinite_sides((u1, u2, v1, v2): SurfaceBounds) -> (bool, bool, bool, bool) {
    (
        precision::is_negative_infinite(u1),
        precision::is_positive_infinite(u2),
        precision::is_negative_infinite(v1),
        precision::is_positive_infinite(v2),
    )
}

/// The bounds drawn for a surface: an infinite U range is made finite along the iso
/// V at the middle (or at the finite V end, or 0), then an infinite V range along
/// the iso U at the middle of the (finite) U range.
fn finite_bounds(
    surface: &dyn GeomSurface,
    declared: SurfaceBounds,
) -> Result<SurfaceBounds, Failure> {
    let (u1, u2, v1, v2) = declared;
    let (_, _, v1_inf, v2_inf) = infinite_sides(declared);
    let v = match (v1_inf, v2_inf) {
        (true, true) => 0.0,
        (true, false) => v2,
        (false, true) => v1,
        (false, false) => (v1 + v2) / 2.,
    };
    let (u1, u2) = finite_range::<Point3>((u1, u2), |u| surface.d0(u, v))?;
    let u = (u1 + u2) / 2.;
    let (v1, v2) = finite_range::<Point3>((v1, v2), |v| surface.d0(u, v))?;
    Ok((u1, u2, v1, v2))
}

impl Drawable3D for DrawTrSurfSurface {
    fn base(&self) -> &Drawable3DBase {
        self.drawable.base()
    }

    fn type_name(&self) -> &'static str {
        "DrawTrSurf_Surface"
    }

    fn ancestor(&self, type_id: TypeId) -> Option<&dyn Any> {
        self.drawable.as_kind(type_id)
    }

    /// `DrawOn`: `DrawOn(dis, true)`.
    fn draw_on(&self, display: &mut DrawDisplay<'_>) -> Result<(), Failure> {
        self.draw_on_with_isos(display, true)
    }

    /// `Copy`: a copy of the surface with the same look and discretisation.
    fn copy(&self, _this: &DrawableHandle) -> Result<DrawableHandle, Failure> {
        let copied = geometry(&self.surface)?.copy()?.cast::<dyn GeomSurface>();
        let d = &self.drawable;
        let discretisation = (d.discretisation(), d.deflection(), d.draw_mode());
        Ok(Rc::new(Self::with_look(
            copied,
            self.look(),
            discretisation,
        )))
    }

    /// `Dump`: `GeomTools_SurfaceSet::PrintSurface` in the readable layout.
    fn dump(&self, stream: &mut DrawOStream) -> Result<(), Failure> {
        stream.write_through(|io| SurfaceSet::print_surface(&self.surface, io, false))
    }

    /// `Save`: `GeomTools_SurfaceSet::PrintSurface` in the compact layout.
    fn save(&self, stream: &mut DrawOStream) -> Result<(), Failure> {
        stream.write_through(|io| SurfaceSet::print_surface(&self.surface, io, true))
    }

    fn whatis(&self, di: &DrawInterpretor) -> Result<(), Failure> {
        di.append("a surface")?;
        Ok(())
    }
}
