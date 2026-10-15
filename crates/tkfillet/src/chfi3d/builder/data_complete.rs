// Rust translation of the ChFi3d_Builder::CompleteData overloads (ChFi3d_Builder_6.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The fillet of a walked line approximated and stored, and the surface of a corner
//! filling stored with its sides on the faces.
use super::ChFi3dBuilder;
use crate::brep_blend::{
    AppSurfaceOptions, BRepBlendAppFunc, BRepBlendAppFuncRst, BRepBlendAppFuncRstRst,
    BRepBlendAppSurface, BRepBlendLineHandle, SharedAppFunction,
};
use crate::chfi_ds::{SurfDataHandle, lock};
use crate::chfi3d::same_parameter;
use rustycad_tkbool::top_ope_brep_ds::{TopOpeBRepDSCurve, TopOpeBRepDSSurface};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::{self, Orientation};
use rustycad_tkgeombase::approx::SweepFunction;
use rustycad_tkmath::gp::{Direction2, Point2, Vector2};

/// The kind of function a line was walked with.
pub(crate) enum LineFunction {
    /// A function between two surfaces (`BRepBlend_AppFunc`).
    Surfaces,
    /// A function between a surface and a restriction (`BRepBlend_AppFuncRst`).
    SurfRst,
    /// A function between two restrictions (`BRepBlend_AppFuncRstRst`).
    RstRst,
}

/// The straight pcurve on the corner surface from `pd` to `pf`.
fn line_2d(pd: Point2, pf: Point2) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    Ok(Handle::new(Geom2dLine::from_point(
        pd,
        Direction2::from_vector(Vector2::between(pd, pf))?,
    ))
    .cast())
}

