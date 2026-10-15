// Rust translation of the writing of the pieces of ChFi3d_Builder::PerformIntersectionAtEnd
// into the data structure (ChFi3d_Builder_C1.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Each piece of the cut written into the data structure with its points, the pieces of
//! the edges prolonged between the faces, and the pieces where a side of the fillet
//! shrinks to a point.
use super::ChFi3dBuilder;
use super::at_end::AtEnd;
use super::at_end_cut::CutLoop;
use super::corner_tools::compute_curve_2d;
use super::find_face::{contain_v, is_shrink};
use super::one_corner::value_2d;
use crate::chfi_ds::lock;
use crate::chfi3d::{compute_pcurv_geom, eval_tol_reached, fil_curve_in_ds, fil_point_in_ds};
use rustycad_tkbool::top_ope_brep_ds::{TopOpeBRepDSCurve, TopOpeBRepDSPoint};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomBoundedCurve, GeomCurve, GeomTrimmedCurve};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkg3d::top_abs::{self, Orientation};
use rustycad_tkgeomalgo::geom2d_api::ProjectPointOnCurve;
use rustycad_tkgeombase::extrema::ExtPC;
use rustycad_tkgeombase::geom_lib::extend_curve_to_point;

impl ChFi3dBuilder {
    /// The tolerances of the ends of the piece: the gaps between the ends of its pcurves on
    /// the fillet and the face and the ends asked for.
    fn piece_tolerances(ae: &AtEnd, cl: &mut CutLoop) -> Result<(f64, f64), Failure> {
        let hgs = ae.hgs.clone();
        let hbs = ae.hbs.clone();
        let (pc, ps) = (
            cl.pc
                .get()
                .ok_or_else(|| Failure::new("IntersectionAtEnd"))?,
            cl.ps
                .get()
                .ok_or_else(|| Failure::new("IntersectionAtEnd"))?,
        );
        let p1 = hgs.value(cl.pfil1.x(), cl.pfil1.y())?;
        let p2 = hgs.value(cl.pfil2.x(), cl.pfil2.y())?;
        let p3 = hbs.value(cl.pfac1.x(), cl.pfac1.y())?;
        let p4 = hbs.value(cl.pfac2.x(), cl.pfac2.y())?;
        let (a, b) = (pc.value(cl.p1)?, pc.value(cl.p2)?);
        let p7 = hbs.value(a.x(), a.y())?;
        let p8 = hbs.value(b.x(), b.y())?;
        cl.p2d1 = ps.value(cl.p1)?;
        cl.p2d2 = ps.value(cl.p2)?;
        let p5 = hgs.value(cl.p2d1.x(), cl.p2d1.y())?;
        let p6 = hgs.value(cl.p2d2.x(), cl.p2d2.y())?;
        Ok((
            (p1.distance(&p5) + p3.distance(&p7)).max(cl.tolreached),
            (p2.distance(&p6) + p4.distance(&p8)).max(cl.tolreached),
        ))
    }

    /// The tolerance of a point of the structure raised to `tol`.
    fn raise_point_tolerance(&mut self, ip: i32, tol: f64) -> Result<(), Failure> {
        let p = self.ds.change_point(ip)?;
        let t = p.tolerance().max(tol);
        p.set_tolerance(t);
        Ok(())
    }

