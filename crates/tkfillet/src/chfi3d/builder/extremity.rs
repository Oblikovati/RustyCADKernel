// Rust translation of ChFi3d_Builder::PerformExtremity and the file statics
// TangentExtremity and TangentOnVertex (ChFi3d_Builder_1.cxx) and of
// ChFi3d_Builder::ConexFaces (ChFi3d_Builder_2.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The status of the ends of a contour: tangency of the faces, break points and free
//! boundaries.
use super::ChFi3dBuilder;
use crate::chfi_ds::{Map, SpineHandle, State, lock};
use crate::chfi3d::{conexfaces, edge_state, is_tangent_faces};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::brep_lprop::BRepLPropSLProps;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::{IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::Vector3;

/// The normal of a face at the parameter of `v` on `e`, oriented as the face (None when it
/// is not defined).
fn oriented_normal(
    v: &Vertex,
    e: &Edge,
    f: &Face,
    o: Orientation,
    hs: &BRepAdaptorSurface,
) -> Result<Option<Vector3>, Failure> {
    let p = BRepTool::parameter_on_face(v, e, f)?;
    let pc = BRepTool::curve_on_face(e, f)?.pcurve;
    let uv = pc
        .get()
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "TangentExtremity: no pcurve"))?
        .value(p)?;
    let mut prop = BRepLPropSLProps::with_parameters(hs, uv.x(), uv.y(), 1, 1.0e-9)?;
    if !prop.is_normal_defined()? {
        return Ok(None);
    }
    let mut n = Vector3::from(prop.normal()?);
    if o == Orientation::Reversed {
        n.reverse();
    }
    Ok(Some(n))
}

/// `TangentExtremity(V, E, hs1, hs2, tang)`: the faces are tangent at `v` (the angle of
/// their normals below `tang`); false when a normal is not defined.
pub(super) fn tangent_extremity(
    v: &Vertex,
    e: &Edge,
    hs1: &BRepAdaptorSurface,
    hs2: &BRepAdaptorSurface,
    tang: f64,
) -> Result<bool, Failure> {
    let mut f1 = hs1.face();
    let o1 = f1.orientation();
    f1.set_orientation(Orientation::Forward);
    let mut f2 = hs2.face();
    let o2 = f2.orientation();
    f2.set_orientation(Orientation::Forward);
    let (mut e1, mut e2) = (e.clone(), e.clone());
    e1.set_orientation(Orientation::Forward);
    e2.set_orientation(Orientation::Forward);
    if f1.is_same(&f2) && BRepTool::is_closed_on_face(&e1, &f1)? {
        e2.set_orientation(Orientation::Reversed);
    }
    let Some(n1) = oriented_normal(v, &e1, &f1, o1, hs1)? else {
        return Ok(false);
    };
    let Some(n2) = oriented_normal(v, &e2, &f2, o2, hs2)? else {
        return Ok(false);
    };
    Ok(n1.angle(n2)? < tang)
}

/// `TangentOnVertex(V, E, EFMap, tang)`: the two faces of `e` are tangent at `v`.
pub(super) fn tangent_on_vertex(
    v: &Vertex,
    e: &Edge,
    ef_map: &Map,
    tang: f64,
) -> Result<bool, Failure> {
    let (ff1, ff2) = conexfaces(e, ef_map)?;
    if ff1.is_null() || ff2.is_null() {
        return Ok(false);
    }
    let s1 = BRepAdaptorSurface::with_face(&ff1, true)?;
    let s2 = BRepAdaptorSurface::with_face(&ff2, true)?;
    tangent_extremity(v, e, &s1, &s2, tang)
}

impl ChFi3dBuilder {
    /// `ConexFaces(Spine, IEdge, HS1, HS2)`: the faces of the `iedge`-th edge of the
    /// spine, the first face of the edge first (the handles created when null).
    pub(crate) fn conex_faces(
        &self,
        spine: &SpineHandle,
        iedge: i32,
        hs1: &mut Handle<BRepAdaptorSurface>,
        hs2: &mut Handle<BRepAdaptorSurface>,
    ) -> Result<(), Failure> {
        if hs1.is_null() {
            *hs1 = Handle::new(BRepAdaptorSurface::default());
        }
        if hs2.is_null() {
            *hs2 = Handle::new(BRepAdaptorSurface::default());
        }
        let an_edge = lock(spine).edges(iedge)?.clone();
        let (mut ff1, mut ff2) = conexfaces(&an_edge, &self.my_ef_map)?;
        let first_face = self.first_face_of(&an_edge)?;
        if ff2.is_same(&first_face) {
            std::mem::swap(&mut ff1, &mut ff2);
        }
        hs1.initialize(&ff1, true)?;
        hs2.initialize(&ff2, true)?;
        Ok(())
    }

