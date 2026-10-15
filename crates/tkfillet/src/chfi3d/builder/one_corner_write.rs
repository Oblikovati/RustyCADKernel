// Rust translation of ChFi3d_Builder::PerformOneCorner (ChFi3d_Builder_C1.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The cork of a corner of one contour written in the data structure.
use super::ChFi3dBuilder;
use super::corner_tools::compute_curve_2d;
use super::one_corner::Cork;
use super::one_corner::OneCorner;
use super::one_corner::edge_orientation;
use super::one_corner::pcurve_of;
use super::one_corner::value_2d;
use crate::chfi_ds::{CommonPoint, SurfDataHandle, lock};
use crate::chfi3d::{
    enlarge_box_ds, enlarge_box_edge, enlarge_box_pcurve, fil_curve_in_ds, fil_point_in_ds,
    has_common_endpoint, has_transversal_intersection, index_point_in_ds, is_pseudo_seam,
    set_point_tolerance,
};
use rustycad_tkbool::top_ope_brep_ds::{TopOpeBRepDSCurve, TopOpeBRepDSSurfaceCurveInterference};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomCurve, GeomTrimmedCurve};
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum};
use rustycad_tkgeomalgo::geom2d_api::ProjectPointOnCurve;
use rustycad_tkgeomalgo::geom2d_int::GInter;
use rustycad_tkmath::bnd::BoundingBox3;

impl ChFi3dBuilder {
    /// The cork written into the data structure, the prolongation OnSame, and the boxes of
    /// the ends.
    pub(super) fn write_cork(
        &mut self,
        oc: &OneCorner,
        vtx: &Vertex,
        cork: &mut Cork,
    ) -> Result<(), Failure> {
        let ishape = self.ds.add_shape(oc.fv.as_ref());
        let (cv1, cv2) = {
            let g = lock(&oc.fd);
            (
                g.vertex(oc.isfirst, 1).clone(),
                g.vertex(oc.isfirst, 2).clone(),
            )
        };
        let et = if oc.ifad == 1 {
            match edge_orientation(&oc.fv, cv1.arc()?)? {
                Some(o) => top_abs::reverse(top_abs::compose(o, cv1.transition_on_arc()?)),
                None => Orientation::Forward,
            }
        } else {
            match edge_orientation(&oc.fv, cv2.arc()?)? {
                Some(o) => top_abs::compose(o, cv2.transition_on_arc()?),
                None => Orientation::Forward,
            }
        };
        let i1 = index_point_in_ds(&cv1, &mut self.ds)?;
        let i2 = index_point_in_ds(&cv2, &mut self.ds)?;
        {
            let mut s = lock(&oc.stripe);
            s.set_index_point(i1, oc.isfirst, 1);
            s.set_index_point(i2, oc.isfirst, 2);
        }
        if !cork.intcouture {
            self.write_whole_cork(oc, vtx, ishape, et, cork, (&cv1, &cv2))?;
        } else {
            self.write_split_cork(oc, ishape, et, cork)?;
        }
        let (mut box1, mut box2) = (BoundingBox3::default(), BoundingBox3::default());
        let hbs: Handle<dyn Adaptor3dSurface> = oc.hbs.clone().cast();
        enlarge_box_pcurve(&hbs, &cork.pc, (cork.udeb, cork.ufin), &mut box1, &mut box2)?;
        if oc.onsame {
            self.write_prolongation(oc, vtx, ishape, cork, (&mut box1, &mut box2))?;
        }
        enlarge_box_ds(
            &self.ds,
            Some(&oc.stripe),
            &oc.fd,
            (&mut box1, &mut box2),
            oc.isfirst,
        )?;
        let (cv1, cv2) = {
            let g = lock(&oc.fd);
            (
                g.vertex(oc.isfirst, 1).clone(),
                g.vertex(oc.isfirst, 2).clone(),
            )
        };
        for (cv, b) in [(&cv1, &mut box1), (&cv2, &mut box2)] {
            if cv.is_on_arc() {
                enlarge_box_edge(
                    cv.arc()?,
                    self.my_ef_map.find_from_key(cv.arc()?.as_ref())?,
                    cv.parameter_on_arc()?,
                    b,
                )?;
            }
        }
        let (ip1, ip2) = {
            let s = lock(&oc.stripe);
            (s.index_point(oc.isfirst, 1), s.index_point(oc.isfirst, 2))
        };
        if !cv1.is_vertex() {
            set_point_tolerance(&mut self.ds, &box1, ip1)?;
        }
        if !cv2.is_vertex() {
            set_point_tolerance(&mut self.ds, &box2, ip2)?;
        }
        Ok(())
    }

