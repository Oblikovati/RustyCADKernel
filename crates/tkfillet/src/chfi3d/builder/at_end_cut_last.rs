// Rust translation of the loop over the faces of ChFi3d_Builder::PerformIntersectionAtEnd
// (ChFi3d_Builder_C1.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The last end of a piece of fillet cut by the faces around the vertex, and the cut by one face.
use super::ChFi3dBuilder;
use super::at_end::AtEnd;
use super::at_end_cut::CutLoop;
use super::at_end_cut::surface_parameters;
use super::corner_tools::{extend_surface, inters_update_on_same};
use super::one_corner::value_2d;
use crate::chfi_ds::lock;
use crate::chfi3d::{IntersectionCurves, boite, bound_fac, compute_curves};
use rustycad_tkbool::top_ope_brep_ds::TopOpeBRepDSPoint;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::geom2d_api::ProjectPointOnCurve;

impl ChFi3dBuilder {
    /// The last end of the cut on the side 2 of the fillet.
    pub(super) fn last_end(
        &self,
        ae: &AtEnd,
        cl: &mut CutLoop,
        nb: usize,
        f: &Face,
    ) -> Result<(), Failure> {
        let e2 = &ae.edges[nb];
        let hc2 = BRepTool::curve_on_face(e2, &ae.faces[ae.nbface - 1])?.pcurve;
        if hc2.is_null() {
            extend_surface(&mut cl.sfacemoins1, &mut cl.prolface[0])?;
            if cl.prolface[0] != 0 {
                cl.extend = true;
                let tol = BRepTool::face_tolerance(f)?;
                BRepBuilder::default().make_face_on_located_surface(
                    &mut cl.faceprol[nb - 1],
                    &cl.sfacemoins1,
                    f.location(),
                    tol,
                )?;
                surface_parameters(&cl.sfacemoins1, ae.cv(2).point(), tol, &mut cl.pfac2)?;
            }
        } else {
            cl.pfac2 = value_2d(&hc2, ae.cv(2).parameter_on_arc()?)?;
        }
        cl.paredge2 = ae.cv(2).parameter_on_arc()?;
        let fi2 = &ae.fi[1];
        let t = if fi2.line_index() != 0 {
            fi2.parameter(ae.isfirst)
        } else {
            fi2.parameter(!ae.isfirst)
        };
        cl.pfil2 = value_2d(fi2.pcurve_on_surf(), t)?;
        Ok(())
    }

    /// The end of the side 2 OnSame: the arc of the last face crossed by the fillet.
    pub(super) fn last_end_on_same(
        &mut self,
        ae: &mut AtEnd,
        cl: &mut CutLoop,
        nb: usize,
        f: &Face,
    ) -> Result<(), Failure> {
        self.init_cut_face(ae, cl, nb, true)?;
        let c3df = self.ds.curve(ae.fi[1].line_index()).curve().clone();
        let mut ufi = ae.fi[0].parameter(ae.isfirst);
        let (hgs, hbs): (Handle<dyn Adaptor3dSurface>, Handle<dyn Adaptor3dSurface>) =
            (ae.hgs.clone().cast(), ae.hbs.clone().cast());
        let ok = {
            let mut g = lock(&ae.fd);
            let (mut fi, mut cp) = (g.interference(2).clone(), g.vertex(ae.isfirst, 2).clone());
            let r = inters_update_on_same(
                &hgs,
                &hbs,
                &c3df,
                (&ae.f2, f, &ae.edges[nb], &ae.vtx),
                ae.isfirst,
                10. * self.tolapp3d,
                (&mut fi, &mut cp),
                &mut cl.pfac2,
                &mut ufi,
            )?;
            *g.change_interference(2) = fi.clone();
            *g.change_vertex(ae.isfirst, 2) = cp;
            ae.fi[1] = fi;
            r
        };
        if !ok {
            return Err(Failure::new("IntersectionAtEnd: pb intersection Face - Fi"));
        }
        if cl.inters_on_same_failed {
            let c = if cl.extend {
                cl.c2dint2.clone()
            } else {
                BRepTool::curve_on_face(&ae.edges[nb], &ae.faces[ae.nbface - 1])?.pcurve
            };
            cl.paredge2 =
                ProjectPointOnCurve::with_curve(cl.pfac2, &c)?.lower_distance_parameter()?;
        }
        let cv2 = ae.cv(2);
        cl.indpoint2 = self
            .ds
            .add_point(&TopOpeBRepDSPoint::new(cv2.point(), self.tolapp3d));
        lock(&ae.stripe).set_index_point(cl.indpoint2, ae.isfirst, 2);
        let (vert1, _) = topexp::vertices(&ae.edges[ae.nbface], false);
        let arc_ori = if ae.vtx.is_same(&vert1) {
            Orientation::Forward
        } else {
            Orientation::Reversed
        };
        let (tol, e, pe) = (self.tolapp3d, ae.edges[ae.nbface].clone(), cl.paredge2);
        ae.change_cv(2, |c| c.set_arc(tol, &e, pe, arc_ori));
        Ok(())
    }

