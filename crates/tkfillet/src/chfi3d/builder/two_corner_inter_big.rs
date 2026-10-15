// Rust translation of the second branch of ChFi3d_Builder::PerformTwoCornerbyInter
// (ChFi3d_Builder_C2.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The bevel of two fillets reaching the pivot at different points: the small fillet
//! cut by the big one, the big one cut by the face at the end.
use super::ChFi3dBuilder;
use super::two_corner_inter::{CornerEnd, reduce_curve, reduce_surfaces, surf_point};
use crate::chfi_ds::{CommonPoint, lock};
use crate::chfi3d::{
    IntersectionCurves, boite, bound_fac, computes_int_pc, enlarge_box_curve, enlarge_box_ds,
    enlarge_box_edge, enlarge_box_pcurve, fil_curve_in_ds, fil_point_in_ds, index_point_in_ds,
    int_cs, set_point_tolerance,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve2d, BRepAdaptorSurface};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, compose, reverse};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::Point2;

/// The end of the big or small fillet with its bounded surface.
type Fillet = (CornerEnd, Handle<GeomAdaptorSurface>);

/// The boxes of the points of the bevel: on the common face, in the middle (where the
/// small fillet meets the face at the end) and on the arc.
#[derive(Default)]
struct BevelBoxes {
    bco: BoundingBox3,
    bmil: BoundingBox3,
    barc: BoundingBox3,
}

/// The index in the data structure of the point where the small fillet meets the face.
struct BevelPoints {
    ipoint_mil: i32,
}

/// Whether the vertex is the first vertex of the pivot (its parameter grows from it).
fn pivot_grows_from(pivot: &Edge, vtx: &Vertex) -> Result<bool, Failure> {
    let mut ex = Explorer::new(
        &pivot.oriented(Orientation::Forward),
        ShapeEnum::Vertex,
        ShapeEnum::Shape,
    )?;
    while ex.more() {
        if ex.current().is_same(vtx) {
            return Ok(ex.current().orientation() == Orientation::Forward);
        }
        ex.next()?;
    }
    Ok(false)
}

/// The orientation of the edge `e` in the forward face, if it is there.
fn orientation_in_face(f: &Face, e: &Edge) -> Result<Option<Orientation>, Failure> {
    let mut ex = Explorer::new(
        &f.oriented(Orientation::Forward),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )?;
    while ex.more() {
        if ex.current().is_same(e) {
            return Ok(Some(ex.current().orientation()));
        }
        ex.next()?;
    }
    Ok(None)
}

impl ChFi3dBuilder {
    /// The bevel of fillets reaching the pivot at different points: the big one is the
    /// one reaching it farther from the vertex.
    pub(super) fn bevel_big_small(
        &mut self,
        vtx: &Vertex,
        pivot: &Edge,
        (par_cp1, par_cp2): (f64, f64),
        f1: Fillet,
        f2: Fillet,
    ) -> Result<bool, Failure> {
        let parcrois = pivot_grows_from(pivot, vtx)?;
        let (big, sma) = if (parcrois && par_cp2 > par_cp1) || (!parcrois && par_cp2 < par_cp1) {
            (f2, f1)
        } else {
            (f1, f2)
        };
        self.bevel_cut(big, sma)
    }