    /// The piece written: its points, its curve and its interferences on the edges at its
    /// ends; whether its ends are vertices.
    pub(super) fn write_piece(
        &mut self,
        ae: &AtEnd,
        cl: &mut CutLoop,
        nb: usize,
    ) -> Result<(bool, bool), Failure> {
        let (to1, to2) = Self::piece_tolerances(ae, cl)?;
        let (mut isvtx1, mut isvtx2) = (false, false);
        let (cv1, cv2) = (ae.cv(1), ae.cv(2));
        if nb == 1 {
            cl.indpoint1 = lock(&ae.stripe).index_point(ae.isfirst, 1);
            if !cv1.is_vertex() {
                self.raise_point_tolerance(cl.indpoint1, to1)?;
            } else {
                isvtx1 = true;
            }
        }
        let c = cl
            .cc
            .get()
            .ok_or_else(|| Failure::new("IntersectionAtEnd"))?;
        if nb == ae.nbface {
            cl.indpoint2 = lock(&ae.stripe).index_point(ae.isfirst, 2);
            if !cv2.is_vertex() {
                self.raise_point_tolerance(cl.indpoint2, to2)?;
            } else {
                isvtx2 = true;
            }
        } else {
            let point = c.value(c.last_parameter())?;
            cl.indpoint2 = self.ds.add_point(&TopOpeBRepDSPoint::new(point, to2));
        }
        if nb != 1 {
            self.raise_point_tolerance(cl.indpoint1, to1)?;
        }
        let ic = self
            .ds
            .add_curve(&mut TopOpeBRepDSCurve::new(&cl.cc, cl.tolreached, false));
        cl.indcurve[nb - 1] = ic;
        let i1 = fil_point_in_ds(
            Orientation::Forward,
            ic,
            cl.indpoint1,
            c.first_parameter(),
            isvtx1,
        );
        let i2 = fil_point_in_ds(
            Orientation::Reversed,
            ic,
            cl.indpoint2,
            c.last_parameter(),
            isvtx2,
        );
        self.ds.change_curve_interferences(ic).append(i1);
        self.ds.change_curve_interferences(ic).append(i2);
        if nb == 1 {
            let iarc1 = self.ds.add_shape(ae.edges[0].as_ref());
            cl.interfedge[0] = fil_point_in_ds(
                cv1.transition_on_arc()?,
                iarc1,
                cl.indpoint1,
                cl.paredge1,
                isvtx1,
            );
        }
        if nb == ae.nbface {
            let iarc2 = self.ds.add_shape(ae.edges[nb].as_ref());
            cl.interfedge[nb] = fil_point_in_ds(
                cv2.transition_on_arc()?,
                iarc2,
                cl.indpoint2,
                cl.paredge2,
                isvtx2,
            );
        }
        Ok((isvtx1, isvtx2))
    }

    /// The orientation of the edge `nb` between the faces from the vertex.
    fn prolonged_edge_orientation(&self, ae: &AtEnd, nb: usize) -> Result<Orientation, Failure> {
        let e = &ae.edges[nb];
        if ae.nface != 3 {
            return Ok(if topexp::first_vertex(e, false).is_same(&ae.vtx) {
                Orientation::Forward
            } else {
                Orientation::Reversed
            });
        }
        let (v1, v2) = (
            topexp::first_vertex(e, false),
            topexp::last_vertex(e, false),
        );
        let mut ori = if contain_v(&ae.f1, &v1)? || contain_v(&ae.f2, &v1)? {
            Orientation::Forward
        } else if contain_v(&ae.f1, &v2)? || contain_v(&ae.f2, &v2)? {
            Orientation::Reversed
        } else {
            return Err(Failure::new("IntersectionAtEnd : pb orientation"));
        };
        let pv = BRepTool::pnt(&ae.vtx)?;
        for f in [&ae.f1, &ae.f2] {
            if contain_v(f, &v1)? && contain_v(f, &v2)? {
                let (d1, d2) = (
                    BRepTool::pnt(&v1)?.distance(&pv),
                    BRepTool::pnt(&v2)?.distance(&pv),
                );
                ori = if d1 < d2 {
                    Orientation::Forward
                } else {
                    Orientation::Reversed
                };
            }
        }
        Ok(ori)
    }