    /// The piece of the cut by the face `nb`: the intersection curve between its ends;
    /// false when it fails (the corner is then left to `PerformMoreThreeCorner`).
    pub(super) fn cut_piece(
        &mut self,
        ae: &AtEnd,
        cl: &mut CutLoop,
        nb: usize,
    ) -> Result<bool, Failure> {
        self.init_cut_face(ae, cl, nb, true)?;
        let pardeb = [cl.pfil1.x(), cl.pfil1.y(), cl.pfac1.x(), cl.pfac1.y()];
        let parfin = [cl.pfil2.x(), cl.pfil2.y(), cl.pfac2.x(), cl.pfac2.y()];
        bound_fac(&ae.hbs, boite(cl.pfac1, cl.pfac2), true)?;
        let (hgs, hbs): (Handle<dyn Adaptor3dSurface>, Handle<dyn Adaptor3dSurface>) =
            (ae.hgs.clone().cast(), ae.hbs.clone().cast());
        let mut out = IntersectionCurves::default();
        if !compute_curves(
            (&hgs, &hbs),
            (pardeb, parfin),
            (self.tolapp3d, self.tol2d),
            ae.nbface == 1,
            &mut out,
        )? {
            return Ok(false);
        }
        cl.cc = out.c3d;
        cl.ps = out.pc1;
        cl.pc = out.pc2;
        cl.tolreached = out.tolreached;
        let c = cl
            .cc
            .get()
            .ok_or_else(|| Failure::new("IntersectionAtEnd"))?;
        cl.p1 = c.first_parameter();
        cl.p2 = c.last_parameter();
        Ok(true)
    }

    /// One face of the loop: the ends of the piece and the piece; false when the corner
    /// is left to `PerformMoreThreeCorner`.
    pub(super) fn cut_by_face(
        &mut self,
        ae: &mut AtEnd,
        cl: &mut CutLoop,
        nb: usize,
    ) -> Result<Option<bool>, Failure> {
        cl.extend = false;
        let f = if nb == 0 {
            ae.f1.clone()
        } else {
            if cl.prolface[nb - 1] == 0 {
                cl.faceprol[nb - 1] = ae.faces[nb - 1].clone();
            }
            ae.faces[nb - 1].clone()
        };
        if f.is_null() {
            return Err(Failure::with_message(
                "Standard_NullObject",
                "IntersectionAtEnd : Trying to intersect with NULL face",
            ));
        }
        cl.sfacemoins1 = BRepTool::surface(&f)?;
        cl.cint = Handle::default();
        cl.c2dint1 = Handle::default();
        cl.c2dint2 = Handle::default();
        if nb == 1 {
            self.first_end(ae, cl, &f)?;
        } else {
            cl.pfil1 = cl.pfil2;
            cl.paredge1 = cl.paredge2;
            cl.pfac1 = cl.pint;
        }
        if nb != ae.nbface || ae.is_on_same[1] {
            if !self.edge_end(ae, cl, nb, &f)? {
                return Ok(Some(false));
            }
        } else {
            self.last_end(ae, cl, nb, &f)?;
        }
        if nb == 0 {
            return Ok(None);
        }
        if nb == ae.nbface && ae.is_on_same[1] {
            self.last_end_on_same(ae, cl, nb, &f)?;
        }
        if !self.cut_piece(ae, cl, nb)? {
            return Ok(Some(false));
        }
        Ok(Some(true))
    }
}