    /// The cut of the small fillet by the big one, then of the big one by the face at
    /// the end.
    fn bevel_cut(&mut self, (big, big_hs): Fillet, (sma, sma_hs): Fillet) -> Result<bool, Failure> {
        let fi = |e: &CornerEnd, on_s: i32| lock(&e.fd).interference(on_s).clone();
        let (usma, ubig, _) = computes_int_pc(
            (&fi(&sma, sma.ifaco), &sma_hs),
            (&fi(&big, big.ifaco), &big_hs),
            (sma.uintpc, big.uintpc),
        )?;
        let big = CornerEnd {
            uintpc: ubig,
            ..big
        };
        let sma = CornerEnd {
            uintpc: usma,
            ..sma
        };
        let uvb = surf_point(&big.fd, big.ifaco, ubig)?;
        let uvs = surf_point(&sma.fd, sma.ifaco, usma)?;
        let pardeb = [uvs.x(), uvs.y(), uvb.x(), uvb.y()];
        let pfaco = sma_hs.value(uvs.x(), uvs.y())?;
        let fi_arc_sma = fi(&sma, sma.ifaarc());
        let ctg = self.ds.curve(fi_arc_sma.line_index()).curve().clone();
        let mut wi;
        let hctg = Handle::new(if sma.isfirst {
            wi = fi_arc_sma.first_parameter();
            let temp = wi.min(usma);
            GeomAdaptorCurve::with_range(&ctg, temp, fi_arc_sma.last_parameter())?
        } else {
            wi = fi_arc_sma.last_parameter();
            let temp = wi.max(usma);
            GeomAdaptorCurve::with_range(&ctg, fi_arc_sma.first_parameter(), temp)?
        });
        let same_surf = lock(&sma.fd).surf() == lock(&big.fd).surf();
        if same_surf {
            reduce_surfaces(usma, ubig, &sma_hs, &big_hs)?;
            reduce_curve(usma, ubig, &hctg)?;
        }
        let gbig: Handle<dyn Adaptor3dSurface> = big_hs.clone().cast();
        let Some((uvi, w)) = int_cs(&gbig, &hctg.clone().cast(), wi)? else {
            return Ok(false);
        };
        wi = w;
        let uvm = fi_arc_sma
            .pcurve_on_surf()
            .get()
            .ok_or_else(|| Failure::with_message("Standard_NullObject", "PCurveOnSurf"))?
            .value(wi)?;
        let parfin = [uvm.x(), uvm.y(), uvi.x(), uvi.y()];
        let pmil = sma_hs.value(parfin[0], parfin[1])?;
        let gsma: Handle<dyn Adaptor3dSurface> = sma_hs.clone().cast();
        let Some(gc) = self.inter_fillets((&gsma, &gbig), pardeb, parfin)? else {
            return Ok(false);
        };
        let mut boxes = BevelBoxes::default();
        let pts = self.write_small(&sma, &big, &gc, (pfaco, pmil, wi), &big_hs, &mut boxes)?;
        self.cut_big_by_face(&sma, &big, &big_hs, wi, same_surf, pts, &mut boxes)
    }

