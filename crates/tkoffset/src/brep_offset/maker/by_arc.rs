// Rust translation of BRepOffset_MakeOffset::BuildOffsetByArc, SelfInter and ToContext of
// ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_MakeOffset.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The offset by arcs: the offset faces joined by tubes along the convex (or concave)
//! edges and spheres at their vertices, extended to the caps.
use super::BRepOffsetMakeOffset;
use super::statics::not_bound;
use crate::brep_offset::inter3d::{OffsetFaces, ShapeShapeMap};
use crate::brep_offset::{
    BRepOffsetError, BRepOffsetOffset, BRepOffsetStatus, BRepOffsetTool, PipeOptions,
};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkbrep::toptools::{IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkfillet::chfi_ds::TypeOfConcavity;
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum, State};

/// The entries of a map in its iteration order.
pub(super) fn entries<V: Clone>(
    map: &rustycad_tkernel::ncollection::DataMap<
        Shape,
        V,
        rustycad_tkbrep::toptools::ShapeMapHasher,
    >,
) -> Result<Vec<(Shape, V)>, Failure> {
    let mut out = Vec::new();
    let mut it = map.cursor();
    while it.more() {
        out.push((it.key()?.clone(), it.value()?.clone()));
        it.next();
    }
    Ok(out)
}

/// The first interval type of an edge.
fn first_type(
    analyse: &crate::brep_offset::BRepOffsetAnalyse,
    e: &Shape,
) -> Result<Option<TypeOfConcavity>, Failure> {
    Ok(match analyse.type_of(&topods::edge(e)?)?.first() {
        Some(i) => Some(i.get_type()?),
        None => None,
    })
}

