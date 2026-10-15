// Rust translation of the search of the faces cut by the fillet in
// ChFi3d_Builder::PerformIntersectionAtEnd (ChFi3d_Builder_C1.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The chain of faces and edges around the vertex between the two sides of a fillet.
use super::ChFi3dBuilder;
use super::at_end::{AtEnd, NN};
use super::find_face::{cherche_edge1, cherche_face};
use crate::chfi_ds::State;
use crate::chfi3d::{
    angle_edge, cherche_edge, cherche_element, cherche_face1, cherche_vertex, edge_state,
    is_tangent_faces,
};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use std::f64::consts::PI;

/// The failure of a chain longer than `nn` faces.
fn too_many() -> Failure {
    Failure::new("IntersectionAtEnd : the max number of faces reached")
}

/// The edge has two vertices, none being `vtx`.
fn far_edge(e: &Edge, vtx: &Vertex) -> Result<bool, Failure> {
    let mut m = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(e.as_ref(), ShapeEnum::Vertex, &mut m)?;
    Ok(m.extent() == 2
        && !m.find_key(1)?.is_same(vtx.as_ref())
        && !m.find_key(2)?.is_same(vtx.as_ref()))
}

/// The vertex is a vertex of the shape.
fn has_vertex(s: &Shape, v: &Vertex) -> Result<bool, Failure> {
    let mut ex = Explorer::new(s, ShapeEnum::Vertex, ShapeEnum::Shape)?;
    while ex.more() {
        if v.is_same(ex.current()) {
            return Ok(true);
        }
        ex.next()?;
    }
    Ok(false)
}

impl ChFi3dBuilder {
    /// The faces of `fref` crossed from the vertex `v1` along the edges of `fref` until
    /// `vfin` (`nb` counting them in the chain).
    fn walk_chain(
        &self,
        ae: &mut AtEnd,
        (mut v1, vfin): (Vertex, Vertex),
        mut ei: Edge,
        fref: &Face,
        nb: &mut usize,
    ) -> Result<(), Failure> {
        let (mut e, mut v2) = (Edge::default(), Vertex::default());
        while !v1.is_same(&vfin) {
            cherche_element(&v1, &ei, fref, &mut e, &mut v2)?;
            v1 = v2.clone();
            ei = e.clone();
            let mut fnb = Face::default();
            cherche_face1(self.my_ef_map.find_from_key(e.as_ref())?, fref, &mut fnb)?;
            ae.faces[*nb] = fnb;
            ae.edges[*nb] = cherche_edge1(&ae.faces[*nb - 1], &ae.faces[*nb])?;
            *nb += 1;
            if *nb >= NN {
                return Err(too_many());
            }
        }
        Ok(())
    }

    /// The chain when three faces meet at the vertex.
    pub(super) fn three_faces_chain(&self, ae: &mut AtEnd) -> Result<(), Failure> {
        let (cv1, cv2) = (ae.cv(1), ae.cv(2));
        let mut findonf1 = cv1.is_vertex() || far_edge(&ae.edges[0], &ae.vtx)?;
        let findonf2 = cv2.is_vertex() || far_edge(&ae.edges[2], &ae.vtx)?;
        let mut nb = 1usize;
        if ae.state == State::OnSame {
            let mut three = [Edge::default(), Edge::default(), ae.edge_spine.clone()];
            let mut v2 = Vertex::default();
            cherche_element(&ae.vtx, &ae.edge_spine, &ae.f1, &mut three[0], &mut v2)?;
            cherche_element(&ae.vtx, &ae.edge_spine, &ae.f2, &mut three[1], &mut v2)?;
            if edge_state(&three, &self.my_ef_map)? == State::OnSame {
                ae.is_on_same[0] = true;
                nb = 1;
                ae.edges[0] = three[0].clone();
                let mut f0 = Face::default();
                cherche_face1(
                    self.my_ef_map.find_from_key(ae.edges[0].as_ref())?,
                    &ae.f1,
                    &mut f0,
                )?;
                ae.faces[0] = f0;
                if findonf2 {
                    findonf1 = true;
                } else {
                    ae.edges[1] = cv2.arc()?.clone();
                }
            } else {
                ae.is_on_same[1] = true;
            }
        }
        if findonf1 && !ae.is_on_same[0] {
            let v1 = if cv1.transition_on_arc()? == Orientation::Forward {
                topexp::first_vertex(cv1.arc()?, false)
            } else {
                topexp::last_vertex(cv1.arc()?, false)
            };
            let mut f0 = Face::default();
            cherche_face1(
                self.my_ef_map.find_from_key(cv1.arc()?.as_ref())?,
                &ae.f1,
                &mut f0,
            )?;
            ae.faces[0] = f0;
            nb = 1;
            let (ei, f1) = (ae.edges[0].clone(), ae.f1.clone());
            self.walk_chain(ae, (v1, ae.vtx.clone()), ei, &f1, &mut nb)?;
            if !findonf2 {
                ae.edges[nb] = cv2.arc()?.clone();
            }
        }
        if findonf2 && !ae.is_on_same[1] {
            if !findonf1 {
                nb = 1;
            }
            let vfin = if cv2.transition_on_arc()? == Orientation::Forward {
                topexp::last_vertex(cv2.arc()?, false)
            } else {
                topexp::first_vertex(cv2.arc()?, false)
            };
            if !findonf1 {
                let mut f = Face::default();
                cherche_face1(
                    self.my_ef_map.find_from_key(cv1.arc()?.as_ref())?,
                    &ae.f1,
                    &mut f,
                )?;
                ae.faces[nb - 1] = f;
            }
            let (mut e, mut v2) = (Edge::default(), Vertex::default());
            cherche_element(&ae.vtx, &ae.edge_spine, &ae.f2, &mut e, &mut v2)?;
            let f2 = ae.f2.clone();
            self.walk_chain(ae, (v2, vfin), e, &f2, &mut nb)?;
            ae.edges[nb] = cv2.arc()?.clone();
        }
        if ae.is_on_same[1] {
            ae.edges[nb] = cherche_edge1(&ae.faces[nb - 1], &ae.f2)?;
            ae.faces[nb] = ae.f2.clone();
        }
        ae.nbface = nb;
        Ok(())
    }