    /// The small fillet ends on its intersection with the big one: its contour, its
    /// points, the curve on the big one; the first point of the big one too.
    fn write_small(
        &mut self,
        sma: &CornerEnd,
        big: &CornerEnd,
        gc: &IntersectionCurves,
        (pfaco, pmil, wi): (
            rustycad_tkmath::gp::Point3,
            rustycad_tkmath::gp::Point3,
            f64,
        ),
        big_hs: &Handle<GeomAdaptorSurface>,
        boxes: &mut BevelBoxes,
    ) -> Result<BevelPoints, Failure> {
        let c = gc
            .c3d
            .get()
            .ok_or_else(|| Failure::new("TwoCorner: null curve"))?;
        let (wfirst, wlast) = (c.first_parameter(), c.last_parameter());
        let tolpco = lock(&sma.fd)
            .vertex(sma.isfirst, sma.ifaco)
            .tolerance()
            .max(lock(&big.fd).vertex(big.isfirst, big.ifaco).tolerance());
        let tolpmil = lock(&sma.fd).vertex(sma.isfirst, sma.ifaarc()).tolerance();
        let icurv = self.ds.add_curve(
            &mut rustycad_tkbool::top_ope_brep_ds::TopOpeBRepDSCurve::new(
                &gc.c3d,
                gc.tolreached,
                false,
            ),
        );
        {
            let mut s = lock(&sma.stripe);
            s.set_parameters(sma.isfirst, wfirst, wlast);
            s.set_curve(icurv, sma.isfirst);
            *s.change_pcurve(sma.isfirst) = gc.pc1.clone();
        }
        let (psmaco, psmamil) = {
            let mut g = lock(&sma.fd);
            let set = |cp: &mut CommonPoint, p, tol: f64| {
                cp.reset();
                cp.set_point(p);
                cp.set_tolerance(tol.max(gc.tolreached));
            };
            set(g.change_vertex(sma.isfirst, sma.ifaco), pfaco, tolpco);
            g.change_interference(sma.ifaco)
                .set_parameter(sma.uintpc, sma.isfirst);
            set(g.change_vertex(sma.isfirst, sma.ifaarc()), pmil, tolpmil);
            g.change_interference(sma.ifaarc())
                .set_parameter(wi, sma.isfirst);
            (
                g.vertex(sma.isfirst, sma.ifaco).clone(),
                g.vertex(sma.isfirst, sma.ifaarc()).clone(),
            )
        };
        let ipoint_co = index_point_in_ds(&psmaco, &mut self.ds)?;
        let ipoint_mil = index_point_in_ds(&psmamil, &mut self.ds)?;
        {
            let mut s = lock(&sma.stripe);
            s.set_index_point(ipoint_co, sma.isfirst, sma.ifaco);
            s.set_index_point(ipoint_mil, sma.isfirst, sma.ifaarc());
            if sma.ifaco == 2 {
                s.set_orientation_at_end(Orientation::Reversed, sma.isfirst);
            }
        }
        lock(&big.stripe).set_index_point(ipoint_co, big.isfirst, big.ifaco);
        {
            let mut g = lock(&big.fd);
            *g.change_vertex(big.isfirst, big.ifaco) = psmaco;
            g.change_interference(big.ifaco)
                .set_parameter(big.uintpc, big.isfirst);
        }
        let li = self.ds.change_curve_interferences(icurv);
        li.append(fil_point_in_ds(
            Orientation::Forward,
            icurv,
            ipoint_co,
            wfirst,
            false,
        ));
        li.append(fil_point_in_ds(
            Orientation::Reversed,
            icurv,
            ipoint_mil,
            wlast,
            false,
        ));
        let tracurv = self.big_transition(big)?;
        let isurf = lock(&big.fd).surf();
        let interfc = fil_curve_in_ds(icurv, isurf, &gc.pc2, tracurv);
        self.ds.change_surface_interferences(isurf).append(interfc);
        let pair = if sma.ifaco == 1 {
            (&mut boxes.bco, &mut boxes.bmil)
        } else {
            (&mut boxes.bmil, &mut boxes.bco)
        };
        enlarge_box_ds(&self.ds, Some(&sma.stripe), &sma.fd, pair, sma.isfirst)?;
        let gbig: Handle<dyn Adaptor3dSurface> = big_hs.clone().cast();
        enlarge_box_pcurve(
            &gbig,
            &gc.pc2,
            (wfirst, wlast),
            &mut boxes.bco,
            &mut boxes.bmil,
        )?;
        Ok(BevelPoints { ipoint_mil })
    }

    /// The transition of the intersection curves on the big fillet.
    fn big_transition(&self, big: &CornerEnd) -> Result<Orientation, Failure> {
        let g = lock(&big.fd);
        let tra = g.interference_on_s1().transition();
        let ofac = self.ds.shape(g.index_of_s1(), true).orientation();
        let ofil = g.orientation();
        let mut tracurv = compose(compose(ofac, ofil), tra);
        if !big.isfirst {
            tracurv = reverse(tracurv);
        }
        if big.ifaco != 1 {
            tracurv = reverse(tracurv);
        }
        Ok(tracurv)
    }

