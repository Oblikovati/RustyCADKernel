// Rust translation of the data structure part of ChFi3d_Builder::IntersectMoreCorner
// (ChFi3d_Builder_C1.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The corner of one contour at a vertex of more than three edges written: the cut of
//! the fillet, the vertex on the spine, and the edge of the opposite face continued on
//! the face at the end.
use super::ChFi3dBuilder;
use super::corner_tools::recale;
use super::one_corner::{Cork, OneCorner, edge_orientation, pcurve_of, value_2d};
use crate::chfi_ds::lock;
use crate::chfi3d::{
    IntersectionCurves, boite, bound_fac, compute_curves, enlarge_box_ds, enlarge_box_edge,
    enlarge_box_pcurve, fil_curve_in_ds, fil_point_in_ds, fil_vertex_in_ds, index_point_in_ds,
    set_point_tolerance,
};
use rustycad_tkbool::top_ope_brep_ds::TopOpeBRepDSCurve;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, compose, reverse};
use rustycad_tkmath::bnd::BoundingBox3;

/// The orientation of the vertex in the forward edge (the first occurrence), `default`
/// when absent.
fn vertex_in_edge(e: &Edge, v: &Vertex, default: Orientation) -> Result<Orientation, Failure> {
    let mut ex = Explorer::new(
        &e.oriented(Orientation::Forward),
        ShapeEnum::Vertex,
        ShapeEnum::Shape,
    )?;
    while ex.more() {
        if v.is_same(ex.current()) {
            return Ok(ex.current().orientation());
        }
        ex.next()?;
    }
    Ok(default)
}

