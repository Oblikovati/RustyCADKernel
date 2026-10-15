// Rust translation of BRepOffset_Inter3d::ContextIntByInt and ContextIntByArc of
// ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_Inter3d.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The intersections of the offset faces with the context faces (the caps of a thick
//! solid or a deepened shape).
use super::{BRepOffsetInter3d, OffsetFaces, ShapeShapeMap, extent_edge, first_image};
use crate::brep_offset::{BRepOffsetAnalyse, BRepOffsetTool, EnlargeFaceOptions};
use rustycad_tkbool::brep_algo::BRepAlgoImage;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Compound, Edge, Face, Shape};
use rustycad_tkbrep::toptools::{IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum, State};

/// The extended edge of a context edge outside the initial shape, with its vertices
/// as descendants and internal vertices on it.
fn extended_context_edge(inter: &BRepOffsetInter3d, e: &Edge) -> Result<Edge, Failure> {
    let builder = BRepBuilder::default();
    let (f, l) = BRepTool::range(e)?;
    let tol = BRepTool::edge_tolerance(e)?;
    let mut ne = extent_edge(e)?;
    let (v1, v2) = topexp::vertices(e, false);
    ne.set_orientation(Orientation::Forward);
    inter
        .as_des()
        .add_shared(ne.as_ref(), &v1.oriented(Orientation::Reversed));
    inter
        .as_des()
        .add_shared(ne.as_ref(), &v2.oriented(Orientation::Forward));
    builder.update_vertex_on_edge(
        &topods::vertex(&v1.oriented(Orientation::Internal))?,
        f,
        &ne,
        tol,
    )?;
    builder.update_vertex_on_edge(
        &topods::vertex(&v2.oriented(Orientation::Internal))?,
        l,
        &ne,
        tol,
    )?;
    ne.set_orientation(e.orientation());
    Ok(ne)
}