impl ChFi3dBuilder {
    /// One side of the corner surface on its face: its iso curve, the pcurve of the face
    /// trimmed and brought to the same parameter; the transition and whether the normals
    /// agree.
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::type_complexity)]
    fn coin_side(
        &self,
        surfcoin: &Handle<dyn GeomSurface>,
        v: f64,
        (s, pc): (&Handle<dyn Adaptor3dSurface>, &Handle<dyn Geom2dCurve>),
        (ufirst, ulast): (f64, f64),
        tolreached: &mut f64,
    ) -> Result<(Handle<dyn GeomCurve>, Handle<dyn Geom2dCurve>, Option<f64>), Failure> {
        if pc.is_null() {
            return Ok((Handle::default(), Handle::default(), None));
        }
        let sc = surfcoin.get().ok_or_else(|| Failure::new("CompleteData"))?;
        let crv3d = sc.v_iso(v)?;
        let hcs: Handle<dyn Adaptor3dCurve> =
            Handle::new(GeomAdaptorCurve::with_curve(&crv3d)?).cast();
        let mut c2dtrim: Handle<dyn Geom2dCurve> =
            Handle::new(Geom2dTrimmedCurve::new(pc, ufirst, ulast, true, true)?).cast();
        same_parameter(&hcs, &mut c2dtrim, s, self.tolapp3d, tolreached)?;
        let w = 0.5 * (ufirst + ulast);
        let xy = c2dtrim
            .get()
            .ok_or_else(|| Failure::new("CompleteData"))?
            .value(w)?;
        let d1 = s
            .get()
            .ok_or_else(|| Failure::new("CompleteData"))?
            .d1(xy.x(), xy.y())?;
        let nf = d1.d1u.crossed(d1.d1v);
        let ds = sc.d1(w, v)?;
        let ns = ds.d1u.crossed(ds.d1v);
        Ok((crv3d, c2dtrim, Some(nf.dot(ns))))
    }

    /// `CompleteData(Data, Surfcoin, S1, PC1, S2, PC2, Or, On1, Gd1, Gd2, Gf1, Gf2)`: the
    /// surface of a corner filling stored with its isos on the two faces (the points of
    /// the ends not given set on its corners); `g` is `[Gd1, Gd2, Gf1, Gf2]`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn complete_data_coin(
        &mut self,
        data: &SurfDataHandle,
        surfcoin: &Handle<dyn GeomSurface>,
        side1: (&Handle<dyn Adaptor3dSurface>, &Handle<dyn Geom2dCurve>),
        side2: (&Handle<dyn Adaptor3dSurface>, &Handle<dyn Geom2dCurve>),
        or: Orientation,
        on1: bool,
        g: [bool; 4],
    ) -> Result<bool, Failure> {
        let isurf = self
            .ds
            .add_surface(&TopOpeBRepDSSurface::new(surfcoin, self.tolapp3d));
        lock(data).change_surf(isurf);
        let sc = surfcoin.get().ok_or_else(|| Failure::new("CompleteData"))?;
        let (ufirst, ulast, vfirst, vlast) = sc.bounds();
        {
            let mut d = lock(data);
            let corners = [
                (g[0], true, 1, ufirst, vfirst),
                (g[1], true, 2, ufirst, vlast),
                (g[2], false, 1, ulast, vfirst),
                (g[3], false, 2, ulast, vlast),
            ];
            for (given, first, ons, u, v) in corners {
                if !given {
                    d.change_vertex(first, ons).set_point(sc.value(u, v)?);
                }
            }
        }
        let mut orsurf = or;
        let mut tolreached = 1.0e-5;
        let (crv3d1, c2d1, dot1) =
            self.coin_side(surfcoin, vfirst, side1, (ufirst, ulast), &mut tolreached)?;
        let mut tra1 = Orientation::Forward;
        if let Some(d) = dot1 {
            if d > 0. {
                tra1 = Orientation::Reversed;
            } else if on1 {
                orsurf = top_abs::reverse(orsurf);
            }
        }
        let index1 = self
            .ds
            .add_curve(&mut TopOpeBRepDSCurve::new(&crv3d1, tolreached, false));
        {
            let mut d = lock(data);
            let f = d.change_interference(1);
            f.set_first_parameter(ufirst);
            f.set_last_parameter(ulast);
            f.set_interference(
                index1,
                tra1,
                &c2d1,
                &line_2d(Point2::new(ufirst, vfirst), Point2::new(ulast, vfirst))?,
            );
        }
        let (crv3d2, c2d2, dot2) =
            self.coin_side(surfcoin, vlast, side2, (ufirst, ulast), &mut tolreached)?;
        let c2d2 = if side2.1.is_null() { c2d1 } else { c2d2 };
        let mut tra2 = Orientation::Forward;
        if let Some(d) = dot2
            && d < 0.
        {
            tra2 = Orientation::Reversed;
            if !on1 {
                orsurf = top_abs::reverse(orsurf);
            }
        }
        let index2 = self
            .ds
            .add_curve(&mut TopOpeBRepDSCurve::new(&crv3d2, tolreached, false));
        let mut d = lock(data);
        let f = d.change_interference(2);
        f.set_first_parameter(ufirst);
        f.set_last_parameter(ulast);
        f.set_interference(
            index2,
            tra2,
            &c2d2,
            &line_2d(Point2::new(ufirst, vlast), Point2::new(ulast, vlast))?,
        );
        *d.change_orientation() = orsurf;
        Ok(true)
    }

    /// `CompleteData(Data, Func, Lin, S1, S2, Or1, Gd1, Gd2, Gf1, Gf2, Reversed)` and the
    /// overloads on restrictions: the line approximated (degree 20 and 5000 segments at
    /// most between two surfaces, the defaults otherwise) and stored.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn complete_data(
        &mut self,
        data: &SurfDataHandle,
        (kind, func): (LineFunction, &SharedAppFunction),
        lin: &BRepBlendLineHandle,
        faces: (&Handle<dyn Adaptor3dSurface>, &Handle<dyn Adaptor3dSurface>),
        or1: Orientation,
        g: [bool; 4],
        reversed: bool,
    ) -> Result<bool, Failure> {
        let (the_func, degmax, segmax): (Handle<dyn SweepFunction>, i32, i32) = match kind {
            LineFunction::Surfaces => (
                Handle::new(BRepBlendAppFunc::new(lin, func, self.tolapp3d, 1.0e-5)?).cast(),
                20,
                5000,
            ),
            LineFunction::SurfRst => (
                Handle::new(BRepBlendAppFuncRst::new(lin, func, self.tolapp3d, 1.0e-5)?).cast(),
                11,
                50,
            ),
            LineFunction::RstRst => (
                Handle::new(BRepBlendAppFuncRstRst::new(
                    lin,
                    func,
                    self.tolapp3d,
                    1.0e-5,
                )?)
                .cast(),
                11,
                50,
            ),
        };
        let (first, last) = {
            let l = lock(lin);
            (l.point(1)?.parameter(), l.point(l.nb_points())?.parameter())
        };
        let options = AppSurfaceOptions {
            tol3d: self.tolapp3d,
            tol2d: 1.0e-5,
            tol_angular: self.tolappangle,
            continuity: self.my_conti,
            degmax,
            segmax,
        };
        let approx = BRepBlendAppSurface::new(&the_func, first, last, options)?;
        if !rustycad_tkgeomalgo::app_blend::AppBlendApprox::is_done(&approx) {
            return Ok(false);
        }
        self.store_data(data, &approx, lin, faces, or1, g, reversed)
    }
}
