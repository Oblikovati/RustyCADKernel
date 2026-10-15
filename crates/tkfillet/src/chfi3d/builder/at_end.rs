// Rust translation of the start of ChFi3d_Builder::PerformIntersectionAtEnd
// (ChFi3d_Builder_C1.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The corner of a contour ending at a vertex of more than three edges: the fillet cut by
//! the faces around the vertex, one after the other. Here: the configuration at the
//! vertex and the chain of faces to cut by.
use super::ChFi3dBuilder;
use super::compute::spine_of;
use super::find_face::contain_e;
use crate::chfi_ds::{
    CommonPoint, FaceInterference, SpineHandle, State, StripeHandle, SurfDataHandle, lock,
};
use crate::chfi3d::{
    bound_surf, cherche_bords_libres, cherche_element, couture_on_vertex, edge_common_faces,
    index_of_surf_data, index_point_in_ds, is_tangent_faces, nb_not_degenerated_edges, nbface,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::{self, Orientation};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::Point2;

/// The maximal number of faces cut by a fillet at a vertex (`nn`).
pub(super) const NN: usize = 15;

/// The state of `PerformIntersectionAtEnd`.
pub(super) struct AtEnd {
    pub stripe: StripeHandle,
    pub spine: SpineHandle,
    pub vtx: Vertex,
    pub fd: SurfDataHandle,
    pub isfirst: bool,
    pub state: State,
    pub edge_spine: Edge,
    pub num1: i32,
    pub bordlibre: bool,
    pub edgelibre: [Edge; 2],
    pub nbarete: i32,
    pub nface: i32,
    pub couture: bool,
    pub facecouture: Face,
    pub edgecouture: Edge,
    pub reg: [bool; 2],
    pub onecorner: bool,
    pub face: Face,
    pub hgs: Handle<GeomAdaptorSurface>,
    pub fi: [FaceInterference; 2],
    pub hbs: Handle<BRepAdaptorSurface>,
    pub orsurfdata: Orientation,
    pub isurf: i32,
    pub isurf_prev: i32,
    pub sdprev: Option<SurfDataHandle>,
    pub orcourbe: Orientation,
    pub is_on_same: [bool; 2],
    pub check_shrink: bool,
    pub is_shrink: bool,
    pub is_u_shrink: bool,
    pub check_shr_param: f64,
    pub prev_sd_param: f64,
    pub mid_p2d: Point2,
    pub mid_ipoint: i32,
    pub fi_length: [f64; 2],
    pub edges: Vec<Edge>,
    pub faces: Vec<Face>,
    pub tabedg: Vec<Shape>,
    pub nbface: usize,
    pub oneintersection: [bool; 2],
    pub facesau: Face,
    pub edgesau: Edge,
    pub f1: Face,
    pub f2: Face,
    pub f3: Face,
}

/// What `PerformIntersectionAtEnd` does after its start.
pub(super) enum AtEndNext {
    /// The corner is computed by `PerformMoreSurfdata`.
    MoreSurfdata,
    /// The corner is computed by `PerformMoreThreeCorner(Index, 1)`.
    MoreThreeCorner,
    /// The corner is computed by `IntersectMoreCorner`.
    IntersectMoreCorner,
    /// The fillet is cut by the faces.
    Cut,
}

impl AtEnd {
    /// The common point of the fillet at the vertex on its side `ons`.
    pub(super) fn cv(&self, ons: i32) -> CommonPoint {
        lock(&self.fd).vertex(self.isfirst, ons).clone()
    }

    /// The common point of the fillet at the vertex on its side `ons`, changed by `f`.
    pub(super) fn change_cv<R>(&self, ons: i32, f: impl FnOnce(&mut CommonPoint) -> R) -> R {
        f(lock(&self.fd).change_vertex(self.isfirst, ons))
    }
}

/// The two vertices of an edge, the one other than `v` first among them in the native
/// order (`Vt`).
fn other_vertex(e: &Edge, v: &Vertex) -> (Vertex, Vertex, Vertex) {
    let (v1, v2) = (
        topexp::first_vertex(e, false),
        topexp::last_vertex(e, false),
    );
    let vt = if v.is_same(&v1) {
        v2.clone()
    } else {
        v1.clone()
    };
    (v1, v2, vt)
}

impl ChFi3dBuilder {
    /// The surface data at the vertex, the one before a regular neighbour removed when the
    /// next one reaches the neighbours of the edge.
    fn at_end_surf_data(
        &mut self,
        ae: &mut AtEnd,
        num: &mut i32,
        sens: &mut i32,
    ) -> Result<(), Failure> {
        let seq = lock(&ae.stripe)
            .set_of_surf_data()
            .cloned()
            .ok_or_else(|| Failure::new("PerformIntersectionAtEnd"))?;
        let (nbsurf, nbedge) = (lock(&seq).len() as i32, lock(&ae.spine).nb_edges());
        if nbsurf == nbedge || nbsurf == 1 {
            return Ok(());
        }
        let (f1, f2) = edge_common_faces(self.my_ef_map.find_from_key(ae.edge_spine.as_ref())?)?;
        let mut vbid = Vertex::default();
        let mut eadj = [Edge::default(), Edge::default()];
        for (k, f) in [f1, f2].iter().enumerate() {
            if f.is_same(&ae.facecouture) {
                eadj[k] = ae.edgecouture.clone();
            } else {
                cherche_element(&ae.vtx, &ae.edge_spine, f, &mut eadj[k], &mut vbid)?;
            }
            let (fga, fdr) = edge_common_faces(self.my_ef_map.find_from_key(eadj[k].as_ref())?)?;
            ae.reg[k] = is_tangent_faces(&eadj[k], &fga, &fdr, Continuity::G1)?;
        }
        if !(ae.reg[0] || ae.reg[1]) {
            return Ok(());
        }
        let sd1 = lock(&seq)
            .get(ae.num1 as usize - 1)
            .cloned()
            .ok_or_else(|| Failure::out_of_range("NCollection_Sequence::Value"))?;
        let (cp1, cp2) = {
            let g = lock(&sd1);
            (
                g.vertex(ae.isfirst, 1).clone(),
                g.vertex(ae.isfirst, 2).clone(),
            )
        };
        let on_adj = |cp: &CommonPoint| -> Result<bool, Failure> {
            Ok(cp.is_on_arc() && (cp.arc()?.is_same(&eadj[0]) || cp.arc()?.is_same(&eadj[1])))
        };
        if on_adj(&cp1)? && on_adj(&cp2)? {
            lock(&seq).remove(*num as usize - 1);
            let (n, s) = index_of_surf_data(&ae.vtx, &ae.stripe)?;
            *num = n;
            *sens = s;
            ae.num1 = if ae.isfirst { n + 1 } else { n - 1 };
            ae.reg = [false, false];
        }
        Ok(())
    }

    /// The start of `PerformIntersectionAtEnd`: the configuration at the vertex, and
    /// whether another corner computation takes over.
    pub(super) fn at_end_start(&mut self, index: i32) -> Result<(AtEnd, AtEndNext), Failure> {
        let stripe = self
            .my_vdata_map
            .find_from_index(index)?
            .first()
            .cloned()
            .ok_or_else(|| Failure::new("PerformIntersectionAtEnd"))?;
        let spine = spine_of(&stripe)?;
        let vtx = self.my_vdata_map.find_key(index)?;
        let nface = nbface(self.my_vf_map.find_from_key(vtx.as_ref())?);
        let mut nbarete = nb_not_degenerated_edges(&vtx, &self.my_ve_map)?;
        let (bordlibre, el1, el2) = cherche_bords_libres(&self.my_ve_map, &vtx)?;
        nbarete = if bordlibre {
            (nbarete - 2) / 2 + 2
        } else {
            nbarete / 2
        };
        let (mut couture, mut facecouture, mut edgecouture) =
            (false, Face::default(), Edge::default());
        for s in self.my_vf_map.find_from_key(vtx.as_ref())?.iter() {
            if couture {
                break;
            }
            let fcur = topods::face(s)?;
            if let Some(e) = couture_on_vertex(&fcur, &vtx)? {
                couture = true;
                edgecouture = e;
                facecouture = fcur;
            }
        }
        let (mut num, mut sens) = index_of_surf_data(&vtx, &stripe)?;
        let isfirst = sens == 1;
        let (edge_spine, state) = {
            let sp = lock(&spine);
            if isfirst {
                (sp.edges(1)?.clone(), sp.first_status())
            } else {
                (sp.edges(sp.nb_edges())?.clone(), sp.last_status())
            }
        };
        let fd = crate::chfi3d::surf_data_at(&stripe, num)?;
        let mut ae = AtEnd {
            stripe,
            spine,
            vtx,
            fd,
            isfirst,
            state,
            edge_spine,
            num1: if isfirst { num + 1 } else { num - 1 },
            bordlibre,
            edgelibre: [el1, el2],
            nbarete,
            nface,
            couture,
            facecouture,
            edgecouture,
            reg: [false; 2],
            onecorner: false,
            face: Face::default(),
            hgs: Handle::default(),
            fi: [FaceInterference::new(), FaceInterference::new()],
            hbs: Handle::new(BRepAdaptorSurface::default()),
            orsurfdata: Orientation::Forward,
            isurf: 0,
            isurf_prev: 0,
            sdprev: None,
            orcourbe: Orientation::Forward,
            is_on_same: [false; 2],
            check_shrink: false,
            is_shrink: false,
            is_u_shrink: false,
            check_shr_param: 0.,
            prev_sd_param: 0.,
            mid_p2d: Point2::default(),
            mid_ipoint: 0,
            fi_length: [0.; 2],
            edges: vec![Edge::default(); NN],
            faces: vec![Face::default(); NN],
            tabedg: vec![Shape::default(); NN + 1],
            nbface: 0,
            oneintersection: [false; 2],
            facesau: Face::default(),
            edgesau: Edge::default(),
            f1: Face::default(),
            f2: Face::default(),
            f3: Face::default(),
        };
        self.at_end_surf_data(&mut ae, &mut num, &mut sens)?;
        ae.fd = crate::chfi3d::surf_data_at(&ae.stripe, num)?;
        let (cv1, cv2) = (ae.cv(1), ae.cv(2));
        let mut face = Face::default();
        if self.find_face(&ae.vtx, (&cv1, &cv2), &mut face)?
            && (!ae.couture || !face.is_same(&ae.facecouture))
        {
            ae.onecorner = true;
        }
        ae.face = face;
        if ae.onecorner && self.more_surfdata(index)? {
            return Ok((ae, AtEndNext::MoreSurfdata));
        }
        if !ae.onecorner && (ae.reg[0] || ae.reg[1]) && !ae.couture && ae.state != State::OnSame {
            return Ok((ae, AtEndNext::MoreThreeCorner));
        }
        self.at_end_fillet(&mut ae)?;
        let next = self.at_end_faces(&mut ae)?;
        Ok((ae, next))
    }

    /// The fillet at the vertex: its surface, its interferences, the orientation of its
    /// trace, the points of its ends and whether it shrinks to a point there.
    fn at_end_fillet(&mut self, ae: &mut AtEnd) -> Result<(), Failure> {
        ae.hgs = bound_surf(&self.ds, &ae.fd, 1, 2)?;
        {
            let g = lock(&ae.fd);
            ae.fi = [g.interference(1).clone(), g.interference(2).clone()];
            ae.orsurfdata = g.orientation();
            ae.isurf = g.surf();
        }
        let seq_len = lock(&ae.stripe)
            .set_of_surf_data()
            .map_or(0, |s| lock(s).len() as i32);
        if ae.num1 > 0 && ae.num1 <= seq_len {
            let sd = crate::chfi3d::surf_data_at(&ae.stripe, ae.num1)?;
            ae.isurf_prev = lock(&sd).surf();
            ae.sdprev = Some(sd);
        }
        let (cv1, cv2) = (ae.cv(1), ae.cv(2));
        let i1 = index_point_in_ds(&cv1, &mut self.ds)?;
        let i2 = index_point_in_ds(&cv2, &mut self.ds)?;
        {
            let mut s = lock(&ae.stripe);
            s.set_index_point(i1, ae.isfirst, 1);
            s.set_index_point(i2, ae.isfirst, 2);
        }
        let ishape1 = lock(&ae.fd).index_of_s1();
        let mut trafil1 = Orientation::Forward;
        if ishape1 != 0 {
            if ishape1 > 0 {
                trafil1 = self.ds.shape(ishape1, true).orientation();
            }
            trafil1 = top_abs::compose(trafil1, ae.orsurfdata);
            trafil1 = top_abs::compose(top_abs::reverse(ae.fi[0].transition()), trafil1);
        }
        ae.orcourbe = if ae.isfirst {
            top_abs::reverse(trafil1)
        } else {
            trafil1
        };
        self.at_end_shrink(ae)
    }

    /// Whether a side of the fillet shrinks to a point at the vertex, and the iso it lies
    /// on.
    fn at_end_shrink(&self, ae: &mut AtEnd) -> Result<(), Failure> {
        let hgs: Handle<dyn Adaptor3dSurface> = ae.hgs.clone().cast();
        let s = hgs
            .get()
            .ok_or_else(|| Failure::new("PerformIntersectionAtEnd"))?;
        let mut ends = [(Point2::default(), Point2::default()); 2];
        #[allow(clippy::needless_range_loop)]
        for k in 0..2 {
            let fi = &ae.fi[k];
            let pc = fi.pcurve_on_surf().get().ok_or_else(|| {
                Failure::with_message("Standard_NullObject", "PerformIntersectionAtEnd")
            })?;
            let (p_a, p_b) = (
                pc.value(fi.parameter(ae.isfirst))?,
                pc.value(fi.parameter(!ae.isfirst))?,
            );
            ae.fi_length[k] = s
                .value(p_a.x(), p_a.y())?
                .distance(&s.value(p_b.x(), p_b.y())?);
            ends[k] = (p_a, p_b);
        }
        ae.check_shrink =
            ae.fi_length[0] <= precision::confusion() || ae.fi_length[1] <= precision::confusion();
        if ae.check_shrink {
            let (p2d2, p2d4) = (ends[0].1, ends[1].1);
            if (p2d2.y() - p2d4.y()).abs() <= precision::p_confusion() {
                ae.is_u_shrink = false;
                ae.check_shr_param = p2d2.y();
            } else if (p2d2.x() - p2d4.x()).abs() <= precision::p_confusion() {
                ae.is_u_shrink = true;
                ae.check_shr_param = p2d2.x();
            } else {
                ae.check_shrink = false;
            }
        }
        Ok(())
    }

    /// The arc of a common point on a vertex replaced by the other edge of the face there
    /// when its far end is nearer to the vertex.
    fn nearer_arc(&self, ae: &mut AtEnd, ons: i32, k: usize, face: &Face) -> Result<(), Failure> {
        let cv = ae.cv(ons);
        if !cv.is_vertex() {
            return Ok(());
        }
        let v = cv.vertex()?.clone();
        let mut e = Edge::default();
        for s in self.my_ve_map.find_from_key(v.as_ref())?.iter() {
            e = topods::edge(s)?;
            if !e.is_same(&ae.edges[k]) && contain_e(face, &e)? {
                break;
            }
        }
        let pvtx = BRepTool::pnt(&ae.vtx)?;
        let (v1, v2, vt) = other_vertex(&ae.edges[k], &v);
        let dist1 = BRepTool::pnt(&vt)?.distance(&pvtx);
        let (v3, v4, vt) = other_vertex(&e, &v);
        let dist2 = BRepTool::pnt(&vt)?.distance(&pvtx);
        if dist2 < dist1 {
            ae.edges[k] = e;
            let ori = if v2.is_same(&v3) || v1.is_same(&v4) {
                cv.transition_on_arc()?
            } else {
                top_abs::reverse(cv.transition_on_arc()?)
            };
            let par = BRepTool::parameter(&v, &ae.edges[k])?;
            let (tol, edge) = (cv.tolerance(), ae.edges[k].clone());
            ae.change_cv(ons, |c| c.set_arc(tol, &edge, par, ori));
        }
        Ok(())
    }

    /// The chain of faces cut by the fillet, from the face of its first side to the face
    /// of its second side around the vertex.
    fn at_end_faces(&mut self, ae: &mut AtEnd) -> Result<AtEndNext, Failure> {
        ae.f1 = topods::face(self.ds.shape(lock(&ae.fd).index_of_s1(), true))?;
        ae.f2 = topods::face(self.ds.shape(lock(&ae.fd).index_of_s2(), true))?;
        ae.f3 = ae.f1.clone();
        if ae.couture || ae.bordlibre {
            ae.nface += 1;
        }
        ae.nbface = if ae.nface == 3 {
            2
        } else {
            (ae.nface - 2).max(0) as usize
        };
        let (cv1, cv2) = (ae.cv(1), ae.cv(2));
        if !cv1.is_on_arc() || !cv2.is_on_arc() {
            return Ok(AtEndNext::MoreThreeCorner);
        }
        ae.edges[0] = cv1.arc()?.clone();
        ae.edges[ae.nbface] = cv2.arc()?.clone();
        ae.tabedg[0] = ae.edges[0].clone().into();
        ae.tabedg[ae.nbface] = ae.edges[ae.nbface].clone().into();
        let f1 = ae.f1.clone();
        self.nearer_arc(ae, 1, 0, &f1)?;
        let f2 = ae.f2.clone();
        self.nearer_arc(ae, 2, 2, &f2)?;
        if ae.onecorner {
            ae.nbface = 1;
            ae.faces[0] = ae.face.clone();
            ae.edges[1] = ae.edges[2].clone();
            return Ok(AtEndNext::Cut);
        }
        if ae.nface == 3 {
            self.three_faces_chain(ae)?;
            return Ok(AtEndNext::Cut);
        }
        if !self.many_faces_chain(ae)? {
            return Ok(AtEndNext::IntersectMoreCorner);
        }
        Ok(AtEndNext::Cut)
    }
}
