// Rust translation of Draw/TKDraw/DrawTrSurf/DrawTrSurf_BezierSurface.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DrawTrSurf_BezierSurface`: a drawable Bezier surface with its pole net.
use super::geometry::geometry;
use super::params::current;
use super::spline_common::{PolesStyle, draw_pole_net, find_net_index, scaled_target};
use super::surface::{DrawTrSurfSurface, SurfaceLook};
use crate::draw::{
    DrawColor, DrawColorKind, DrawDisplay, DrawIStream, DrawInterpretor, DrawOStream, DrawSession,
    Drawable3D, Drawable3DBase, DrawableHandle, kind_of,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBezierSurface, GeomGeometry, GeomSurface};
use rustycad_tkgeombase::geom_tools::SurfaceSet;
use std::any::{Any, TypeId};
use std::rc::Rc;

/// `DrawTrSurf_BezierSurface`: a [`DrawTrSurfSurface`] on a `Geom_BezierSurface` that
/// also draws its poles.
pub struct DrawTrSurfBezierSurface {
    surface: DrawTrSurfSurface,
    poles: PolesStyle,
}

impl DrawTrSurfBezierSurface {
    /// `DrawTrSurf_BezierSurface(S)`: one iso each way, yellow bounds, blue isos, red
    /// poles shown, 30 points, deflection 0.05, discrete mode.
    pub fn new(surface: Handle<GeomBezierSurface>) -> Self {
        let look = SurfaceLook {
            nb_u_isos: 1,
            nb_v_isos: 1,
            bounds_color: DrawColor::new(DrawColorKind::Jaune),
            isos_color: DrawColor::new(DrawColorKind::Bleu),
        };
        Self::with_style(surface, look, PolesStyle::default_shown(), (30, 0.05, 0))
    }

    /// `DrawTrSurf_BezierSurface(S, NbUIsos, NbVIsos, BoundsColor, IsosColor,
    /// PolesColor, ShowPoles, Discret, Deflection, DrawMode)`.
    pub fn with_style(
        surface: Handle<GeomBezierSurface>,
        look: SurfaceLook,
        poles: PolesStyle,
        discretisation: (i32, f64, i32),
    ) -> Self {
        let geometry = surface.cast::<dyn GeomSurface>();
        Self {
            surface: DrawTrSurfSurface::with_look(geometry, look, discretisation),
            poles,
        }
    }

    /// The `DrawTrSurf_Surface` part.
    pub fn surface_part(&self) -> &DrawTrSurfSurface {
        &self.surface
    }

    /// `ShowPoles`, `ClearPoles`, `SetPolesColor`, `PolesColor`.
    pub fn poles(&self) -> &PolesStyle {
        &self.poles
    }

    /// The Bezier surface (`occ::down_cast<Geom_BezierSurface>(surf)`).
    fn bezier(&self) -> Handle<GeomBezierSurface> {
        self.surface.surface().cast::<GeomBezierSurface>()
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
        let handle = self.bezier();
        let bezier = geometry(&handle)?;
        let target = scaled_target(display, (x, y), precision);
        let counts = (bezier.nb_u_poles(), bezier.nb_v_poles());
        find_net_index(display, target, indices, counts, |u, v| bezier.pole(u, v))
    }

    /// `RegisterFactory` (the `Draw_Drawable3D_FACTORY` macro).
    pub fn register_factory(session: &DrawSession) {
        session.register_factory("DrawTrSurf_BezierSurface", Self::restore);
    }

    /// `Restore`: a surface read in the compact GeomTools format (no surface when it
    /// is not a Bezier surface), with the settings of `DrawTrSurf::Parameters()`.
    pub fn restore(session: &DrawSession, stream: &mut DrawIStream) -> Option<DrawableHandle> {
        let params = current(session);
        let surface = stream
            .read_through(SurfaceSet::read_surface)
            .cast::<GeomBezierSurface>();
        let look = SurfaceLook {
            nb_u_isos: params.nb_u_isos,
            nb_v_isos: params.nb_v_isos,
            bounds_color: params.bounds_color,
            isos_color: params.isos_color,
        };
        let poles = PolesStyle::new(params.is_show_poles, params.poles_color);
        let discretisation = (params.discret, params.deflection, params.draw_mode);
        Some(Rc::new(Self::with_style(
            surface,
            look,
            poles,
            discretisation,
        )))
    }
}

impl Drawable3D for DrawTrSurfBezierSurface {
    fn base(&self) -> &Drawable3DBase {
        self.surface.base()
    }

    fn type_name(&self) -> &'static str {
        "DrawTrSurf_BezierSurface"
    }

    fn ancestor(&self, type_id: TypeId) -> Option<&dyn Any> {
        kind_of(&self.surface, type_id)
    }

    /// `DrawOn`: the pole net when shown, then the surface with its isos.
    fn draw_on(&self, display: &mut DrawDisplay<'_>) -> Result<(), Failure> {
        if self.poles.shown() {
            let handle = self.bezier();
            let bezier = geometry(&handle)?;
            let counts = (bezier.nb_u_poles(), bezier.nb_v_poles());
            draw_pole_net(
                display,
                self.poles.color(),
                counts,
                (false, false),
                |u, v| bezier.pole(u, v),
            )?;
        }
        self.surface.draw_on_with_isos(display, true)
    }

    /// `Copy`: a copy of the surface with the same look and discretisation.
    fn copy(&self, _this: &DrawableHandle) -> Result<DrawableHandle, Failure> {
        let handle = self.bezier();
        let copied = geometry(&handle)?.copy()?.cast::<GeomBezierSurface>();
        let d = self.surface.drawable();
        let discretisation = (d.discretisation(), d.deflection(), d.draw_mode());
        let copy = Self::with_style(
            copied,
            self.surface.look(),
            self.poles.clone(),
            discretisation,
        );
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
