// Rust translation of the corner surface of ChFi3d_FilBuilder::PerformThreeCorner
// (ChFi3d_FilBuilder_C3.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The surface of a corner of three fillets: on the face of the pivot and the fillet of
//! the pivot, a torus, a sphere, a fillet walked along a spine joining the two
//! neighbours, or a filling.
use super::ChFi3dFilBuilder;
use super::three_corner::{SpineEnd, ThreeCorner, general};
use crate::blend::{BlendFuncInv, BlendFunction};
use crate::blend_func::{ConstRad, ConstRadInv, EvolRad, EvolRadInv};
use crate::brep_blend::{SharedAppFunction, WalkingDomain};
use crate::chfi_ds::{
    ElSpine, Shared, Stripe, StripeHandle, SurfData, SurfDataHandle, lock, shared,
};
use crate::chfi3d::builder::{ComputeCall, LineFunction, SurfOut, WalkSettings};
use crate::chfi3d::{
    circular_spine, mkbound_geom, mkbound_segment, mkbound_tangents_2d, next_side, spine,
};
use crate::chfikpart::ChFiKPartComputeData;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface, Adaptor3dTopolTool};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeomalgo::geom_fill::{GeomFillBoundary, GeomFillConstrainedFilling};
use rustycad_tkgeomalgo::law::{LawFunction, LawS};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point2, Vector2};
use rustycad_tkmath::math::Vector;
use std::rc::Rc;

/// The surfaces of the corner: the face of the pivot, the fillet of the pivot (bounded
/// around the corner), their domains, and the side of the fillet.
pub(super) struct CornerSurfaces {
    pub fac: Handle<BRepAdaptorSurface>,
    pub surf: Handle<GeomAdaptorSurface>,
    pub ifac: WalkingDomain,
    pub isurf: WalkingDomain,
    pub choix: i32,
    /// The orientations of the face and of the fillet of the pivot after `NextSide`.
    pub o1: rustycad_tkg3d::top_abs::Orientation,
    pub o2: rustycad_tkg3d::top_abs::Orientation,
    pub pfac: [(Point2, Vector2); 2],
    pub psurf: [Point2; 2],
}

/// The corner: the new contour with its surface data.
pub(super) struct ThreeCornerData {
    pub corner: StripeHandle,
    pub coin: SurfDataHandle,
}

impl ChFi3dFilBuilder {
    /// The fillet of the pivot as a surface adaptor bounded around the corner, enlarged
    /// on a cylinder (in V) or a torus (in U), whole on a Bezier or B-spline surface.
    fn pivot_surface(&self, tc: &ThreeCorner) -> Result<Handle<GeomAdaptorSurface>, Failure> {
        let fdpiv = tc.sd(tc.pivot, tc.deb)?;
        let (ppp1, ppp2, s) = {
            let g = lock(&fdpiv);
            let b1 = g.interference_on_s1();
            let b2 = g.interference_on_s2();
            (
                b1.pcurve_on_surf().value(b1.first_parameter())?,
                b2.pcurve_on_surf().value(b2.last_parameter())?,
                self.base.ds.surface(g.surf()).surface().clone(),
            )
        };
        let (mut uu1, mut uu2, mut vv1, mut vv2) = (ppp1.x(), ppp2.x(), ppp1.y(), ppp2.y());
        let gasurf = GeomAdaptorSurface::with_bounds(&s, (uu1, uu2, vv1, vv2))?;
        match gasurf.get_type()? {
            SurfaceType::Cylinder => {
                let h = vv2 - vv1;
                vv1 -= 0.5 * h;
                vv2 += 0.5 * h;
                gasurf.load_bounds(&s, (uu1, uu2, vv1, vv2))?;
            }
            SurfaceType::Torus => {
                let h = uu2 - uu1;
                uu1 -= 0.1 * h;
                uu2 += 0.1 * h;
                gasurf.load_bounds(&s, (uu1, uu2, vv1, vv2))?;
            }
            SurfaceType::BezierSurface | SurfaceType::BSplineSurface => gasurf.load(&s)?,
            _ => {}
        }
        Ok(Handle::new(gasurf))
    }