    /// `myEdgeFirstFace(E)`: the first face of an edge of a contour
    /// (Standard_NoSuchObject when unbound).
    pub(crate) fn first_face_of(&self, e: &Edge) -> Result<Face, Failure> {
        let f = self.my_edge_first_face.seek(e.as_ref()).ok_or_else(|| {
            Failure::with_message("Standard_NoSuchObject", "NCollection_DataMap::Find")
        })?;
        topods::face(f)
    }

    /// The number of distinct faces at a vertex.
    fn distinct_faces(&self, v: &Vertex) -> Result<i32, Failure> {
        let faces: Vec<Shape> = self
            .my_vf_map
            .find_from_key(v.as_ref())?
            .iter()
            .cloned()
            .collect();
        let mut nbf = 0;
        for (jf, cur) in faces.iter().enumerate() {
            if !faces[..jf].iter().any(|f| cur.is_same(f)) {
                nbf += 1;
            }
        }
        Ok(nbf)
    }

    /// The status of a break point at `v`: the state of the three sharp edges there (the
    /// first one `e0`), the smooth edges counted in `nb_g1`.
    fn break_point_state(
        &self,
        v: &Vertex,
        e0: &Edge,
        sst: State,
        nb_g1: &mut i32,
    ) -> Result<State, Failure> {
        let mut e = [e0.clone(), Edge::default(), Edge::default()];
        let mut loc_nb_g1 = 0;
        let mut edges_of_v = IndexedMapOfShape::default();
        let mut edges = MapOfShape::default();
        edges.add(e0.clone().into());
        edges_of_v.add(e0.clone().into());
        let mut ind_of_e = 0;
        let list: Vec<Shape> = self
            .my_ve_map
            .find_from_key(v.as_ref())?
            .iter()
            .cloned()
            .collect();
        for s in &list {
            let mut an_edge = topods::edge(s)?;
            if BRepTool::degenerated(&an_edge)? {
                continue;
            }
            let (f1, f2) = conexfaces(&an_edge, &self.my_ef_map)?;
            if !f2.is_null() && is_tangent_faces(&an_edge, &f1, &f2, Continuity::G2)? {
                if !f1.is_same(&f2) {
                    *nb_g1 += 1;
                    loc_nb_g1 += 1;
                }
                continue;
            }
            if edges.add(an_edge.clone().into()) {
                edges_of_v.add(an_edge.clone().into());
                if ind_of_e < 2 {
                    ind_of_e += 1;
                    e[ind_of_e] = an_edge;
                }
            } else {
                let (v1, v2) = topexp::vertices(&an_edge, false);
                if v1.is_same(&v2) {
                    let mut an_ind = edges_of_v.find_index(an_edge.as_ref());
                    if an_ind == 0 {
                        an_ind = edges_of_v.find_index(&an_edge.reversed());
                    }
                    an_edge = topods::edge(edges_of_v.find_key(an_ind)?)?;
                    an_edge.reverse();
                    if edges_of_v.add(an_edge.clone().into()) > 0 && ind_of_e < 2 {
                        ind_of_e += 1;
                        e[ind_of_e] = an_edge;
                    }
                }
            }
        }
        let sommetpourri = edges_of_v.extent() != 3;
        if !sommetpourri && loc_nb_g1 < 4 {
            return edge_state(&e, &self.my_ef_map);
        }
        Ok(sst)
    }

    /// `PerformExtremity(Spine)`: the tangency of the faces at the ends of the contour,
    /// the status of its break points and the break points of more than three faces;
    /// `b.perform_extremity(&spine)?`.
    pub fn perform_extremity(&mut self, spine: &SpineHandle) -> Result<(), Failure> {
        let mut nb_g1_connections = 0;
        for ii in 1..=2 {
            let (mut sst, iedge, v) = {
                let sp = lock(spine);
                if ii == 1 {
                    (sp.first_status(), 1, sp.first_vertex()?)
                } else {
                    (sp.last_status(), sp.nb_edges(), sp.last_vertex()?)
                }
            };
            let e0 = lock(spine).edges(iedge)?.clone();
            let (mut hs1, mut hs2) = (Handle::default(), Handle::default());
            self.conex_faces(spine, iedge, &mut hs1, &mut hs2)?;
            if tangent_extremity(&v, &e0, &hs1, &hs2, self.angular)? {
                lock(spine).set_tangency_extremity(true, ii == 1);
            }
            if sst == State::BreakPoint {
                sst = self.break_point_state(&v, &e0, sst, &mut nb_g1_connections)?;
                if ii == 1 {
                    lock(spine).set_first_status(sst);
                } else {
                    lock(spine).set_last_status(sst);
                }
            }
        }
        if !lock(spine).is_periodic() {
            let (fv, lv) = {
                let sp = lock(spine);
                (sp.first_vertex()?, sp.last_vertex()?)
            };
            if self.distinct_faces(&fv)? - nb_g1_connections > 3 {
                lock(spine).set_first_status(State::BreakPoint);
            }
            if self.distinct_faces(&lv)? - nb_g1_connections > 3 {
                lock(spine).set_last_status(State::BreakPoint);
            }
        }
        Ok(())
    }
}