    /// The ends of the cork merged with the ends of the traces of the fillets on the face
    /// they meet (StdFail_NotDone when they cross).
    pub(super) fn merge_with_traces(
        &mut self,
        ishape: i32,
        icurve: i32,
        cork: &Cork,
    ) -> Result<(), Failure> {
        let tol2d = self.tol2d;
        let cork_pc = Geom2dAdaptorCurve::with_range(&cork.pc, cork.udeb, cork.ufin)?;
        let mut inter = GInter::new();
        let mut check = |ds: &mut rustycad_tkbool::top_ope_brep_ds::TopOpeBRepDSDataStructure,
                         other: &Geom2dAdaptorCurve,
                         other_index: Option<i32>|
         -> Result<(), Failure> {
            inter.perform_between(&cork_pc, other, tol2d, precision::p_confusion())?;
            if has_transversal_intersection(&inter)? {
                return Err(Failure::with_message(
                    "StdFail_NotDone",
                    "OneCorner : fillets have too big radiuses",
                ));
            }
            let (mut cs, mut os) = (false, false);
            if has_common_endpoint(&inter, (&cork_pc, other), tol2d, (&mut cs, &mut os))?
                && let Some(oi) = other_index
                && oi > 0
            {
                ds.merge_equivalent_curve_points(icurve, cs, oi, os);
            }
            Ok(())
        };
        for st in self.my_list_stripe.clone() {
            let Some(seq) = lock(&st).set_of_surf_data().cloned() else {
                continue;
            };
            let seq: Vec<SurfDataHandle> = lock(&seq).clone();
            for data in &seq {
                let fi = {
                    let g = lock(data);
                    if ishape == g.index_of_s1() {
                        g.interference(1).clone()
                    } else if ishape == g.index_of_s2() {
                        g.interference(2).clone()
                    } else {
                        continue;
                    }
                };
                if fi.pcurve_on_face().is_null() {
                    continue;
                }
                let other = Geom2dAdaptorCurve::with_range(
                    fi.pcurve_on_face(),
                    fi.first_parameter(),
                    fi.last_parameter(),
                )?;
                // `IsEqual(Last, First)`: closer than RealSmall().
                if (other.last_parameter()? - other.first_parameter()?).abs() < f64::MIN_POSITIVE {
                    continue;
                }
                check(&mut self.ds, &other, Some(fi.line_index()))?;
            }
        }
        let list = self.ds.change_shape_interferences_index(ishape)?.clone();
        for i in list.iter() {
            let sci: Handle<TopOpeBRepDSSurfaceCurveInterference> = i.clone().cast();
            let Some(s) = sci.get() else {
                continue;
            };
            let geom = i.get().map_or(0, |b| b.geometry());
            let oc: Handle<GeomTrimmedCurve> = self.ds.curve(geom).curve().clone().cast();
            let Some(o) = oc.get() else {
                continue;
            };
            let other = Geom2dAdaptorCurve::with_range(
                &s.pcurve(),
                o.first_parameter(),
                o.last_parameter(),
            )?;
            check(&mut self.ds, &other, Some(geom))?;
        }
        Ok(())
    }