impl ChFi3dBuilder {
    /// The corner written (the second half of `IntersectMoreCorner`).
    pub(super) fn write_more_corner(
        &mut self,
        oc: &OneCorner,
        vtx: &Vertex,
        (arcpiv, fopbis): (&Edge, &Face),
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
                Some(o) => reverse(compose(o, cv1.transition_on_arc()?)),
                None => Orientation::Forward,
            }
        } else {
            match edge_orientation(&oc.fv, cv2.arc()?)? {
                Some(o) => compose(o, cv2.transition_on_arc()?),
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
            let icurve = self.ds.add_curve(&mut TopOpeBRepDSCurve::new(
                &cork.cc,
                cork.tolreached,
                false,
            ));
            let interfc = fil_curve_in_ds(icurve, ishape, &cork.pc, et);
            self.ds
                .change_shape_interferences_index(ishape)?
                .append(interfc);
            let mut s = lock(&oc.stripe);
            *s.change_pcurve(oc.isfirst) = cork.ps.clone();
            s.set_curve(icurve, oc.isfirst);
            s.set_parameters(oc.isfirst, cork.udeb, cork.ufin);
        } else {
            self.write_split_cork(oc, ishape, et, cork)?;
        }
        let (mut box1, mut box2) = (BoundingBox3::default(), BoundingBox3::default());
        let hbs: Handle<dyn Adaptor3dSurface> = oc.hbs.clone().cast();
        enlarge_box_pcurve(&hbs, &cork.pc, (cork.udeb, cork.ufin), &mut box1, &mut box2)?;
        self.write_continuation(oc, vtx, (arcpiv, fopbis), ishape)?;
        enlarge_box_ds(
            &self.ds,
            Some(&oc.stripe),
            &oc.fd,
            (&mut box1, &mut box2),
            oc.isfirst,
        )?;
        for (cv, b) in [(&cv1, &mut box1), (&cv2, &mut box2)] {
            if cv.is_on_arc() {
                let arc = cv.arc()?;
                enlarge_box_edge(
                    arc,
                    self.my_ef_map.find_from_key(arc.as_ref())?,
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

    /// The vertex on the spine, and the edge at the vertex other than the spine, the
    /// pivot and the prolonged edge continued on the face at the end from the vertex to
    /// the end of the fillet on the opposite face.
    fn write_continuation(
        &mut self,
        oc: &OneCorner,
        vtx: &Vertex,
        (arcpiv, fopbis): (&Edge, &Face),
        ishape: i32,
    ) -> Result<(), Failure> {
        let iarcspine = self.ds.add_shape(oc.arcspine.as_ref());
        let ivtx = self.ds.add_shape(vtx.as_ref());
        let ovtx = reverse(vertex_in_edge(&oc.arcspine, vtx, Orientation::Forward)?);
        let par_vtx = BRepTool::parameter(vtx, &oc.arcspine)?;
        let interfv = fil_vertex_in_ds(ovtx, iarcspine, ivtx, par_vtx);
        self.ds
            .change_shape_interferences_index(iarcspine)?
            .append(interfv);
        let mut arcprolbis = Edge::default();
        for s in self.my_ve_map.find_from_key(vtx.as_ref())?.iter() {
            if !(oc.arcprol.is_same(s) || oc.arcspine.is_same(s) || arcpiv.is_same(s)) {
                arcprolbis = topods::edge(s)?;
                break;
            }
        }
        if arcprolbis.is_null() {
            return Err(Failure::with_message(
                "Standard_ConstructionError",
                "IntersectMoreCorner: continuation edge (Arcprolbis) is not found",
            ));
        }
        let ovtx2 = vertex_in_edge(&arcprolbis, vtx, Orientation::Forward)?;
        let ovtx = vertex_in_edge(&oc.arcprol, vtx, Orientation::Forward)?;
        let oarcprolbis = if oc.fop.is_same(fopbis) {
            oc.oarcprolop
        } else {
            edge_orientation(&oc.fop, &arcprolbis)?.unwrap_or(Orientation::Forward)
        };
        let zob = self.continuation_curve(oc, vtx, &arcprolbis)?;
        let c = zob
            .c3d
            .get()
            .ok_or_else(|| Failure::new("IntersectMoreCorner: null curve"))?;
        let (udeb, ufin) = (c.first_parameter(), c.last_parameter());
        let izob = self
            .ds
            .add_curve(&mut TopOpeBRepDSCurve::new(&zob.c3d, zob.tolreached, false));
        let et = reverse(compose(ovtx, oc.oarcprolv));
        let iop = self.ds.add_shape(oc.fop.as_ref());
        let interfv = fil_curve_in_ds(izob, ishape, &zob.pc2, et);
        self.ds
            .change_shape_interferences_index(ishape)?
            .append(interfv);
        let et = reverse(compose(ovtx2, oarcprolbis));
        let interfop = fil_curve_in_ds(izob, iop, &zob.pc1, et);
        self.ds
            .change_shape_interferences_index(iop)?
            .append(interfop);
        let icc = lock(&oc.stripe).index_point(oc.isfirst, oc.ifop);
        let li = self.ds.change_curve_interferences(izob);
        li.append(fil_vertex_in_ds(Orientation::Forward, izob, ivtx, udeb));
        li.append(fil_point_in_ds(
            Orientation::Reversed,
            izob,
            icc,
            ufin,
            false,
        ));
        Ok(())
    }

    /// The intersection of the opposite face with the face at the end from the vertex
    /// (along the continuation edge on the opposite face, the prolonged edge on the face
    /// at the end) to the end of the fillet.
    fn continuation_curve(
        &mut self,
        oc: &OneCorner,
        vtx: &Vertex,
        arcprolbis: &Edge,
    ) -> Result<IntersectionCurves, Failure> {
        let par_vtx = BRepTool::parameter(vtx, arcprolbis)?;
        let pop1 = value_2d(&pcurve_of(arcprolbis, &oc.fop)?, par_vtx)?;
        let pop2 = {
            let g = lock(&oc.fd);
            let fiop = g.interference(oc.ifop);
            value_2d(fiop.pcurve_on_face(), fiop.parameter(oc.isfirst))?
        };
        let par_vtx = BRepTool::parameter(vtx, &oc.arcprol)?;
        let mut pv1 = value_2d(&pcurve_of(&oc.arcprol, &oc.fv)?, par_vtx)?;
        let mut pv2 = oc.p2dbout;
        recale(&oc.hbs, &mut pv1, &mut pv2, true)?;
        let pardeb = [pop1.x(), pop1.y(), pv1.x(), pv1.y()];
        let parfin = [pop2.x(), pop2.y(), pv2.x(), pv2.y()];
        bound_fac(&oc.hbs, boite(pv1, pv2), true)?;
        bound_fac(&oc.hbop, boite(pop1, pop2), true)?;
        let (hbop, hbs): (Handle<dyn Adaptor3dSurface>, Handle<dyn Adaptor3dSurface>) =
            (oc.hbop.clone().cast(), oc.hbs.clone().cast());
        let mut out = IntersectionCurves::default();
        if !compute_curves(
            (&hbop, &hbs),
            (pardeb, parfin),
            (self.tolapp3d, self.tol2d),
            true,
            &mut out,
        )? {
            return Err(Failure::new("OneCorner : echec calcul intersection"));
        }
        Ok(out)
    }
}
