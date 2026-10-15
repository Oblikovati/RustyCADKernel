// Rust translation of ChFi3d_Builder::IntersectMoreCorner (ChFi3d_Builder_C1.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The corner of one contour at a vertex where more than three edges meet: the fillet
//! cut by the face at the end (prolonged), and the edge of the opposite face continued
//! on that face from the vertex.
use super::ChFi3dBuilder;
use super::compute::spine_of;
use super::corner_tools::{extend_surface, inters_update_on_same};
use super::find_face::cherche_face;
use super::one_corner::{OneCorner, edge_orientation, has_vertex};
use crate::chfi_ds::lock;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve2d, BRepAdaptorSurface};
use rustycad_tkbrep::topods::{self, Edge, Face, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point2;

/// The edge at the vertex other than `arcpiv` on the face `fv`, with its orientation in
/// the face (the first in the order of the edges at the vertex).
fn edge_to_prolong(ve: &rustycad_tkernel::ncollection::List<rustycad_tkbrep::topods::Shape>, arcpiv: &Edge, fv: &Face) -> Result<Option<(Edge, Orientation)>, Failure> {
    for s in ve.iter() {
        if arcpiv.is_same(s) {
            continue;
        }
        let e = topods::edge(s)?;
        if let Some(o) = edge_orientation(fv, &e)? {
            return Ok(Some((e, o)));
        }
    }
    Ok(None)
}

impl ChFi3dBuilder {
    /// The face at the end of the fillet: the face of the pivot other than `fad` (the
    /// last face of the list when there is no other).
    fn face_beyond(&self, arcpiv: &Edge, fad: &Face, fv: &mut Face) -> Result<(), Failure> {
        for f in self.my_ef_map.find_from_key(arcpiv.as_ref())?.iter() {
            if !fad.is_same(f) {
                *fv = topods::face(f)?;
                return Ok(());
            }
        }
        Ok(())
    }

    /// The sides of the corner: the side on the pivot (the arc through the vertex), the
    /// faces, the face at the end and the edge to prolong; the face at the end extended;
    /// the pivot and the face beyond the edge to prolong.
    fn more_corner_faces(&mut self, oc: &mut OneCorner, vtx: &Vertex) -> Result<(Edge, Face), Failure> {
        let (cv1, cv2) = {
            let g = lock(&oc.fd);
            (g.vertex(oc.isfirst, 1).clone(), g.vertex(oc.isfirst, 2).clone())
        };
        if !cv1.is_on_arc() && !cv2.is_on_arc() {
            return Err(Failure::new("Corner intersmore : no point on arc"));
        } else if cv1.is_on_arc() && cv2.is_on_arc() {
            if has_vertex(cv2.arc()?.as_ref(), vtx)? {
                oc.ifad = 2;
            }
        } else if cv2.is_on_arc() {
            oc.ifad = 2;
        }
        oc.ifop = 3 - oc.ifad;
        let side = |oc: &OneCorner, s: &Self| -> Result<(Edge, Face, Face), Failure> {
            let g = lock(&oc.fd);
            Ok((
                g.vertex(oc.isfirst, oc.ifad).arc()?.clone(),
                topods::face(s.ds.shape(g.index(oc.ifad), true))?,
                topods::face(s.ds.shape(g.index(oc.ifop), true))?,
            ))
        };
        let (mut arcpiv, fad, fop) = side(oc, self)?;
        (oc.fad, oc.fop) = (fad, fop);
        let mut fv = Face::default();
        self.face_beyond(&arcpiv, &oc.fad, &mut fv)?;
        if !has_vertex(fv.as_ref(), vtx)? {
            oc.ifad = 3 - oc.ifad;
            oc.ifop = 3 - oc.ifop;
            let (a, fad, fop) = side(oc, self)?;
            (arcpiv, oc.fad, oc.fop) = (a, fad, fop);
            self.face_beyond(&arcpiv, &oc.fad, &mut fv)?;
        }
        if fv.is_null() {
            return Err(Failure::with_message("StdFail_NotDone", "OneCorner : face at end is not found"));
        }
        fv.set_orientation(Orientation::Forward);
        oc.fad.set_orientation(Orientation::Forward);
        let ve = self.my_ve_map.find_from_key(vtx.as_ref())?.clone();
        let Some((arcprol, ov)) = edge_to_prolong(&ve, &arcpiv, &fv)? else {
            return Err(Failure::with_message(
                "StdFail_NotDone",
                "IntersectMoreCorner: edge to be extended is not found",
            ));
        };
        (oc.arcprol, oc.oarcprolv) = (arcprol, ov);
        let mut fopbis = Face::default();
        cherche_face(self.my_vf_map.find_from_key(vtx.as_ref())?, &oc.arcprol, (&oc.fad, &fv, &fv), &mut fopbis)?;
        oc.fop.set_orientation(Orientation::Forward);
        if let Some(o) = edge_orientation(&fopbis, &oc.arcprol)? {
            oc.oarcprolop = o;
        }
        oc.fv = fv;
        self.extend_face_at_end(oc)?;
        Ok((arcpiv, fopbis))
    }

    /// The surface of the face at the end extended (B-spline or Bezier) and set as its
    /// new surface; the adaptors of the faces.
    fn extend_face_at_end(&mut self, oc: &mut OneCorner) -> Result<(), Failure> {
        let mut sface = BRepTool::surface(&oc.fv)?;
        let mut prol = 0;
        extend_surface(&mut sface, &mut prol)?;
        let tol = BRepTool::face_tolerance(&oc.fv)?;
        let mut ffv = Face::default();
        BRepBuilder::default().make_face_on_surface(&mut ffv, &sface, tol)?;
        if prol != 0 {
            oc.hbs.initialize(&ffv, false)?;
            self.ds.set_new_surface(oc.fv.as_ref(), &sface);
        } else {
            oc.hbs.initialize(&oc.fv, false)?;
        }
        oc.hbop.initialize(&oc.fop, true)?;
        Ok(())
    }

    /// The ends of the fillet moved onto the face at the end: on the opposite side by
    /// the intersection of its curve (or along the prolonged edge), on the pivot side
    /// along the arc; whether the first succeeded.
    fn more_corner_update(&mut self, oc: &mut OneCorner, vtx: &Vertex) -> Result<bool, Failure> {
        let surf = self.ds.surface(lock(&oc.fd).surf()).surface().clone();
        let hgs: Handle<dyn Adaptor3dSurface> = Handle::new(GeomAdaptorSurface::with_surface(&surf)?).cast();
        let hbs: Handle<dyn Adaptor3dSurface> = oc.hbs.clone().cast();
        let mut wop = lock(&oc.fd).interference(oc.ifad).parameter(oc.isfirst);
        let c3df = self.ds.curve(lock(&oc.fd).interference(oc.ifop).line_index()).curve().clone();
        let inters = {
            let mut g = lock(&oc.fd);
            let (mut fi, mut cp) = (g.interference(oc.ifop).clone(), g.vertex(oc.isfirst, oc.ifop).clone());
            let r = inters_update_on_same(
                &hgs,
                &hbs,
                &c3df,
                (&oc.fop, &oc.fv, &oc.arcprol, vtx),
                oc.isfirst,
                10. * self.tolapp3d,
                (&mut fi, &mut cp),
                &mut oc.p2dbout,
                &mut wop,
            )?;
            *g.change_interference(oc.ifop) = fi;
            *g.change_vertex(oc.isfirst, oc.ifop) = cp;
            r
        };
        let cpad = lock(&oc.fd).vertex(oc.isfirst, oc.ifad).clone();
        let pced: Handle<dyn Adaptor2dCurve2d> =
            Handle::new(BRepAdaptorCurve2d::with_edge_on_face(cpad.arc()?, &oc.fv)?).cast();
        self.update_side_on_edge(&oc.fd, (oc.isfirst, oc.ifad), &hbs, &pced, &hgs)?;
        Ok(inters)
    }

    /// `IntersectMoreCorner(Index)`: the corner of one contour at the `index`-th vertex
    /// where more than three edges meet.
    pub(crate) fn intersect_more_corner(&mut self, index: i32) -> Result<(), Failure> {
        let vtx = self.my_vdata_map.find_key(index)?;
        let stripe = self
            .my_vdata_map
            .find_from_index(index)?
            .first()
            .cloned()
            .ok_or_else(|| Failure::new("IntersectMoreCorner: no contour"))?;
        let spine = spine_of(&stripe)?;
        let (fd, isfirst) = self.corner_surf_data(&stripe, &vtx)?;
        let arcspine = {
            let sp = lock(&spine);
            if isfirst { sp.edges(1)?.clone() } else { sp.edges(sp.nb_edges())?.clone() }
        };
        let fop = topods::face(self.ds.shape(lock(&fd).index(2), true))?;
        let mut oc = OneCorner {
            stripe,
            fd,
            isfirst,
            onsame: true,
            fv: Face::default(),
            fad: Face::default(),
            fop,
            arcprol: Edge::default(),
            arcspine,
            oarcprolv: Orientation::Forward,
            oarcprolop: Orientation::Forward,
            hbs: Handle::new(BRepAdaptorSurface::default()),
            hbop: Handle::new(BRepAdaptorSurface::default()),
            ifad: 1,
            ifop: 2,
            p2dbout: Point2::default(),
        };
        let (arcpiv, fopbis) = self.more_corner_faces(&mut oc, &vtx)?;
        if !self.more_corner_update(&mut oc, &vtx)? {
            return Err(Failure::with_message("Standard_NotImplemented", "OneCorner : cap not written"));
        }
        let mut cork = self.cork_curve(&oc)?;
        self.write_more_corner(&oc, &vtx, (&arcpiv, &fopbis), &mut cork)
    }
}