    /// The surfaces of the corner and the side of the fillet on them.
    fn corner_surfaces(&self, tc: &ThreeCorner) -> Result<CornerSurfaces, Failure> {
        let (pivot, deb, fin) = (tc.pivot, tc.deb, tc.fin);
        let fac = Handle::new(BRepAdaptorSurface::with_face(&tc.face[pivot], true)?);
        let surf = self.pivot_surface(tc)?;
        let bidsurf: Handle<dyn Adaptor3dSurface> = Handle::new(GeomAdaptorSurface::with_surface(
            &fac.transformed_surface().geom_surface_original(),
        )?)
        .cast();
        let ifac: WalkingDomain = Rc::new(Adaptor3dTopolTool::with_surface(&bidsurf)?);
        let isurf: WalkingDomain = Rc::new(Adaptor3dTopolTool::with_surface(&surf.clone().cast())?);
        let fdpiv = tc.sd(pivot, deb)?;
        let (mut choix, os1, os2) = {
            let s = lock(&tc.cd[deb]);
            (
                s.choix(),
                s.orientation_on_face1(),
                s.orientation_on_face2(),
            )
        };
        let mut o1 = tc.face[pivot].orientation();
        let mut o2 = lock(&fdpiv).orientation();
        let flip = |c: i32| if c % 2 == 1 { c + 1 } else { c - 1 };
        if tc.jf[deb][fin] == 1 {
            choix = next_side(&mut o1, &mut o2, os1, os2, choix);
            if tc.sens[deb] == 1 {
                choix = flip(choix);
            }
        } else {
            choix = next_side(&mut o2, &mut o1, os1, os2, -choix);
            if tc.sens[deb] == -1 {
                choix = flip(choix);
            }
        }
        let fac_d1 = |j: usize, k: usize| -> Result<(Point2, Vector2), Failure> {
            let sd = tc.sd(j, tc.pivot)?;
            let pc = lock(&sd).interference(tc.jf[j][k]).pcurve_on_face().clone();
            pc.d1(tc.p[j][tc.pivot])
        };
        let surf_pt = |k: usize| -> Result<Point2, Failure> {
            let sd = tc.sd(pivot, k)?;
            let pc = lock(&sd)
                .interference(tc.jf[pivot][k])
                .pcurve_on_surf()
                .clone();
            pc.value(tc.p[pivot][k])
        };
        Ok(CornerSurfaces {
            fac,
            surf,
            ifac,
            isurf,
            choix,
            o1,
            o2,
            pfac: [fac_d1(deb, fin)?, fac_d1(fin, deb)?],
            psurf: [surf_pt(deb)?, surf_pt(fin)?],
        })
    }

    /// The corner surface (torus, sphere, walking or filling), then the corner written
    /// and the surface data of the contours beyond the corner removed.
    pub(super) fn three_corner_surface(
        &mut self,
        tc: &mut ThreeCorner,
        ends: &[SpineEnd; 4],
    ) -> Result<(), Failure> {
        let cs = self.corner_surfaces(tc)?;
        let coin = shared(SurfData::new());
        let mut stripe = Stripe::new();
        *stripe.change_set_of_surf_data() = Some(shared(vec![coin.clone()]));
        let data = ThreeCornerData {
            corner: shared(stripe),
            coin,
        };
        let [edeb, efin, edp, _] = *ends;
        let (gfac, gsurf) = (general(&cs.fac), cs.surf.clone().cast());
        let ofac = tc.face[tc.pivot].orientation();
        let mut done = false;
        if tc.c1toric {
            done = ChFiKPartComputeData::compute_corner_torus(
                &mut self.base.ds,
                &data.coin,
                (&gfac, &gsurf),
                (ofac, cs.o1, cs.o2),
                (edeb.r, edp.r),
                [cs.pfac[0].0, cs.pfac[1].0, cs.psurf[0], cs.psurf[1]],
            )?;
        } else if tc.c1spheric {
            done = ChFiKPartComputeData::compute_corner_sphere(
                &mut self.base.ds,
                &data.coin,
                (&gfac, &gsurf),
                (ofac, cs.o1),
                edp.r,
                (cs.pfac[0].0, cs.psurf[0], cs.psurf[1]),
            )?;
        } else if tc.c1pointu {
            tc.filling = true;
        }
        self.base.done = done;
        if !self.base.done {
            if !tc.filling {
                tc.filling = !self.walk_corner(tc, &cs, &data, (edeb, efin, ends[2], ends[3]))?;
            }
            if tc.filling {
                self.base.done = self.fill_three_corner(tc, &cs, &data)?;
            }
        }
        let (pivot, deb, fin) = (tc.pivot, tc.deb, tc.fin);
        if !tc.c1pointu {
            tc.p[deb][fin] = tc.p[deb][pivot];
            tc.p[fin][deb] = tc.p[fin][pivot];
        }
        if self.base.done {
            self.write_three_corner(tc, &data)?;
        }
        self.trim_three_contours(tc)?;
        let isurf = lock(&data.coin).surf();
        self.append_evi(&tc.vtx, isurf);
        let solid = lock(&tc.cd[pivot]).solid_index();
        lock(&data.corner).set_solid_index(solid);
        self.base.my_list_stripe.push(data.corner);
        Ok(())
    }