    /// The curve of the prolonged edge: the intersection of the extended faces, or the
    /// edge saved (extended to the end of the fillet when it was).
    fn prolonged_curve(
        ae: &AtEnd,
        cl: &CutLoop,
        tolpt: f64,
    ) -> Result<Handle<dyn GeomCurve>, Failure> {
        let one = ae.oneintersection[0] || ae.oneintersection[1];
        if !one {
            return Ok(cl.cint.clone());
        }
        let (mut csau, _, _) = BRepTool::curve(&ae.edgesau)?;
        let mut c1: Handle<dyn GeomBoundedCurve> = csau.clone().cast();
        if cl.extend && !c1.is_null() {
            let pext = BRepTool::pnt(&ae.vtx)?;
            let target = if ae.oneintersection[0] {
                ae.cv(1).point()
            } else {
                ae.cv(2).point()
            };
            let c = c1.get().ok_or_else(|| Failure::new("IntersectionAtEnd"))?;
            let pl = c.value(c.last_parameter())?;
            let sens = pl.distance(&pext) < tolpt;
            extend_curve_to_point(&mut c1, target, 1, sens)?;
            csau = c1.cast();
        }
        Ok(csau)
    }

    /// The edge between the faces prolonged to the end of the piece: its curve from the
    /// vertex to the point of the piece, written with its pcurves on the faces around it.
    #[allow(clippy::too_many_lines)]
    fn write_prolonged_edge(
        &mut self,
        ae: &AtEnd,
        cl: &mut CutLoop,
        nb: usize,
        orien: Orientation,
    ) -> Result<(), Failure> {
        let one = ae.oneintersection[0] || ae.oneintersection[1];
        if !one {
            cl.proledge[nb] = true;
        }
        let tolpt = 1.0e-4;
        let pext = BRepTool::pnt(&ae.vtx)?;
        let csau = Self::prolonged_curve(ae, cl, tolpt)?;
        let cad = GeomAdaptorCurve::with_curve(&csau)?;
        let ext = ExtPC::with_curve(pext, &cad as &dyn Adaptor3dCurve, tolpt)?;
        let (mut par1, mut par2);
        par1 = ext.point(1)?.parameter();
        let par_vtx = par1;
        let ind;
        if one {
            let (p, i) = if ae.oneintersection[1] {
                (ae.cv(2).point(), cl.indpoint2)
            } else {
                (ae.cv(1).point(), cl.indpoint1)
            };
            ind = i;
            let ext2 = ExtPC::with_curve(p, &cad as &dyn Adaptor3dCurve, tolpt)?;
            par2 = ext2.point(1)?.parameter();
        } else {
            par2 = cl.paredge2;
            ind = cl.indpoint2;
        }
        let (indp1, indp2, vtx1, vtx2);
        if par1 > par2 {
            indp1 = ind;
            indp2 = self.ds.add_shape(ae.vtx.as_ref());
            (vtx1, vtx2) = (false, true);
            std::mem::swap(&mut par1, &mut par2);
        } else {
            indp1 = self.ds.add_shape(ae.vtx.as_ref());
            indp2 = ind;
            (vtx1, vtx2) = (true, false);
        }
        let ct: Handle<dyn GeomCurve> =
            Handle::new(GeomTrimmedCurve::new(&csau, par1, par2, true, true)?).cast();
        let c = cl
            .cc
            .get()
            .ok_or_else(|| Failure::new("IntersectionAtEnd"))?;
        let ctr = ct.get().ok_or_else(|| Failure::new("IntersectionAtEnd"))?;
        let (pp1, pp2) = (c.value(c.first_parameter())?, c.value(c.last_parameter())?);
        let (pp3, pp4) = (
            ctr.value(ctr.first_parameter())?,
            ctr.value(ctr.last_parameter())?,
        );
        let mut orient = if pp2.distance(&pp3) < tolpt || pp1.distance(&pp4) < tolpt {
            orien
        } else {
            top_abs::reverse(orien)
        };
        let mut indice;
        if one {
            indice = self.ds.add_shape(ae.faces[0].as_ref());
            if cl.extend {
                self.ds
                    .set_new_surface(ae.faces[0].as_ref(), &cl.sfacemoins1);
                compute_curve_2d(&ct, &cl.faceprol[0], &mut cl.c2dint1)?;
            } else {
                let mut le = ae.edgesau.clone();
                if ae.edgesau.orientation() != orient {
                    le.reverse();
                }
                cl.c2dint1 = BRepTool::curve_on_face(&le, &ae.faces[0])?.pcurve;
            }
        } else {
            indice = self.ds.add_shape(ae.faces[nb - 1].as_ref());
            self.ds
                .set_new_surface(ae.faces[nb - 1].as_ref(), &cl.sfacemoins1);
        }
        if cad.is_periodic()? && !cl.c2dint1.is_null() {
            let p2d = BRepTool::parameters(&ae.vtx, &ae.faces[0])?;
            let par =
                ProjectPointOnCurve::with_curve(p2d, &cl.c2dint1)?.lower_distance_parameter()?;
            let shift = par - par_vtx;
            if shift.abs() > precision::confusion() {
                par1 += shift;
                par2 += shift;
            }
        }
        let ct: Handle<dyn GeomCurve> =
            Handle::new(GeomTrimmedCurve::new(&csau, par1, par2, true, true)?).cast();
        if one {
            cl.tolex = 10. * BRepTool::edge_tolerance(&ae.edgesau)?;
        }
        if cl.extend {
            let h1: Handle<dyn Adaptor3dSurface> =
                Handle::new(GeomAdaptorSurface::with_surface(&cl.sfacemoins1)?).cast();
            let reached = if cl.sface.is_null() {
                eval_tol_reached((&h1, &cl.c2dint1), (&h1, &cl.c2dint1), &ct)?
            } else {
                let h2: Handle<dyn Adaptor3dSurface> =
                    Handle::new(GeomAdaptorSurface::with_surface(&cl.sface)?).cast();
                eval_tol_reached((&h1, &cl.c2dint1), (&h2, &cl.c2dint2), &ct)?
            };
            cl.tolex = cl.tolex.max(reached);
        }
        let indcurv = self
            .ds
            .add_curve(&mut TopOpeBRepDSCurve::new(&ct, cl.tolex, false));
        let i1 = fil_point_in_ds(Orientation::Forward, indcurv, indp1, par1, vtx1);
        let i2 = fil_point_in_ds(Orientation::Reversed, indcurv, indp2, par2, vtx2);
        self.ds.change_curve_interferences(indcurv).append(i1);
        self.ds.change_curve_interferences(indcurv).append(i2);
        let i = fil_curve_in_ds(indcurv, indice, &cl.c2dint1, orient);
        self.ds.change_shape_interferences_index(indice)?.append(i);
        if one {
            indice = self.ds.add_shape(ae.facesau.as_ref());
            if ae.facesau.orientation() == ae.faces[0].orientation() {
                orient = top_abs::reverse(orient);
            }
            if cl.extend {
                compute_curve_2d(&ct, &cl.faceprol[1], &mut cl.c2dint2)?;
            } else {
                let mut le = ae.edgesau.clone();
                if ae.edgesau.orientation() != orient {
                    le.reverse();
                }
                cl.c2dint2 = BRepTool::curve_on_face(&le, &ae.facesau)?.pcurve;
            }
        } else {
            indice = self.ds.add_shape(ae.faces[nb].as_ref());
            self.ds.set_new_surface(ae.faces[nb].as_ref(), &cl.sface);
            if ae.faces[nb].orientation() == ae.faces[nb - 1].orientation() {
                orient = top_abs::reverse(orient);
            }
        }
        if !ae.bordlibre {
            let i = fil_curve_in_ds(indcurv, indice, &cl.c2dint2, orient);
            self.ds.change_shape_interferences_index(indice)?.append(i);
        }
        Ok(())
    }

