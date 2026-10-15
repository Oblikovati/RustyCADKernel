// Rust translation of the initializations and of the parse of the edges and faces around
// the vertex in ChFi3d_Builder::PerformMoreThreeCorner, and of the file statics
// RemoveSurfData and CpOnEdge (ChFi3d_Builder_CnCrn.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The edges around the vertex in order, each one with its contour (or sharp), and the
//! faces between them.
use super::statics::{SurfType, remove_sd, summarize_normal, surf_index};
use super::{Corner, indices};
use crate::chfi_ds::{StripeHandle, lock};
use crate::chfi3d::builder::{ChFi3dBuilder, spine_of};
use crate::chfi3d::{
    angle_edge, cherche_bords_libres, cherche_edge, cherche_element, cherche_face1,
    couture_on_vertex, edge_common_faces, index_of_surf_data, nb_not_degenerated_edges,
    surf_data_at,
};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkernel::standard::Failure;
use std::f64::consts::PI;

/// `CpOnEdge(stripe, num, isfirst, Eadj1, Eadj2, compoint)`: a common point of the end of
/// the `num`-th surface data is on one of the two edges.
fn cp_on_edge(
    stripe: &StripeHandle,
    num: i32,
    isfirst: bool,
    eadj: [&Edge; 2],
) -> Result<bool, Failure> {
    let fd = surf_data_at(stripe, num)?;
    let g = lock(&fd);
    for ons in [1, 2] {
        let cp = g.vertex(isfirst, ons);
        if cp.is_on_arc() && (cp.arc()?.is_same(eadj[0]) || cp.arc()?.is_same(eadj[1])) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// `e` set to the edge of the face `f` other than `ecur` at the vertex, the seam on the
/// face with the seam (`e` keeps its value when there is none, as natively).
fn adjacent(
    seam: (&Face, &Edge),
    v1: &Vertex,
    f: &Face,
    ecur: &Edge,
    e: &mut Edge,
) -> Result<(), Failure> {
    if f.is_same(seam.0) {
        *e = seam.1.clone();
        return Ok(());
    }
    cherche_element(v1, ecur, f, e, &mut Vertex::default())
}

/// The number of surface data of the stripe.
fn nb_surf(stripe: &StripeHandle) -> i32 {
    lock(stripe)
        .set_of_surf_data()
        .map(|s| lock(s).len() as i32)
        .unwrap_or(0)
}

/// The edge of the spine of the stripe at its end of sense `sense`.
fn end_edge(stripe: &StripeHandle, sense: i32) -> Result<Edge, Failure> {
    let sp = spine_of(stripe)?;
    let g = lock(&sp);
    let i = if sense == 1 { 1 } else { g.nb_edges() };
    Ok(g.edges(i)?.clone())
}

impl ChFi3dBuilder {
    /// `RemoveSurfData(myVDataMap, myEFMap, edgecouture, facecouture, V1)`: for each
    /// contour, the surface data before the last one with a common point on an edge
    /// adjacent to its end edge removed.
    fn remove_surf_data(&self, c: &Corner) -> Result<(), Failure> {
        // Eadj1 and Eadj2 are kept from one contour to the next, as natively.
        let mut eadj = [Edge::default(), Edge::default()];
        for stripe in self.my_vdata_map.find_from_key(&c.v1)?.to_vec() {
            let nbsurf = nb_surf(&stripe);
            if nbsurf == 1 {
                continue;
            }
            let (num, sense) = index_of_surf_data(&c.v1, &stripe)?;
            let ecur = end_edge(&stripe, sense)?;
            let (f1, f2) = edge_common_faces(self.my_ef_map.find_from_key(ecur.as_ref())?)?;
            let seam = (&c.facecouture, &c.edgecouture);
            for (k, f) in [f1, f2].iter().enumerate() {
                adjacent(seam, &c.v1, f, &ecur, &mut eadj[k])?;
                edge_common_faces(self.my_ef_map.find_from_key(eadj[k].as_ref())?)?;
            }
            remove_unused(&stripe, (num, sense, nbsurf), [&eadj[0], &eadj[1]])?;
        }
        Ok(())
    }

    /// The state of the corner: the number of edges, the free borders and the seam at the
    /// vertex, and the unused surface data removed.
    pub(super) fn corner_init(&mut self, jndex: i32, nconges: i32) -> Result<Corner, Failure> {
        let v1 = self.my_vdata_map.find_key(jndex)?;
        let mut nedge = nb_not_degenerated_edges(&v1, &self.my_ve_map)?;
        let (bordlibre, el1, el2) = cherche_bords_libres(&self.my_ve_map, &v1)?;
        let mut droit = false;
        if bordlibre {
            nedge = (nedge - 2) / 2 + 2;
            let angedg = angle_edge(&v1, &el1, &el2)?.abs();
            droit = (angedg - PI).abs() < 0.01;
        } else {
            nedge /= 2;
        }
        let mut c = Corner::new(
            (v1, jndex),
            nconges,
            nedge.max(0) as usize,
            [el1, el2],
            droit,
        );
        for s in self.my_vf_map.find_from_key(c.v1.as_ref())?.iter() {
            let fcur = topods::face(s)?;
            if let Some(e) = couture_on_vertex(&fcur, &c.v1)? {
                (c.couture, c.edgecouture, c.facecouture) = (true, e, fcur);
                break;
            }
        }
        self.remove_surf_data(&c)?;
        Ok(c)
    }
}

/// The surface data of the stripe before (after for a last end) the one nearest to the
/// vertex with a common point on the adjacent edges removed.
fn remove_unused(
    stripe: &StripeHandle,
    (num, sense, nbsurf): (i32, i32, i32),
    eadj: [&Edge; 2],
) -> Result<(), Failure> {
    let isfirst = sense == 1;
    if isfirst {
        let mut ind = 0;
        for i in 1..=nbsurf {
            if cp_on_edge(stripe, i, isfirst, eadj)? {
                ind = i;
            }
        }
        if ind >= 2 {
            remove_sd(stripe, 1, ind - 1)?;
        }
        return Ok(());
    }
    let mut ind = num;
    for i in (1..=num).rev() {
        if cp_on_edge(stripe, i, isfirst, eadj)? {
            ind = i;
        }
    }
    if ind < num {
        remove_sd(stripe, ind + 1, num)?;
    }
    Ok(())
}

/// The cursor of the parse around the vertex: the current edge and face, and the next
/// edge (kept from one search to the next as natively).
struct Parse {
    ecur: Edge,
    fcur: Face,
    enext: Edge,
}

impl ChFi3dBuilder {
    /// The first contour at the vertex with a next edge on its second face.
    fn parse_start(&self, c: &mut Corner, jndex: i32, cur: &mut Parse) -> Result<(), Failure> {
        let mut vv = Vertex::default();
        for cnext in self.my_vdata_map.find_from_index(jndex)?.to_vec() {
            let (num, sense) = index_of_surf_data(&c.v1, &cnext)?;
            c.cd[0] = Some(cnext.clone());
            (c.index[0], c.sens[0]) = (num, sense);
            c.numfa[0][1] = surf_index(c, 0, num, SurfType::Face2)?;
            c.numfa[1][0] = c.numfa[0][1];
            cur.fcur = topods::face(self.ds.shape(c.numfa[0][1], true))?;
            c.fvive[0][1] = cur.fcur.clone();
            c.fvive[1][0] = cur.fcur.clone();
            c.jf[0] = 2;
            cur.ecur = end_edge(&cnext, sense)?;
            c.evive[0] = cur.ecur.clone();
            cherche_edge(&c.v1, &evive_shapes(c), &cur.fcur, &mut cur.enext, &mut vv)?;
            if !cur.enext.is_null() {
                break;
            }
        }
        summarize_normal(&c.v1, &cur.fcur, &cur.ecur, &mut c.sum_normal)
    }

    /// `PerformMoreThreeCorner`: the parse of the edges and faces around the vertex.
    pub(super) fn parse_edges(&mut self, c: &mut Corner, jndex: i32) -> Result<(), Failure> {
        let mut cur = Parse {
            ecur: Edge::default(),
            fcur: Face::default(),
            enext: Edge::default(),
        };
        self.parse_start(c, jndex, &mut cur)?;
        let (mut nbcouture, mut ii) = (0, 1usize);
        while ii < c.nedge {
            if cur.fcur.is_same(&c.facecouture) && nbcouture == 0 {
                cur.enext = c.edgecouture.clone();
                nbcouture += 1;
            } else {
                let mut vv = Vertex::default();
                cherche_edge(&c.v1, &evive_shapes(c), &cur.fcur, &mut cur.enext, &mut vv)?;
            }
            if cur.enext.is_null() {
                return Err(Failure::with_message(
                    "Standard_ConstructionError",
                    "PerformMoreThreeCorner: pb in the parsing of edges and faces",
                ));
            }
            if cur.enext.is_same(&c.edgelibre[0]) || cur.enext.is_same(&c.edgelibre[1]) {
                ii = self.parse_free_borders(c, ii, &mut cur)?;
            } else {
                self.parse_next(c, jndex, ii, &mut cur)?;
            }
            ii += 1;
        }
        Ok(())
    }

    /// The two free borders met at `ii` and `ii + 1` (both sharp); the index of the
    /// second one.
    fn parse_free_borders(
        &mut self,
        c: &mut Corner,
        ii: usize,
        cur: &mut Parse,
    ) -> Result<usize, Failure> {
        let enext = cur.enext.clone();
        set_sharp(c, ii, &enext);
        self.set_face(c, ii, &cur.fcur);
        let ii = ii + 1;
        cur.ecur = if enext.is_same(&c.edgelibre[0]) {
            c.edgelibre[1].clone()
        } else {
            c.edgelibre[0].clone()
        };
        // ChFi3d_edge_common_faces(.., Fcur, Fcur): the second face wins.
        cur.fcur = edge_common_faces(self.my_ef_map.find_from_key(cur.ecur.as_ref())?)?.1;
        self.set_face(c, ii, &cur.fcur);
        let ecur = cur.ecur.clone();
        set_sharp(c, ii, &ecur);
        Ok(ii)
    }

    /// `Fvive(ii, icplus)` and `numfa(ii, icplus)` (and the symmetric ones) set to `f`.
    fn set_face(&mut self, c: &mut Corner, ii: usize, f: &Face) {
        let (icplus, _) = indices(c.nedge, ii);
        c.fvive[ii][icplus] = f.clone();
        c.fvive[icplus][ii] = f.clone();
        c.numfa[ii][icplus] = self.ds.add_shape(f.as_ref());
        c.numfa[icplus][ii] = c.numfa[ii][icplus];
    }

    /// The contour of the stripe ending on `enext` at the vertex, with its index and
    /// sense.
    fn stripe_on(
        &self,
        c: &Corner,
        jndex: i32,
        enext: &Edge,
    ) -> Result<Option<(StripeHandle, i32, i32)>, Failure> {
        for st in self.my_vdata_map.find_from_index(jndex)?.to_vec() {
            let (index, sense) = index_of_surf_data(&c.v1, &st)?;
            let ee = end_edge(&st, sense)?;
            if enext.is_same(&ee) {
                return Ok(Some((st, index, sense)));
            }
        }
        Ok(None)
    }

    /// The edge `ii` (`Enext`): its contour or sharp, the face after it, and `jf`.
    fn parse_next(
        &mut self,
        c: &mut Corner,
        jndex: i32,
        ii: usize,
        cur: &mut Parse,
    ) -> Result<(), Failure> {
        let enext = cur.enext.clone();
        let found = self.stripe_on(c, jndex, &enext)?;
        match &found {
            Some((st, index, sense)) => {
                (c.cd[ii], c.index[ii], c.sens[ii]) = (Some(st.clone()), *index, *sense);
                c.sharp[ii] = false;
                c.evive[ii] = enext.clone();
            }
            None => set_sharp(c, ii, &enext),
        }
        let mut fnext = cur.fcur.clone();
        cherche_face1(
            self.my_ef_map.find_from_key(enext.as_ref())?,
            &cur.fcur,
            &mut fnext,
        )?;
        self.set_face(c, ii, &fnext);
        if found.is_some() {
            self.update_jf(c, ii)?;
        }
        cur.ecur = enext;
        cur.fcur = fnext;
        summarize_normal(&c.v1, &cur.fcur, &cur.ecur, &mut c.sum_normal)
    }

    /// `jf(ii)` from the face of the contour that is `numfa(ii, icplus)`; when neither
    /// is, the faces found again from the previous one (cts16288).
    fn update_jf(&mut self, c: &mut Corner, ii: usize) -> Result<(), Failure> {
        let (icplus, icmoins) = indices(c.nedge, ii);
        let numface2 = surf_index(c, ii, c.index[ii], SurfType::Face2)?;
        if numface2 == c.numfa[ii][icplus] {
            c.jf[ii] = 2;
            return Ok(());
        }
        let numface1 = surf_index(c, ii, c.index[ii], SurfType::Face1)?;
        if numface1 == c.numfa[ii][icplus] {
            c.jf[ii] = 1;
            return Ok(());
        }
        if numface1 == c.numfa[icmoins][ii] {
            c.jf[ii] = 2;
            let f = topods::face(self.ds.shape(numface2, true))?;
            self.set_face(c, ii, &f);
        }
        if numface2 == c.numfa[icmoins][ii] {
            c.jf[ii] = 1;
            let f = topods::face(self.ds.shape(numface1, true))?;
            self.set_face(c, ii, &f);
        }
        Ok(())
    }
}

/// The edge `ii` is sharp (`Enext` without contour): no contour, its sense from its first
/// vertex.
fn set_sharp(c: &mut Corner, ii: usize, e: &Edge) {
    c.cd[ii] = None;
    c.index[ii] = 0;
    c.sens[ii] = if topexp::first_vertex(e, false).is_same(&c.v1) {
        1
    } else {
        -1
    };
    c.sharp[ii] = true;
    c.evive[ii] = e.clone();
    c.jf[ii] = 0;
}

/// `Evive` as the array of shapes searched by `ChFi3d_cherche_edge`.
pub(super) fn evive_shapes(c: &Corner) -> Vec<Shape> {
    c.evive.iter().map(|e| e.clone().into()).collect()
}