    /// The fillet walked along a circular (or Bezier) spine joining the ends of the two
    /// neighbours, of constant radius or evolving from one to the other; false when the
    /// walking does not reach the end on the fillet of the pivot.
    fn walk_corner(
        &mut self,
        tc: &ThreeCorner,
        cs: &CornerSurfaces,
        data: &ThreeCornerData,
        (edeb, efin, edp, efp): (SpineEnd, SpineEnd, SpineEnd, SpineEnd),
    ) -> Result<bool, Failure> {
        let radpondere = (edp.r + efp.r) / 2.;
        let mut locfleche = self.base.fleche;
        let (spinecoin, wfirst, wlast): (Handle<dyn GeomCurve>, f64, f64) =
            match circular_spine(edeb.p, edeb.v, efin.p, efin.v, radpondere)? {
                Some((c, wf, wl)) => {
                    locfleche = radpondere * (wl - wf) * self.base.fleche;
                    (c.cast(), wf, wl)
                }
                None => {
                    let (mut vd, mut vf) = (edeb.v, efin.v);
                    (
                        spine(edeb.p, &mut vd, efin.p, &mut vf, radpondere)?.cast(),
                        0.,
                        1.,
                    )
                }
            };
        let pasmax = (wlast - wfirst) * 0.05;
        let cornerspine = Handle::new(ElSpine::new());
        cornerspine.set_curve(&spinecoin)?;
        cornerspine.set_first_parameter(wfirst - pasmax);
        cornerspine.set_last_parameter(wlast + pasmax);
        let mut soldep = Vector::with_bounds(1, 4)?;
        let (pf, ps) = (cs.pfac[0].0, cs.psurf[0]);
        soldep[1] = pf.x();
        soldep[2] = pf.y();
        soldep[3] = ps.x();
        soldep[4] = ps.y();
        let (gfac, gsurf): (Handle<dyn Adaptor3dSurface>, Handle<dyn Adaptor3dSurface>) =
            (general(&cs.fac), cs.surf.clone().cast());
        let guide: Handle<dyn Adaptor3dCurve> = cornerspine.clone().cast();
        let tol_guide = cornerspine.resolution(self.base.tolapp3d)?;
        let ccall = ComputeCall {
            hguide: &cornerspine,
            spine: None,
            s1: &gfac,
            i1: &cs.ifac,
            s2: &gsurf,
            i2: &cs.isurf,
            pfirst: wfirst,
            walk: WalkSettings {
                max_step: pasmax,
                fleche: locfleche,
                tol_guide,
                inside: false,
                appro: false,
                forward: true,
            },
            soldep: &soldep,
            rec: (false, true),
        };
        let mut out = SurfOut {
            first: wfirst,
            last: wlast + pasmax,
            intf: 3,
            intl: 3,
        };
        let ofac = tc.face[tc.pivot].orientation();
        if (edeb.r - efin.r).abs() <= self.base.tolapp3d {
            let mut func = ConstRad::new(&gfac, &gsurf, &guide)?;
            let mut finv = ConstRadInv::new(&gfac, &gsurf, &guide);
            func.set_radius(edeb.r, cs.choix);
            func.set_shape(self.section_shape);
            finv.set_radius(edeb.r, cs.choix);
            self.walk_corner_with(data, &ccall, shared(func), &mut finv, &mut out, ofac)
        } else {
            let law = LawS::new();
            law.set(wfirst, edeb.r, wlast, efin.r)?;
            let law: Handle<dyn LawFunction> = Handle::new(law).cast();
            let mut func = EvolRad::new(&gfac, &gsurf, &guide, &law)?;
            let mut finv = EvolRadInv::new(&gfac, &gsurf, &guide, &law);
            func.set_choix(cs.choix);
            func.set_shape(self.section_shape);
            finv.set_choix(cs.choix);
            self.walk_corner_with(data, &ccall, shared(func), &mut finv, &mut out, ofac)
        }
    }