impl BRepOffsetMakeOffset {
    /// `BuildOffsetByArc(Range)`: the offset faces, the tubes of the edges of the offset
    /// side (with the connectors of the tubes of their tangent edges), the spheres at the
    /// vertices of three such edges, the extension to the caps, and the history.
    pub(super) fn build_offset_by_arc(&mut self, the_range: &ProgressRange) -> Result<(), Failure> {
        let mut done = MapOfShape::default();
        let a_ps_outer = ProgressScope::unnamed(the_range, 10., false);
        // The faces parallel to the initial faces.
        let mut map_sf = OffsetFaces::default();
        self.make_offset_faces(&mut map_sf, &a_ps_outer.next(1.))?;
        if self.my_error != BRepOffsetError::NoError {
            return Ok(());
        }
        // The tubes on the edges.
        let ot = if self.my_offset < 0. {
            TypeOfConcavity::Concave
        } else {
            TypeOfConcavity::Convex
        };
        {
            let a_ps1 = ProgressScope::with_literal(
                &a_ps_outer.next(4.),
                "Constructing tubes on edges",
                1.,
                true,
            );
            for es in Explorer::new(&self.my_face_comp, ShapeEnum::Edge, ShapeEnum::Shape)? {
                if !a_ps1.more() {
                    self.my_error = BRepOffsetError::UserBreak;
                    return Ok(());
                }
                let _ = a_ps1.next(1.);
                if !done.add(es.clone()) {
                    continue;
                }
                let e = topods::edge(&es)?;
                let anc = self.my_analyse.ancestors(&es)?.to_vec();
                let first = anc.first().cloned().unwrap_or_default();
                if anc.len() == 2 {
                    if first_type(&self.my_analyse, &es)? != Some(ot) {
                        continue;
                    }
                    let cur_offset = self
                        .my_face_offset
                        .seek(&first)
                        .copied()
                        .unwrap_or(self.my_offset);
                    let e_on1 =
                        topods::edge(&map_sf.seek(&first).ok_or_else(not_bound)?.generated(&es))?;
                    let e_on2 =
                        topods::edge(&map_sf.seek(&anc[1]).ok_or_else(not_bound)?.generated(&es))?;
                    // The connectors of the pipes of the tangent edges already built.
                    let (v1f, v1l) = topexp::vertices(&e, false);
                    let connector = |v: &topods::Vertex| -> Result<Edge, Failure> {
                        let mut tang_e = Vec::new();
                        self.my_analyse.tangent_edges(&e, v, &mut tang_e)?;
                        for t in &tang_e {
                            if let Some(o) = map_sf.seek(t) {
                                return topods::edge(&o.generated(v.as_ref()));
                            }
                        }
                        Ok(Edge::default())
                    };
                    let e1f = connector(&v1f)?;
                    let e1l = connector(&v1l)?;
                    let of = BRepOffsetOffset::from_edges_with_connectors(
                        &e,
                        &e_on1,
                        &e_on2,
                        cur_offset,
                        &e1f,
                        &e1l,
                        PipeOptions::default(),
                    )?;
                    map_sf.bind(es.clone(), of);
                } else {
                    // A free border.
                    let e_on1 = map_sf.seek(&first).ok_or_else(not_bound)?.generated(&es);
                    self.my_init_offset_edge.set_root(&es);
                    self.my_init_offset_edge
                        .bind(&es, &topods::edge(&e_on1)?.into_shape())?;
                }
            }
        }
        // The spheres on the vertices.
        done.clear(true);
        {
            let a_ps2 = ProgressScope::with_literal(
                &a_ps_outer.next(4.),
                "Constructing spheres on vertices",
                1.,
                true,
            );
            for vs in Explorer::new(&self.my_face_comp, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                if !a_ps2.more() {
                    self.my_error = BRepOffsetError::UserBreak;
                    return Ok(());
                }
                let _ = a_ps2.next(1.);
                if !done.add(vs.clone()) {
                    continue;
                }
                let v = topods::vertex(&vs)?;
                let la = self.my_analyse.ancestors(&vs)?.to_vec();
                let mut le = Vec::new();
                self.my_analyse.edges_on_vertex(&v, ot, &mut le)?;
                if le.len() >= 3 && le.len() == la.len() {
                    // The connected edges on the tubes.
                    let mut loe = List::default();
                    for s in &le {
                        loe.append(
                            map_sf
                                .seek(s)
                                .ok_or_else(not_bound)?
                                .generated(&vs)
                                .reversed(),
                        );
                    }
                    // The sphere.
                    let lla = self
                        .my_analyse
                        .ancestors(la.first().ok_or_else(not_bound)?)?;
                    let ff = lla.first().cloned().unwrap_or_default();
                    let cur_offset = self
                        .my_face_offset
                        .seek(&ff)
                        .copied()
                        .unwrap_or(self.my_offset);
                    let of = BRepOffsetOffset::from_vertex(
                        &v,
                        &loe,
                        cur_offset,
                        PipeOptions::default(),
                    )?;
                    map_sf.bind(vs.clone(), of);
                }
                // The particular processing of a vertex on at least a free border.
                let mut lbf = Vec::new();
                self.my_analyse
                    .edges_on_vertex(&v, TypeOfConcavity::FreeBound, &mut lbf)?;
                if !lbf.is_empty() {
                    let mut first = true;
                    for s in &le {
                        let g = map_sf.seek(s).ok_or_else(not_bound)?.generated(&vs);
                        if first {
                            self.my_init_offset_edge.set_root(&vs);
                            self.my_init_offset_edge.bind(&vs, &g)?;
                            first = false;
                        } else {
                            self.my_init_offset_edge.add(&vs, &g)?;
                        }
                    }
                }
            }
        }
        // The extension of the parallel faces to the context: the extended faces are
        // stored in the data structure and removed from MapSF.
        if !self.my_faces.is_empty() {
            self.extend_to_context(&mut map_sf)?;
        }
        // The update of the data structure.
        let rt = if self.my_offset < 0. {
            TypeOfConcavity::Convex
        } else {
            TypeOfConcavity::Concave
        };
        let items = entries(&map_sf)?;
        let a_ps3 = ProgressScope::unnamed(&a_ps_outer.next(1.), items.len() as f64, false);
        for (si, sf) in items {
            if !a_ps3.more() {
                self.my_error = BRepOffsetError::UserBreak;
                return Ok(());
            }
            let _ = a_ps3.next(1.);
            if sf.status() == BRepOffsetStatus::Reversed
                || sf.status() == BRepOffsetStatus::Degenerated
            {
                // The degenerated or reversed faces are not stored.
                continue;
            }
            let of = sf.face().as_ref().clone();
            self.my_init_offset_face.bind(&si, &of)?;
            // Initial <-> offset.
            self.my_init_offset_face.set_root(&si);
            // The offset face, root of the images.
            self.my_image_offset.set_root(&of);
            if si.shape_type()? == ShapeEnum::Face {
                // Each face gets the edges that bound it; the edges that generate no
                // tube or are not tangent to two faces are removed.
                for es in Explorer::new(
                    &si.oriented(Orientation::Forward),
                    ShapeEnum::Edge,
                    ShapeEnum::Shape,
                )? {
                    let Some(t) = first_type(&self.my_analyse, &es)? else {
                        continue;
                    };
                    if t != rt {
                        let oo = es.orientation();
                        let oe = sf.generated(&es);
                        self.as_des().add_shared(&of, &oe.oriented(oo));
                    }
                }
            } else {
                for e in Explorer::new(
                    &of.oriented(Orientation::Forward),
                    ShapeEnum::Edge,
                    ShapeEnum::Shape,
                )? {
                    self.as_des().add_shared(&of, &e);
                }
            }
        }
        Ok(())
    }