    /// The interference of the end of the piece on the edge to the next face, or the edge
    /// prolonged when the faces were extended.
    pub(super) fn write_next_edge(
        &mut self,
        ae: &AtEnd,
        cl: &mut CutLoop,
        nb: usize,
        orien: Orientation,
    ) -> Result<(), Failure> {
        let one = ae.oneintersection[0] || ae.oneintersection[1];
        if !(nb != ae.nbface || one) {
            return Ok(());
        }
        let ori = self.prolonged_edge_orientation(ae, nb)?;
        if !cl.extend && !one {
            let iarc2 = self.ds.add_shape(ae.edges[nb].as_ref());
            cl.interfedge[nb] = fil_point_in_ds(ori, iarc2, cl.indpoint2, cl.paredge2, false);
            return Ok(());
        }
        self.write_prolonged_edge(ae, cl, nb, orien)
    }

    /// A piece along an iso where a side of the fillet shrinks: its pcurve put on the
    /// surface data before, at the parameter of its end.
    pub(super) fn shrink_piece(
        &mut self,
        ae: &mut AtEnd,
        cl: &mut CutLoop,
        nb: usize,
    ) -> Result<(), Failure> {
        let pps = Geom2dAdaptorCurve::with_curve(&cl.ps)?;
        if !(ae.check_shrink
            && is_shrink(
                &pps,
                (cl.p1, cl.p2),
                ae.check_shr_param,
                ae.is_u_shrink,
                precision::parametric(cl.tolreached),
            )?)
        {
            return Ok(());
        }
        cl.shrink[nb - 1] = true;
        let sdprev = ae.sdprev.clone().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "IntersectionAtEnd: no previous surface data",
            )
        })?;
        if !ae.is_shrink {
            let g = lock(&sdprev);
            let fi = g.interference(1);
            let uv = value_2d(fi.pcurve_on_surf(), fi.parameter(ae.isfirst))?;
            ae.prev_sd_param = if ae.is_u_shrink { uv.x() } else { uv.y() };
        }
        let (mut uv1, mut uv2) = (cl.p2d1, cl.p2d2);
        if ae.is_u_shrink {
            uv1.set_x(ae.prev_sd_param);
            uv2.set_x(ae.prev_sd_param);
        } else {
            uv1.set_y(ae.prev_sd_param);
            uv2.set_y(ae.prev_sd_param);
        }
        let surf = self.ds.surface(lock(&sdprev).surf()).surface().clone();
        let mut reached = 0.;
        cl.ps = compute_pcurv_geom(
            &cl.cc,
            (uv1, uv2),
            &surf,
            (cl.p1, cl.p2),
            self.tolapp3d,
            &mut reached,
            false,
        )?;
        let ic = cl.indcurve[nb - 1];
        let t = self.ds.change_curve(ic).tolerance().max(reached);
        self.ds.change_curve(ic).set_tolerance(t);
        cl.interf_ps[nb - 1] = fil_curve_in_ds(ic, ae.isurf_prev, &cl.ps, ae.orcourbe);
        let i = cl.interf_ps[nb - 1].clone();
        self.ds
            .change_surface_interferences(ae.isurf_prev)
            .append(i);
        if ae.is_on_same[1] {
            ae.mid_p2d = cl.p2d2;
            ae.mid_ipoint = cl.indpoint2;
        } else if !ae.is_shrink {
            ae.mid_p2d = cl.p2d1;
            ae.mid_ipoint = cl.indpoint1;
        }
        ae.is_shrink = true;
        Ok(())
    }

    /// The interferences of the piece on the face and the fillet (written after the
    /// loop).
    pub(super) fn piece_interferences(
        &mut self,
        ae: &AtEnd,
        cl: &mut CutLoop,
        nb: usize,
        orien: Orientation,
    ) {
        let indice = self.ds.add_shape(ae.faces[nb - 1].as_ref());
        cl.interf_pc[nb - 1] = fil_curve_in_ds(cl.indcurve[nb - 1], indice, &cl.pc, orien);
        if !cl.shrink[nb - 1] {
            cl.interf_ps[nb - 1] =
                fil_curve_in_ds(cl.indcurve[nb - 1], ae.isurf, &cl.ps, ae.orcourbe);
        }
        cl.indpoint1 = cl.indpoint2;
    }
}