impl BRepOffsetInter3d {
    /// `ContextIntByInt(ContextFaces, ExtentContext, MapSF, Analyse, MES, Build, Failed,
    /// Range, bIsPlanar)`: the sections of each context face (enlarged when
    /// `ExtentContext`) with the enlarged offset faces of the faces at its edges (and
    /// vertices, for planar shapes); the edges of the context faces outside the initial
    /// shape are kept (extended when `ExtentContext`);
    /// `inter.context_int_by_int(&faces, true, &map_sf, &analyse, &mut mes, &mut build, &mut failed, &range, false)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn context_int_by_int(
        &mut self,
        context_faces: &IndexedMapOfShape,
        extent_context: bool,
        map_sf: &OffsetFaces,
        analyse: &BRepOffsetAnalyse,
        mes: &mut ShapeShapeMap,
        build: &mut ShapeShapeMap,
        failed: &mut List<Shape>,
        the_range: &ProgressRange,
        b_is_planar: bool,
    ) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let a_nb = context_faces.extent();
        for i in 1..=a_nb {
            let cf = topods::face(context_faces.find_key(i)?)?;
            self.my_touched.add(cf.as_ref().clone());
            if extent_context {
                let mut nf = Face::default();
                let options = EnlargeFaceOptions::with_change_geom(false);
                BRepOffsetTool::en_large_face(&cf, &mut nf, options)?;
                mes.bind(cf.as_ref().clone(), nf.as_ref().clone());
            }
        }
        let side = State::Out;
        let a_ps = ProgressScope::with_literal(
            the_range,
            "Intersecting with deepening faces",
            a_nb as f64,
            false,
        );
        for i in 1..=a_nb {
            if !a_ps.more() {
                return Ok(());
            }
            let _ = a_ps.next(1.);
            let cf = topods::face(context_faces.find_key(i)?)?;
            let wcf = if extent_context {
                topods::face(mes.seek(cf.as_ref()).ok_or_else(not_bound)?)?
            } else {
                cf.clone()
            };
            let mut ve_map = IndexedMapOfShape::default();
            topexp::map_shapes_of_type(
                &cf.oriented(Orientation::Forward),
                ShapeEnum::Edge,
                &mut ve_map,
            )?;
            if b_is_planar {
                topexp::map_shapes_of_type(
                    &cf.oriented(Orientation::Forward),
                    ShapeEnum::Vertex,
                    &mut ve_map,
                )?;
            }
            for j in 1..=ve_map.extent() {
                let a_s = ve_map.find_key(j)?.clone();
                let mut e = Edge::default();
                let mut anc: Vec<Shape> = Vec::new();
                if a_s.shape_type()? == ShapeEnum::Edge {
                    // The faces connected by the edge.
                    e = topods::edge(&a_s)?;
                    if !analyse.has_ancestor(&a_s) {
                        // The edges of the context faces outside the initial shape can
                        // appear in the result.
                        if !extent_context {
                            self.as_des().add_shared(cf.as_ref(), e.as_ref());
                            self.my_new_edges.add(e.as_ref().clone());
                        } else if let Some(ne) = mes.seek(e.as_ref()) {
                            self.as_des()
                                .add_shared(cf.as_ref(), &ne.oriented(e.orientation()));
                        } else {
                            let ne = extended_context_edge(self, &e)?;
                            self.as_des().add_shared(cf.as_ref(), ne.as_ref());
                            self.my_new_edges.add(ne.as_ref().clone());
                            mes.bind(e.as_ref().clone(), ne.as_ref().clone());
                        }
                        continue;
                    }
                    anc = analyse.ancestors(&a_s)?.to_vec();
                } else {
                    // The faces connected by the vertex.
                    if !analyse.has_ancestor(&a_s) {
                        continue;
                    }
                    for a_e in analyse.ancestors(&a_s)? {
                        if BRepTool::degenerated(&topods::edge(a_e)?)? || ve_map.contains(a_e) {
                            continue;
                        }
                        for a_f in analyse.ancestors(a_e)? {
                            let b_add = !Explorer::new(a_f, ShapeEnum::Edge, ShapeEnum::Shape)?
                                .any(|a_ef| ve_map.contains(&a_ef));
                            if b_add {
                                anc.push(a_f.clone());
                            }
                        }
                    }
                }
                for fs in &anc {
                    let f = topods::face(fs)?;
                    let o = map_sf.seek(fs).ok_or_else(not_bound)?;
                    let of = o.face().clone();
                    let nf = if let Some(nf) = mes.seek(of.as_ref()) {
                        topods::face(nf)?
                    } else {
                        let mut nf = Face::default();
                        let options = EnlargeFaceOptions {
                            update_pcurve: true,
                            ..EnlargeFaceOptions::with_change_geom(true)
                        };
                        BRepOffsetTool::en_large_face(&of, &mut nf, options)?;
                        mes.bind(of.as_ref().clone(), nf.as_ref().clone());
                        nf
                    };
                    if self.is_done(&nf, &cf) {
                        continue;
                    }
                    let (mut l_int1, mut l_int2) = (List::default(), List::default());
                    BRepOffsetTool::inter3d(
                        &wcf,
                        &nf,
                        &mut l_int1,
                        &mut l_int2,
                        side,
                        &e,
                        &cf,
                        &f,
                    )?;
                    self.set_done(&nf, &cf);
                    if l_int1.is_empty() {
                        failed.append(a_s.clone());
                        continue;
                    }
                    self.store(&cf, &nf, &l_int1, &l_int2);
                    if l_int1.len() == 1 && build.seek(&a_s).is_none() {
                        build.bind(a_s.clone(), l_int1.first().cloned().unwrap_or_default());
                    } else {
                        let mut c = Compound::default();
                        builder.make_compound(&mut c);
                        let mut c = c.into_shape();
                        if let Some(a_se) = build.seek(&a_s) {
                            for a_ne in Explorer::new(a_se, ShapeEnum::Edge, ShapeEnum::Shape)? {
                                builder.add(&mut c, &a_ne)?;
                            }
                        }
                        for s in l_int1.iter() {
                            builder.add(&mut c, s)?;
                        }
                        build.bind(a_s.clone(), c);
                    }
                }
            }
        }
        Ok(())
    }

    /// `ContextIntByArc(ContextFaces, InSide, Analyse, InitOffsetFace, InitOffsetEdge,
    /// Range)`: the offset edges on each context face (the edges outside the initial
    /// shape kept inside, extended outside), then the faces of the ancestors of its
    /// vertices projected or intersected on it;
    /// `inter.context_int_by_arc(&faces, false, &analyse, &face_image, &mut edge_image, &range)?`.
    pub fn context_int_by_arc(
        &mut self,
        context_faces: &IndexedMapOfShape,
        in_side: bool,
        analyse: &BRepOffsetAnalyse,
        init_offset_face: &BRepAlgoImage,
        init_offset_edge: &mut BRepAlgoImage,
        the_range: &ProgressRange,
    ) -> Result<(), Failure> {
        let (mut l_int1, mut l_int2) = (List::default(), List::default());
        let mut mv = MapOfShape::default();
        let null_edge = Edge::default();
        let null_face = Face::default();
        for j in 1..=context_faces.extent() {
            self.my_touched.add(context_faces.find_key(j)?.clone());
        }
        let a_ps = ProgressScope::with_literal(
            the_range,
            "Intersecting with deepening faces",
            context_faces.extent() as f64,
            false,
        );
        for j in 1..=context_faces.extent() {
            if !a_ps.more() {
                return Ok(());
            }
            let _ = a_ps.next(1.);
            let cf = topods::face(context_faces.find_key(j)?)?;
            for es in Explorer::new(
                &cf.oriented(Orientation::Forward),
                ShapeEnum::Edge,
                ShapeEnum::Shape,
            )? {
                let e = topods::edge(&es)?;
                if !analyse.has_ancestor(&es) {
                    if in_side {
                        self.as_des().add_shared(cf.as_ref(), e.as_ref());
                    } else if !init_offset_edge.has_image(&es) {
                        let ne = extended_context_edge(self, &e)?;
                        self.as_des().add_shared(cf.as_ref(), ne.as_ref());
                        init_offset_edge.bind(&es, ne.as_ref())?;
                    } else {
                        let ne = topods::edge(&first_image(init_offset_edge, &es)?)?;
                        self.as_des()
                            .add_shared(cf.as_ref(), &ne.oriented(e.orientation()));
                    }
                    continue;
                }
                // OF1, the parallel face of the ancestor of E.
                let si = analyse.ancestors(&es)?.first().cloned().unwrap_or_default();
                let of1 = topods::face(&first_image(init_offset_face, &si)?)?;
                let oe = topods::edge(&first_image(init_offset_edge, &es)?)?;
                if !has_pcurves(&oe, &cf, &of1)? {
                    continue;
                }
                // The update of OE on the cap CF.
                self.store_cap_section(&oe, &cf, &of1, &mut l_int1, &mut l_int2)?;
                // The offsets on the ancestors of the vertices.
                let (va, vb) = topexp::vertices(&e, false);
                for v in [va, vb] {
                    if !mv.add(v.as_ref().clone()) {
                        continue;
                    }
                    for evs in analyse.ancestors(v.as_ref())? {
                        if !init_offset_face.has_image(evs) {
                            continue;
                        }
                        // OF1, the parallel face of an ancestor edge of V.
                        let of1 = topods::face(&first_image(init_offset_face, evs)?)?;
                        let oe = topods::edge(&first_image(init_offset_edge, v.as_ref())?)?;
                        if !has_pcurves(&oe, &cf, &of1)? {
                            continue;
                        }
                        self.store_cap_section(&oe, &cf, &of1, &mut l_int1, &mut l_int2)?;
                    }
                }
            }
            for vs in Explorer::new(
                &cf.oriented(Orientation::Forward),
                ShapeEnum::Vertex,
                ShapeEnum::Shape,
            )? {
                if !analyse.has_ancestor(&vs) {
                    continue;
                }
                for evs in analyse.ancestors(&vs)? {
                    for fev in analyse.ancestors(evs)? {
                        // OF1, the parallel face of a face ancestor of V.
                        let of1 = topods::face(&first_image(init_offset_face, fev)?)?;
                        if self.is_done(&of1, &cf) {
                            continue;
                        }
                        // Whether an edge of OF1 has no trace on CF: then intersect.
                        let mut loe = List::default();
                        for s in Explorer::new(
                            &of1.oriented(Orientation::Forward),
                            ShapeEnum::Edge,
                            ShapeEnum::Shape,
                        )? {
                            loe.append(s);
                        }
                        let side = self.my_side;
                        if !BRepOffsetTool::try_project(
                            &cf,
                            &of1,
                            &loe,
                            &mut l_int1,
                            &mut l_int2,
                            side,
                            self.my_tol,
                        )? || l_int1.is_empty()
                        {
                            BRepOffsetTool::inter3d(
                                &cf,
                                &of1,
                                &mut l_int1,
                                &mut l_int2,
                                side,
                                &null_edge,
                                &null_face,
                                &null_face,
                            )?;
                        }
                        self.store(&cf, &of1, &l_int1, &l_int2);
                    }
                }
            }
        }
        Ok(())
    }

    /// The offset edge as the section of the cap and the parallel face, oriented
    /// reversed to its section orientation on the cap.
    fn store_cap_section(
        &mut self,
        oe: &Edge,
        cf: &Face,
        of1: &Face,
        l_int1: &mut List<Shape>,
        l_int2: &mut List<Shape>,
    ) -> Result<(), Failure> {
        let (o1, _) = BRepOffsetTool::orient_section(oe, cf, of1)?;
        l_int1.clear();
        l_int1.append(oe.oriented(top_abs::reverse(o1)));
        l_int2.clear();
        self.store(cf, of1, l_int1, l_int2);
        Ok(())
    }
}

/// Whether the edge has p-curves on both faces.
fn has_pcurves(oe: &Edge, cf: &Face, of1: &Face) -> Result<bool, Failure> {
    let c1 = BRepTool::curve_on_face(oe, cf)?;
    let c2 = BRepTool::curve_on_face(oe, of1)?;
    Ok(!c1.pcurve.is_null() && !c2.pcurve.is_null())
}

fn not_bound() -> Failure {
    Failure::with_message("Standard_NoSuchObject", "NCollection_DataMap::Find")
}
