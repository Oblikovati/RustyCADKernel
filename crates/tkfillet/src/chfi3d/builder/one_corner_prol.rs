// Rust translation of the OnSame prolongation of ChFi3d_Builder::PerformOneCorner
// (ChFi3d_Builder_C1.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OnSame, the edge of the opposite face prolonged on the face at the end, from the
//! vertex to the end of the fillet.
use super::ChFi3dBuilder;
use super::corner_tools::{compute_curve_2d, recale};
use super::one_corner::{Cork, OneCorner, pcurve_of, value_2d};
use crate::chfi_ds::lock;
use crate::chfi3d::{
    IntersectionCurves, boite, bound_fac, compute_curves, couture, fil_curve_in_ds,
    fil_point_in_ds, fil_vertex_in_ds,
};
use rustycad_tkbool::top_ope_brep_ds::{TopOpeBRepDSCurve, TopOpeBRepDSPoint};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tktopalgo::brep_extrema::BRepExtremaExtCC;
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;

/// The orientation of the vertex in the edge (forward), `default` when absent.
fn vertex_orientation(e: &Edge, v: &Vertex, default: Orientation) -> Result<Orientation, Failure> {
    let mut ex = Explorer::new(
        &e.oriented(Orientation::Forward),
        ShapeEnum::Vertex,
        ShapeEnum::Shape,
    )?;
    let mut o = default;
    while ex.more() {
        if v.is_same(ex.current()) {
            o = ex.current().orientation();
        }
        ex.next()?;
    }
    Ok(o)
}