    /// `SelfInter(Modif)`: Standard_NotImplemented, as natively.
    #[allow(dead_code)]
    pub(super) fn self_inter(&mut self, _modif: &mut MapOfShape) -> Result<(), Failure> {
        Err(Failure::with_message(
            "Standard_NotImplemented",
            "BRepOffset_MakeOffset::SelfInter",
        ))
    }

    /// `ToContext(MapSF)`: the offset faces and tubes reaching the caps extended to them
    /// (stored in the data structure, removed from MapSF), the free borders updated.
    pub(super) fn extend_to_context(&mut self, map_sf: &mut OffsetFaces) -> Result<(), Failure> {
        let mut created = ShapeShapeMap::default();
        let mut mef = ShapeShapeMap::default();
        let mut faces_to_build = IndexedMapOfShape::default();
        let side = State::Out;
        // The edges and faces rebuilt by intersection.
        for j in 1..=self.my_faces.extent() {
            let cf = self.my_faces.find_key(j)?.clone();
            for es in Explorer::new(
                &cf.oriented(Orientation::Forward),
                ShapeEnum::Edge,
                ShapeEnum::Shape,
            )? {
                if !self.my_analyse.has_ancestor(&es) {
                    continue;
                }
                for a in self.my_analyse.ancestors(&es)?.to_vec() {
                    let of = map_sf.seek(&a).ok_or_else(not_bound)?;
                    faces_to_build.add(a.clone());
                    mef.bind(of.generated(&es), cf.clone());
                }
                let (va, vb) = topexp::vertices(&topods::edge(&es)?, false);
                for v in [va, vb] {
                    for ev in self.my_analyse.ancestors(v.as_ref())?.to_vec() {
                        if let Some(of) = map_sf.seek(&ev) {
                            faces_to_build.add(ev.clone());
                            mef.bind(of.generated(v.as_ref()), cf.clone());
                        }
                    }
                }
            }
        }
        // The reconstruction of the faces.
        let rt = if self.my_offset < 0. {
            TypeOfConcavity::Convex
        } else {
            TypeOfConcavity::Concave
        };
        for j in 1..=faces_to_build.extent() {
            let s = faces_to_build.find_key(j)?.clone();
            let bof = map_sf.seek(&s).ok_or_else(not_bound)?.clone();
            let f = bof.face().clone();
            let mut nf = topods::Face::default();
            BRepOffsetTool::extent_face(&f, &mut created, &mut mef, side, self.my_tol, &mut nf)?;
            map_sf.unbind(&s);
            // The update of the data structure.
            self.my_init_offset_face.bind(&s, nf.as_ref())?;
            // Initial <-> offset.
            self.my_init_offset_face.set_root(&s);
            self.my_image_offset.set_root(nf.as_ref());
            if s.shape_type()? == ShapeEnum::Face {
                for es in Explorer::new(
                    &s.oriented(Orientation::Forward),
                    ShapeEnum::Edge,
                    ShapeEnum::Shape,
                )? {
                    let t = first_type(&self.my_analyse, &es)?;
                    let mut oe = bof.generated(&es);
                    let or = es.orientation();
                    oe.set_orientation(or);
                    if t.is_some_and(|t| t != rt) {
                        if let Some(ne) = created.seek(&oe).cloned() {
                            let mut ne = ne;
                            if ne.orientation() == Orientation::Reversed {
                                ne.set_orientation(top_abs::reverse(or));
                            } else {
                                ne.set_orientation(or);
                            }
                            self.as_des().add_shared(nf.as_ref(), &ne);
                        } else {
                            self.as_des().add_shared(nf.as_ref(), &oe);
                        }
                    }
                }
            } else {
                // A tube.
                for e in Explorer::new(
                    &nf.oriented(Orientation::Forward),
                    ShapeEnum::Edge,
                    ShapeEnum::Shape,
                )? {
                    self.as_des().add_shared(nf.as_ref(), &e);
                }
            }
            map_sf.unbind(&s);
        }
        // The update of the free borders.
        for (oe, ne) in super::by_arc::entries(&created)? {
            if !self.my_init_offset_edge.is_image(&oe) {
                continue;
            }
            let e = self.my_init_offset_edge.image_from(&oe)?;
            let or = self
                .my_init_offset_edge
                .image(&e)
                .first()
                .ok_or_else(not_bound)?
                .orientation();
            let mut ne = ne;
            if ne.orientation() == Orientation::Reversed {
                ne.set_orientation(top_abs::reverse(or));
            } else {
                ne.set_orientation(or);
            }
            self.my_init_offset_edge.remove(&oe)?;
            self.my_init_offset_edge.bind(&e, &ne)?;
        }
        Ok(())
    }
}