    /// The cork of one piece written: its curve and pcurves, the merges with the traces,
    /// the pseudo-seams at the vertex, the curve of the stripe.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn write_whole_cork(
        &mut self,
        oc: &OneCorner,
        vtx: &Vertex,
        ishape: i32,
        et: Orientation,
        cork: &Cork,
        (cv1, cv2): (&CommonPoint, &CommonPoint),
    ) -> Result<(), Failure> {
        let icurve = self.ds.add_curve(&mut TopOpeBRepDSCurve::new(
            &cork.cc,
            cork.tolreached,
            false,
        ));
        let interfc = fil_curve_in_ds(icurve, ishape, &cork.pc, et);
        self.merge_with_traces(ishape, icurve, cork)?;
        self.ds
            .change_shape_interferences_index(ishape)?
            .append(interfc);
        for cv in [cv1, cv2] {
            if cv.is_on_arc() && is_pseudo_seam(cv.arc()?, &oc.fv)? {
                self.write_pseudo_seam(oc, vtx, cv, cork)?;
            }
        }
        let mut s = lock(&oc.stripe);
        *s.change_pcurve(oc.isfirst) = cork.ps.clone();
        s.set_curve(icurve, oc.isfirst);
        s.set_parameters(oc.isfirst, cork.udeb, cork.ufin);
        Ok(())
    }

    /// The part of a pseudo-seam arc between the vertex and the cork written with its
    /// pcurves, and the degenerated edge at the vertex.
    pub(super) fn write_pseudo_seam(
        &mut self,
        oc: &OneCorner,
        vtx: &Vertex,
        cv: &CommonPoint,
        cork: &Cork,
    ) -> Result<(), Failure> {
        let arc = cv.arc()?.clone();
        let ec = BRepTool::curve_on_face(&arc, &oc.fv)?;
        if ec.pcurve.is_null() {
            return Err(Failure::with_message(
                "Standard_ConstructionError",
                "Failed to get p-curve of edge",
            ));
        }
        let (hc, first, last) = (ec.pcurve.clone(), ec.first, ec.last);
        let hcc = hc.get().ok_or_else(|| Failure::new("OneCorner"))?;
        let pcc = cork.pc.get().ok_or_else(|| Failure::new("OneCorner"))?;
        let pfac1 = hcc.value(cv.parameter_on_arc()?)?;
        let (pcf, pcl) = (pcc.value(cork.udeb)?, pcc.value(cork.ufin)?);
        let onfirst = pfac1.distance(pcf) < pfac1.distance(pcl);
        let der_pc = if onfirst {
            pcc.d1(cork.udeb)?.1
        } else {
            let mut d = pcc.d1(cork.ufin)?.1;
            d.reverse();
            d
        };
        let (_, der_hc) = hcc.d1(cv.parameter_on_arc()?)?;
        let (prm1, prm2, first_to_par) = if der_hc.dot(der_pc) > 0. {
            (cv.parameter_on_arc()?, last, false)
        } else {
            (first, cv.parameter_on_arc()?, true)
        };
        let (ct, _, _) = BRepTool::curve(&arc)?;
        let ct: Handle<dyn GeomCurve> =
            Handle::new(GeomTrimmedCurve::new(&ct, prm1, prm2, true, true)?).cast();
        let toled = BRepTool::edge_tolerance(&arc)?;
        let indcurv = self
            .ds
            .add_curve(&mut TopOpeBRepDSCurve::new(&ct, toled, false));
        let indpoint = {
            let s = lock(&oc.stripe);
            if oc.isfirst {
                s.index_first_point_on_s1()
            } else {
                s.index_last_point_on_s1()
            }
        };
        let indvertex = self.ds.add_shape(vtx.as_ref());
        let (p1, p2) = if first_to_par {
            (
                fil_point_in_ds(Orientation::Forward, indcurv, indvertex, prm1, true),
                fil_point_in_ds(Orientation::Reversed, indcurv, indpoint, prm2, false),
            )
        } else {
            (
                fil_point_in_ds(Orientation::Forward, indcurv, indpoint, prm1, false),
                fil_point_in_ds(Orientation::Reversed, indcurv, indvertex, prm2, true),
            )
        };
        self.ds.change_curve_interferences(indcurv).append(p1);
        self.ds.change_curve_interferences(indcurv).append(p2);
        let indface = self.ds.add_shape(oc.fv.as_ref());
        let interfc = fil_curve_in_ds(indcurv, indface, &hc, arc.orientation());
        self.ds
            .change_shape_interferences_index(indface)?
            .append(interfc);
        let mut local = arc.clone();
        local.reverse();
        let hcr = pcurve_of(&local, &oc.fv)?;
        let interfc = fil_curve_in_ds(indcurv, indface, &hcr, local.orientation());
        self.ds
            .change_shape_interferences_index(indface)?
            .append(interfc);
        self.degenerated_at_vertex(oc, vtx, (&hc, first, last), first_to_par, indvertex)
    }

    /// The point of the vertex on the degenerated edge of the face there, if any.
    pub(super) fn degenerated_at_vertex(
        &mut self,
        oc: &OneCorner,
        vtx: &Vertex,
        (hc, first, last): (&Handle<dyn Geom2dCurve>, f64, f64),
        first_to_par: bool,
        indvertex: i32,
    ) -> Result<(), Failure> {
        let mut explo = Explorer::new(oc.fv.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)?;
        let mut edeg = None;
        while explo.more() {
            let ecur = topods::edge(explo.current())?;
            explo.next()?;
            if BRepTool::degenerated(&ecur)? {
                let (vf, vl) = topexp::vertices(&ecur, false);
                if vf.is_same(vtx) || vl.is_same(vtx) {
                    edeg = Some(ecur);
                    break;
                }
            }
        }
        let Some(edeg) = edeg else {
            return Ok(());
        };
        let ec = BRepTool::curve_on_face(&edeg, &oc.fv)?;
        if ec.pcurve.is_null() {
            return Err(Failure::with_message(
                "Standard_ConstructionError",
                "Failed to get p-curve of edge",
            ));
        }
        let (mut cd, fd) = (ec.pcurve.clone(), ec.first);
        let tcd: Handle<Geom2dTrimmedCurve> = cd.clone().cast();
        if let Some(t) = tcd.get() {
            cd = t.basis_curve();
        }
        let p2d = value_2d(hc, if first_to_par { first } else { last })?;
        let par = ProjectPointOnCurve::with_curve(p2d, &cd)?.lower_distance_parameter()?;
        let ideg = self.ds.add_shape(edeg.as_ref());
        let ori = if par < fd {
            Orientation::Forward
        } else {
            Orientation::Reversed
        };
        let i = fil_point_in_ds(ori, ideg, indvertex, par, true);
        self.ds.change_shape_interferences_index(ideg)?.append(i);
        Ok(())
    }

    /// The cork split at the seam written: its two parts on the face and on the fillet,
    /// the point on the seam and the points of the parts.
    pub(super) fn write_split_cork(
        &mut self,
        oc: &OneCorner,
        ishape: i32,
        mut et: Orientation,
        cork: &mut Cork,
    ) -> Result<(), Failure> {
        let mut c2d = [Handle::<dyn Geom2dCurve>::default(), Handle::default()];
        #[allow(clippy::needless_range_loop)]
        for k in 0..2 {
            compute_curve_2d(&cork.curv[k], &oc.fv, &mut c2d[k])?;
            let i = fil_curve_in_ds(cork.icurv[k], ishape, &c2d[k], et);
            self.ds.change_shape_interferences_index(ishape)?.append(i);
        }
        if lock(&oc.fd).orientation() == oc.fv.orientation() {
            et = top_abs::reverse(et);
        }
        let isurf = lock(&oc.fd).surf();
        let ranges = [(cork.udeb, cork.par2), (cork.par2, cork.ufin)];
        #[allow(clippy::needless_range_loop)]
        for k in 0..2 {
            let c: Handle<dyn Geom2dCurve> = Handle::new(Geom2dTrimmedCurve::new(
                &cork.ps,
                ranges[k].0,
                ranges[k].1,
                true,
                true,
            )?)
            .cast();
            let i = fil_curve_in_ds(cork.icurv[k], isurf, &c, et);
            self.ds.change_surface_interferences(isurf).append(i);
        }
        let iarc = self.ds.add_shape(cork.edgecouture.as_ref());
        let (vdeb, vfin) = (
            topexp::first_vertex(&cork.edgecouture, false),
            topexp::last_vertex(&cork.edgecouture, false),
        );
        let (pard, parf) = (
            BRepTool::parameter(&vdeb, &cork.edgecouture)?,
            BRepTool::parameter(&vfin, &cork.edgecouture)?,
        );
        let ori = if (cork.par1 - pard).abs() < (parf - cork.par1).abs() {
            Orientation::Forward
        } else {
            Orientation::Reversed
        };
        let i = fil_point_in_ds(ori, iarc, cork.indpt, cork.par1, false);
        self.ds.change_shape_interferences_index(iarc)?.append(i);
        lock(&oc.stripe).in_ds(oc.isfirst, 1);
        let (ind1, ind2) = {
            let s = lock(&oc.stripe);
            (s.index_point(oc.isfirst, 1), s.index_point(oc.isfirst, 2))
        };
        let points = [
            (0, Orientation::Forward, ind1, cork.udeb),
            (0, Orientation::Reversed, cork.indpt, cork.par2),
            (1, Orientation::Forward, cork.indpt, cork.par2),
            (1, Orientation::Reversed, ind2, cork.ufin),
        ];
        for (k, o, ip, par) in points {
            let i = fil_point_in_ds(o, cork.icurv[k], ip, par, false);
            self.ds.change_curve_interferences(cork.icurv[k]).append(i);
        }
        Ok(())
    }
}