    /// `ComputeData` then, when the walking ends on the fillet of the pivot,
    /// `CompleteData`; whether the corner is done.
    fn walk_corner_with<F: BlendFunction + 'static>(
        &mut self,
        data: &ThreeCornerData,
        ccall: &ComputeCall<'_>,
        func: Shared<F>,
        finv: &mut dyn BlendFuncInv,
        out: &mut SurfOut,
        ofac: rustycad_tkg3d::top_abs::Orientation,
    ) -> Result<bool, Failure> {
        let mut lin = None;
        let (done, g) = {
            let mut f = lock(&func);
            self.base
                .compute_data(&data.coin, ccall, &mut lin, (&mut *f, finv), out)?
        };
        self.base.done = done;
        let (Some(lin), true, true) = (lin, done, g[3]) else {
            return Ok(false);
        };
        let app: SharedAppFunction = func.clone();
        let done = self.base.complete_data(
            &data.coin,
            (LineFunction::Surfaces, &app),
            &lin,
            (ccall.s1, ccall.s2),
            ofac,
            [g[0], false, g[2], false],
            false,
        )?;
        self.base.done = done;
        Ok(done)
    }

    /// The filling of the corner: the boundaries on the fillets of the neighbours, on the
    /// fillet of the pivot and (unless pointed) on the face of the pivot.
    fn fill_three_corner(
        &mut self,
        tc: &ThreeCorner,
        cs: &CornerSurfaces,
        data: &ThreeCornerData,
    ) -> Result<bool, Failure> {
        let (pivot, deb, fin) = (tc.pivot, tc.deb, tc.fin);
        let (t3d, ta) = (self.base.tolapp3d, 2.0e-4);
        let (gfac, gsurf): (Handle<dyn Adaptor3dSurface>, Handle<dyn Adaptor3dSurface>) =
            (general(&cs.fac), cs.surf.clone().cast());
        let mut bfac: Option<Handle<dyn GeomFillBoundary>> = None;
        let mut pcurve_on_face: Handle<dyn Geom2dCurve> = Handle::default();
        if !tc.c1pointu {
            let (b, pc) = mkbound_tangents_2d(
                &gfac,
                (tc.sens[deb], cs.pfac[0].0, cs.pfac[0].1),
                (tc.sens[fin], cs.pfac[1].0, cs.pfac[1].1),
                (t3d, ta),
            )?;
            bfac = Some(b);
            pcurve_on_face = pc;
        }
        for j in [deb, fin] {
            let sd = tc.sd(j, pivot)?;
            let kkk = lock(&sd).interference(tc.jf[j][pivot]).line_index();
            let c = self.base.ds.curve(kkk).curve().clone();
            c.d1(tc.p[j][pivot])?;
        }
        let (bpiv, pcurve_on_piv) =
            mkbound_segment(&gsurf, cs.psurf[0], cs.psurf[1], (t3d, ta), false)?;
        let side = |j: usize, k: usize| -> Result<Handle<dyn GeomFillBoundary>, Failure> {
            let par2 = if tc.c1pointu {
                tc.p[j][k]
            } else {
                tc.p[j][pivot]
            };
            let sd = tc.sd(j, pivot)?;
            let g = lock(&sd);
            let p1 = g
                .interference(tc.jf[j][pivot])
                .pcurve_on_surf()
                .value(tc.p[j][pivot])?;
            let p2 = g.interference(tc.jf[j][k]).pcurve_on_surf().value(par2)?;
            let s = self.base.ds.surface(g.surf()).surface().clone();
            mkbound_geom(&s, p1, p2, (t3d, ta), false)
        };
        let (bdeb, bfin) = (side(deb, fin)?, side(fin, deb)?);
        let mut fil = GeomFillConstrainedFilling::new(11, 20);
        match &bfac {
            None => fil.init_three([&bpiv, &bfin, &bdeb], true)?,
            Some(b) => fil.init_four([&bpiv, &bfin, b, &bdeb], true)?,
        }
        let surfcoin = fil.surface();
        surfcoin.v_reverse()?;
        let o = lock(&tc.sd(pivot, deb)?).orientation();
        self.base.complete_data_coin(
            &data.coin,
            &surfcoin.cast(),
            (&gfac, &pcurve_on_face),
            (&gsurf, &pcurve_on_piv),
            o,
            false,
            [false; 4],
        )
    }
}