    /// The chain around a vertex of more than three faces; false when the arcs of the
    /// fillet do not suit it (the corner is then left to `IntersectMoreCorner`).
    pub(super) fn many_faces_chain(&self, ae: &mut AtEnd) -> Result<bool, Failure> {
        let (cv1, cv2) = (ae.cv(1), ae.cv(2));
        let possible1 = has_vertex(cv1.arc()?.as_ref(), &ae.vtx)?;
        let possible2 = has_vertex(cv2.arc()?.as_ref(), &ae.vtx)?;
        if !((possible1 && possible2) || (!possible1 && !possible2) || ae.nbarete > 4) {
            return Ok(false);
        }
        let mut nb = 0usize;
        let mut trouve = false;
        let vf = self.my_vf_map.find_from_key(ae.vtx.as_ref())?;
        while !trouve {
            nb += 1;
            if nb >= NN {
                return Err(too_many());
            }
            if nb != 1 {
                ae.f3 = ae.faces[nb - 2].clone();
            }
            ae.faces[nb - 1] = ae.f3.clone();
            let arc1 = cv1.arc()?;
            let e = if arc1.is_same(&ae.edgelibre[0]) {
                ae.edgelibre[1].clone()
            } else if arc1.is_same(&ae.edgelibre[1]) {
                ae.edgelibre[0].clone()
            } else {
                ae.edges[nb - 1].clone()
            };
            // Face[nb - 1] (F3 here) and Edge[nb] stay as they are when the searches
            // find nothing, as natively.
            let mut fnb = ae.f3.clone();
            cherche_face(vf, &e, (&ae.f1, &ae.f2, &ae.f3), &mut fnb)?;
            ae.faces[nb - 1] = fnb;
            let (mut en, mut v) = (ae.edges[nb].clone(), Vertex::default());
            cherche_edge(&ae.vtx, &ae.tabedg, &ae.faces[nb - 1], &mut en, &mut v)?;
            ae.edges[nb] = en;
            ae.tabedg[nb] = ae.edges[nb].clone().into();
            if ae.edges[nb].is_same(cv2.arc()?) {
                trouve = true;
            }
        }
        ae.nbface = nb;
        if ae.nbarete == 4 {
            self.straight_edges(ae)?;
        } else if ae.nbarete == 5 {
            let t0 = is_tangent_faces(&ae.edges[0], &ae.f1, &ae.faces[0], Continuity::G1)?;
            let t1 = is_tangent_faces(&ae.edges[1], &ae.faces[0], &ae.faces[1], Continuity::G1)?;
            let t2 = is_tangent_faces(&ae.edges[2], &ae.faces[1], &ae.faces[2], Continuity::G1)?;
            if (t0 || t2) && t1 {
                ae.facesau = ae.faces[0].clone();
                ae.edgesau = ae.edges[0].clone();
                ae.nbface = 1;
                ae.edges[1] = ae.edges[3].clone();
                ae.faces[0] = ae.faces[2].clone();
                ae.oneintersection[0] = true;
            }
        }
        Ok(true)
    }

    /// With four edges at the vertex, two of them in line: one intersection only.
    fn straight_edges(&self, ae: &mut AtEnd) -> Result<(), Failure> {
        let mut ang1 = 0.0;
        if cherche_vertex(&ae.edges[0], &ae.edges[1])?.is_some_and(|v| v.is_same(&ae.vtx)) {
            ang1 = angle_edge(&ae.vtx, &ae.edges[0], &ae.edges[1])?;
        }
        if (ang1 - PI).abs() < 0.01 {
            ae.oneintersection[0] = true;
            ae.facesau = ae.faces[0].clone();
            ae.edgesau = ae.edges[1].clone();
            ae.faces[0] = ae.faces[1].clone();
            ae.edges[1] = ae.edges[2].clone();
            ae.nbface = 1;
            return Ok(());
        }
        if cherche_vertex(&ae.edges[1], &ae.edges[2])?.is_some_and(|v| v.is_same(&ae.vtx)) {
            ang1 = angle_edge(&ae.vtx, &ae.edges[1], &ae.edges[2])?;
        }
        if (ang1 - PI).abs() < 0.01 {
            ae.oneintersection[1] = true;
            ae.facesau = ae.faces[1].clone();
            ae.edgesau = ae.edges[1].clone();
            ae.edges[1] = ae.edges[2].clone();
            ae.nbface = 1;
        }
        Ok(())
    }
}