    /// The big fillet cut by the face at the end of the small one, from the middle point
    /// to its point on the arc; its contour, the curve on the face and the tolerances.
    #[allow(clippy::too_many_arguments)]
    fn cut_big_by_face(
        &mut self,
        sma: &CornerEnd,
        big: &CornerEnd,
        big_hs: &Handle<GeomAdaptorSurface>,
        wi: f64,
        same_surf: bool,
        pts: BevelPoints,
        boxes: &mut BevelBoxes,
    ) -> Result<bool, Failure> {
        let f = topods::face(self.ds.shape(lock(&sma.fd).index(sma.ifaarc()), true))?;
        let hf = Handle::new(BRepAdaptorSurface::with_face(&f, true)?);
        let fi_arc_sma = lock(&sma.fd).interference(sma.ifaarc()).clone();
        let (mut fsma, mut lsma) = (fi_arc_sma.first_parameter(), fi_arc_sma.last_parameter());
        let delt_sma = 0.05 * (lsma - fsma);
        let pcpc = fi_arc_sma.pcurve_on_face().clone();
        let pc = pcpc
            .get()
            .ok_or_else(|| Failure::with_message("Standard_NullObject", "PCurveOnFace"))?;
        fsma = pc.first_parameter().max(wi - delt_sma);
        lsma = pc.last_parameter().min(wi + delt_sma);
        if lsma < fsma {
            return Ok(false);
        }
        let c2df: Handle<dyn Adaptor2dCurve2d> =
            Handle::new(Geom2dAdaptorCurve::with_range(&pcpc, fsma, lsma)?).cast();
        let ghf: Handle<dyn Adaptor3dSurface> = hf.clone().cast();
        let hconsf: Handle<dyn Adaptor3dCurve> =
            Handle::new(Adaptor3dCurveOnSurface::with_curve(&c2df, &ghf)?).cast();
        let gbig: Handle<dyn Adaptor3dSurface> = big_hs.clone().cast();
        let Some((uvi, wi)) = int_cs(&gbig, &hconsf, wi)? else {
            return Ok(false);
        };
        let ppff1 = pc.value(wi)?;
        let pardeb = [ppff1.x(), ppff1.y(), uvi.x(), uvi.y()];
        let ptg = lock(&big.fd)
            .interference(big.ifaarc())
            .parameter(big.isfirst);
        let uvb = surf_point(&big.fd, big.ifaarc(), ptg)?;
        let cpend = lock(&big.fd).vertex(big.isfirst, big.ifaarc()).clone();
        let mut etest = cpend.arc()?.clone();
        if BRepTool::is_closed_on_face(&etest, &f)? {
            etest.reverse();
        }
        let arc = BRepAdaptorCurve2d::with_edge_on_face(&etest, &f)?;
        let ppff2: Point2 = arc.value(cpend.parameter_on_arc()?)?;
        let parfin = [ppff2.x(), ppff2.y(), uvb.x(), uvb.y()];
        bound_fac(&hf, boite(ppff1, ppff2), true)?;
        let Some(gc) = self.inter_fillets((&ghf, &gbig), pardeb, parfin)? else {
            return Ok(false);
        };
        self.write_big(
            sma,
            big,
            &gc,
            (&f, &hf, &etest),
            big_hs,
            same_surf,
            pts,
            boxes,
        )?;
        Ok(true)
    }