impl ChFi3dBuilder {
    /// The intersection of the opposite face with the face at the end from the vertex to
    /// the end of the fillet: the curve, its pcurves on the two faces.
    fn prolongation_curve(
        &mut self,
        oc: &OneCorner,
        vtx: &Vertex,
        cork: &mut Cork,
    ) -> Result<IntersectionCurves, Failure> {
        let par_vtx = BRepTool::parameter(vtx, &oc.arcprol)?;
        let pop1 = value_2d(&pcurve_of(&oc.arcprol, &oc.fop)?, par_vtx)?;
        let pop2 = {
            let g = lock(&oc.fd);
            let fiop = g.interference(oc.ifop);
            value_2d(fiop.pcurve_on_face(), fiop.parameter(oc.isfirst))?
        };
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
            // The native default wholeCurv = true.
            true,
            &mut out,
        )? {
            return Err(Failure::new("OneCorner : echec calcul intersection"));
        }
        cork.tolreached = out.tolreached;
        Ok(out)
    }

    /// The prolongation split where it crosses the seam of the opposite face (unless the
    /// cork was split already).
    fn split_prolongation(
        &mut self,
        oc: &OneCorner,
        cork: &mut Cork,
        zob3d: &Handle<dyn GeomCurve>,
    ) -> Result<(), Failure> {
        let Some(edgecouture) = couture(&oc.fop)? else {
            return Ok(());
        };
        cork.edgecouture = edgecouture.clone();
        if BRepTool::degenerated(&edgecouture)? {
            return Ok(());
        }
        let edg = BRepLibMakeEdge::from_curve(zob3d)?.edge()?;
        let ext = BRepExtremaExtCC::with_edges(&edgecouture, &edg)?;
        if !ext.is_done() || ext.nb_ext()? == 0 {
            return Ok(());
        }
        let mut i = 1;
        while i <= ext.nb_ext()? && !cork.intcouture {
            if ext.square_distance(i)? <= 1.0e-8 {
                cork.par1 = ext.parameter_on_e1(i)?;
                cork.par2 = ext.parameter_on_e2(i)?;
                cork.indpt = self
                    .ds
                    .add_point(&TopOpeBRepDSPoint::new(ext.point_on_e1(i)?, 1.0e-4));
                cork.intcouture = true;
                cork.cc = zob3d.clone();
                self.add_split_curves(cork, (cork.udeb, cork.par2, cork.ufin))?;
            }
            i += 1;
        }
        Ok(())
    }

    /// OnSame: the edge of the opposite face prolonged on the face at the end, written
    /// into the data structure (in two parts when it crosses a seam).
    pub(super) fn write_prolongation(
        &mut self,
        oc: &OneCorner,
        vtx: &Vertex,
        ishape: i32,
        cork: &mut Cork,
        (box1, box2): (&mut BoundingBox3, &mut BoundingBox3),
    ) -> Result<(), Failure> {
        let iarcspine = self.ds.add_shape(oc.arcspine.as_ref());
        let ivtx = self.ds.add_shape(vtx.as_ref());
        let ovtx = top_abs::reverse(vertex_orientation(&oc.arcspine, vtx, Orientation::Forward)?);
        let par_vtx = BRepTool::parameter(vtx, &oc.arcspine)?;
        let interfv = fil_vertex_in_ds(ovtx, iarcspine, ivtx, par_vtx);
        self.ds
            .change_shape_interferences_index(iarcspine)?
            .append(interfv);
        let ovtx = vertex_orientation(&oc.arcprol, vtx, ovtx)?;
        let out = self.prolongation_curve(oc, vtx, cork)?;
        let zob3d = out.c3d.clone();
        let z = zob3d.get().ok_or_else(|| Failure::new("OneCorner"))?;
        cork.udeb = z.first_parameter();
        cork.ufin = z.last_parameter();
        let izob = self
            .ds
            .add_curve(&mut TopOpeBRepDSCurve::new(&zob3d, cork.tolreached, false));
        self.split_prolongation(oc, cork, &zob3d)?;
        let icc = lock(&oc.stripe).index_point(oc.isfirst, oc.ifop);
        if cork.intcouture {
            self.write_split_prolongation(oc, (ishape, ivtx, icc), ovtx, cork)?;
        } else {
            let et = top_abs::reverse(top_abs::compose(ovtx, oc.oarcprolv));
            let i = fil_curve_in_ds(izob, ishape, &out.pc2, et);
            self.ds.change_shape_interferences_index(ishape)?.append(i);
            let et = top_abs::reverse(top_abs::compose(ovtx, oc.oarcprolop));
            let iop = self.ds.add_shape(oc.fop.as_ref());
            let i = fil_curve_in_ds(izob, iop, &out.pc1, et);
            self.ds.change_shape_interferences_index(iop)?.append(i);
            let i = fil_vertex_in_ds(Orientation::Forward, izob, ivtx, cork.udeb);
            self.ds.change_curve_interferences(izob).append(i);
            let i = fil_point_in_ds(Orientation::Reversed, izob, icc, cork.ufin, false);
            self.ds.change_curve_interferences(izob).append(i);
            let p = z.value(cork.ufin)?;
            if oc.ifop == 1 {
                box1.add_point(p);
            } else {
                box2.add_point(p);
            }
        }
        Ok(())
    }

    /// The prolongation in two parts at the seam written (with the pcurves of the
    /// normal projections on the opposite face).
    fn write_split_prolongation(
        &mut self,
        oc: &OneCorner,
        (ishape, ivtx, icc): (i32, i32, i32),
        ovtx: Orientation,
        cork: &Cork,
    ) -> Result<(), Failure> {
        let et = top_abs::reverse(top_abs::compose(ovtx, oc.oarcprolv));
        for k in 0..2 {
            let mut c2d = Handle::<dyn Geom2dCurve>::default();
            compute_curve_2d(&cork.curv[k], &oc.fop, &mut c2d)?;
            let i = fil_curve_in_ds(cork.icurv[k], ishape, &c2d, et);
            self.ds.change_shape_interferences_index(ishape)?.append(i);
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
            Orientation::Reversed
        } else {
            Orientation::Forward
        };
        let i = fil_point_in_ds(ori, iarc, cork.indpt, cork.par1, false);
        self.ds.change_shape_interferences_index(iarc)?.append(i);
        let iop = self.ds.add_shape(oc.fop.as_ref());
        let et = top_abs::reverse(top_abs::compose(ovtx, oc.oarcprolop));
        for k in 0..2 {
            let mut c2d = Handle::<dyn Geom2dCurve>::default();
            compute_curve_2d(&cork.curv[k], &oc.fop, &mut c2d)?;
            let i = fil_curve_in_ds(cork.icurv[k], iop, &c2d, et);
            self.ds.change_shape_interferences_index(iop)?.append(i);
        }
        let points = [
            (
                0,
                fil_vertex_in_ds(Orientation::Forward, cork.icurv[0], ivtx, cork.udeb),
            ),
            (
                0,
                fil_point_in_ds(
                    Orientation::Reversed,
                    cork.icurv[0],
                    cork.indpt,
                    cork.par2,
                    false,
                ),
            ),
            (
                1,
                fil_point_in_ds(
                    Orientation::Forward,
                    cork.icurv[1],
                    cork.indpt,
                    cork.par2,
                    false,
                ),
            ),
            (
                1,
                fil_point_in_ds(Orientation::Reversed, cork.icurv[1], icc, cork.ufin, false),
            ),
        ];
        for (k, i) in points {
            self.ds.change_curve_interferences(cork.icurv[k]).append(i);
        }
        Ok(())
    }
}