    /// The big fillet ends on its cut by the face: its contour, the curve on the face and
    /// on the fillet, and the tolerances of the points of the bevel.
    #[allow(clippy::too_many_arguments)]
    fn write_big(
        &mut self,
        sma: &CornerEnd,
        big: &CornerEnd,
        gc: &IntersectionCurves,
        (f, hf, etest): (&Face, &Handle<BRepAdaptorSurface>, &Edge),
        big_hs: &Handle<GeomAdaptorSurface>,
        same_surf: bool,
        pts: BevelPoints,
        boxes: &mut BevelBoxes,
    ) -> Result<(), Failure> {
        let c = gc
            .c3d
            .get()
            .ok_or_else(|| Failure::new("TwoCorner: null curve"))?;
        let (wfirst, wlast) = (c.first_parameter(), c.last_parameter());
        let icurv = self.ds.add_curve(
            &mut rustycad_tkbool::top_ope_brep_ds::TopOpeBRepDSCurve::new(
                &gc.c3d,
                gc.tolreached,
                false,
            ),
        );
        let cpend = {
            let mut g = lock(&big.fd);
            let cp = g.change_vertex(big.isfirst, big.ifaarc());
            cp.set_tolerance(cp.tolerance().max(gc.tolreached));
            cp.clone()
        };
        let ipoint_arc = index_point_in_ds(&cpend, &mut self.ds)?;
        lock(&big.stripe).set_index_point(ipoint_arc, big.isfirst, big.ifaarc());
        let li7 = self.ds.change_curve_interferences(icurv);
        li7.append(fil_point_in_ds(
            Orientation::Forward,
            icurv,
            pts.ipoint_mil,
            wfirst,
            false,
        ));
        li7.append(fil_point_in_ds(
            Orientation::Reversed,
            icurv,
            ipoint_arc,
            wlast,
            false,
        ));
        let isurf = lock(&big.fd).surf();
        let tracurv = self.big_transition(big)?;
        let interfc = fil_curve_in_ds(icurv, isurf, &gc.pc2, tracurv);
        self.ds.change_surface_interferences(isurf).append(interfc);
        lock(&big.stripe).in_ds(big.isfirst, 1);
        let ishape = self.ds.add_shape(f.as_ref());
        let tracurv = if same_surf {
            compose(etest.orientation(), cpend.transition_on_arc()?)
        } else {
            match orientation_in_face(f, etest)? {
                Some(o) => compose(o, cpend.transition_on_arc()?),
                None => tracurv,
            }
        };
        let interfc = fil_curve_in_ds(icurv, ishape, &gc.pc1, tracurv);
        self.ds
            .change_shape_interferences(f.as_ref())
            .append(interfc);
        let pair = if big.ifaco == 1 {
            (&mut boxes.bco, &mut boxes.barc)
        } else {
            (&mut boxes.barc, &mut boxes.bco)
        };
        enlarge_box_ds(&self.ds, None, &big.fd, pair, big.isfirst)?;
        self.enlarge_big_boxes(big, gc, (big_hs, hf), (wfirst, wlast), boxes)?;
        let (ico, imil) = {
            let s = lock(&sma.stripe);
            (
                s.index_point(sma.isfirst, sma.ifaco),
                s.index_point(sma.isfirst, sma.ifaarc()),
            )
        };
        let iarc = lock(&big.stripe).index_point(big.isfirst, big.ifaarc());
        set_point_tolerance(&mut self.ds, &boxes.bco, ico)?;
        set_point_tolerance(&mut self.ds, &boxes.bmil, imil)?;
        set_point_tolerance(&mut self.ds, &boxes.barc, iarc)?;
        Ok(())
    }

    /// The boxes of the middle and arc points extended by the cut of the big fillet.
    fn enlarge_big_boxes(
        &self,
        big: &CornerEnd,
        gc: &IntersectionCurves,
        (big_hs, hf): (&Handle<GeomAdaptorSurface>, &Handle<BRepAdaptorSurface>),
        (wfirst, wlast): (f64, f64),
        boxes: &mut BevelBoxes,
    ) -> Result<(), Failure> {
        let gbig: Handle<dyn Adaptor3dSurface> = big_hs.clone().cast();
        enlarge_box_pcurve(
            &gbig,
            &gc.pc2,
            (wfirst, wlast),
            &mut boxes.bmil,
            &mut boxes.barc,
        )?;
        let ghf: Handle<dyn Adaptor3dSurface> = hf.clone().cast();
        enlarge_box_pcurve(
            &ghf,
            &gc.pc1,
            (wfirst, wlast),
            &mut boxes.bmil,
            &mut boxes.barc,
        )?;
        enlarge_box_curve(&gc.c3d, wfirst, wlast, &mut boxes.bmil, &mut boxes.barc)?;
        let cparc = lock(&big.fd).vertex(big.isfirst, big.ifaarc()).clone();
        let arc = cparc.arc()?.clone();
        enlarge_box_edge(
            &arc,
            self.my_ef_map.find_from_key(arc.as_ref())?,
            cparc.parameter_on_arc()?,
            &mut boxes.barc,
        )
    }
}
