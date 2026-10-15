// Rust translation of ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_MakeLoops.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepOffset_MakeLoops`: the offset faces cut by their intersection edges into new
//! faces (BRepAlgo_Loop), on the offset faces, on the context faces and on the faces
//! whose edges were cut elsewhere.
use super::BRepOffsetAnalyse;
use rustycad_tkbool::brep_algo::{BRepAlgoAsDes, BRepAlgoImage, BRepAlgoLoop, VertexSubstitutes};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep::access::vertex_tshape;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Builder, Edge, Face, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::{MapOfShape, ShapeMapHasher};
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// `BRepOffset_MakeLoops`: the cutting of faces by their descendant edges, with the
/// vertex substitutions found on the way; `BRepOffsetMakeLoops::new()`.
#[derive(Default)]
pub struct BRepOffsetMakeLoops {
    my_ver_ver_map: VertexSubstitutes,
}

/// The images of the new faces and the cut edges recorded for a face and its added
/// edges.
fn record_images(
    f: &Face,
    loops: &BRepAlgoLoop,
    added_edges: &[Edge],
    image: &mut BRepAlgoImage,
) -> Result<(), Failure> {
    let nf: Vec<Shape> = loops.new_faces().iter().cloned().collect();
    // F => the new faces.
    image.bind_many(f.as_ref(), &nf)?;
    for e in added_edges {
        // E => the new edges.
        let loop_ne: Vec<Shape> = loops.new_edges(e)?.iter().cloned().collect();
        if image.has_image(e.as_ref()) {
            image.add_many(e.as_ref(), &loop_ne)?;
        } else {
            image.bind_many(e.as_ref(), &loop_ne)?;
        }
    }
    Ok(())
}

/// The cut images of an edge already cut in another face, oriented as the edge.
fn cut_images(image: &BRepAlgoImage, e: &Edge) -> Vec<Shape> {
    image
        .image(e.as_ref())
        .into_iter()
        .map(|ce| ce.oriented(e.orientation()))
        .collect()
}

/// `IsBetweenCorks(E, AsDes, LContext)`: whether every face of the edge in the AsDes is
/// a context face (true for an edge without ascendant).
fn is_between_corks(e: &Shape, as_des: &BRepAlgoAsDes, l_context: &List<Shape>) -> bool {
    if !as_des.has_ascendant(e) {
        return true;
    }
    as_des
        .ascendant(e)
        .all(|s| l_context.iter().any(|c| s.is_same(c)))
}

impl BRepOffsetMakeLoops {
    /// `BRepOffset_MakeLoops()`; `BRepOffsetMakeLoops::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// The substitution of the vertices found by the loops in the edges of the last
    /// images of the faces (the old vertex points moved to the new vertex, the AsDes
    /// updated).
    fn substitute_vertices(
        &self,
        faces: &List<Shape>,
        as_des: &BRepAlgoAsDes,
        image: &BRepAlgoImage,
        the_range: Option<&ProgressRange>,
    ) -> Result<(), Failure> {
        let builder = Builder;
        let a_ps = the_range
            .map(|r| ProgressScope::with_literal(r, "Building loops", faces.len() as f64, false));
        for f in faces.iter() {
            if let Some(ps) = &a_ps {
                if !ps.more() {
                    return Ok(());
                }
                let _ = ps.next(1.);
            }
            for if_ in image.last_image(f) {
                for e0 in Explorer::new(&if_, ShapeEnum::Edge, ShapeEnum::Shape)? {
                    let mut e = e0.clone();
                    let mut v_list = Vec::new();
                    let mut ver_exp = ShapeIterator::new(&e, true, true);
                    while ver_exp.more() {
                        v_list.push(ver_exp.value()?.clone());
                        ver_exp.next();
                    }
                    for v in &v_list {
                        let Some(new_v) = self.my_ver_ver_map.seek(v).cloned() else {
                            continue;
                        };
                        e.tshape().set_free(true);
                        let new_v = new_v.oriented(v.orientation());
                        let tv = vertex_tshape(v, "BRepOffset_MakeLoops")?;
                        let new_tv = vertex_tshape(&new_v, "BRepOffset_MakeLoops")?;
                        if tv.tolerance() > new_tv.tolerance() {
                            new_tv.set_tolerance(tv.tolerance());
                        }
                        // NCollection_List::Append(theOther) moves the points of the old
                        // vertex, leaving its list empty.
                        let points = tv.change_points(std::mem::take);
                        new_tv.change_points(|p| p.extend(points));
                        as_des.replace_shared(v, &new_v);
                        builder.remove(&mut e, v)?;
                        builder.add(&mut e, &new_v)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// `Build(LF, AsDes, Image, theImageVV, Range)`: each face cut by its descendant
    /// edges (the edges already cut taken as their images); the new faces and cut edges
    /// recorded in the image; `loops.build(&faces, &as_des, &mut image, &mut image_vv, &range)?`.
    pub fn build(
        &mut self,
        lf: &List<Shape>,
        as_des: &BRepAlgoAsDes,
        image: &mut BRepAlgoImage,
        the_image_vv: &mut BRepAlgoImage,
        the_range: &ProgressRange,
    ) -> Result<(), Failure> {
        let mut loops = BRepAlgoLoop::new();
        loops.vertices_for_substitute(&self.my_ver_ver_map);
        loops.set_image_vv(the_image_vv);
        let a_ps_outer = ProgressScope::unnamed(the_range, 2., false);
        {
            let a_ps1 = ProgressScope::with_literal(
                &a_ps_outer.next(1.),
                "Init loops",
                lf.len() as f64,
                false,
            );
            for fs in lf.iter() {
                if !a_ps1.more() {
                    return Ok(());
                }
                let f = topods::face(fs)?;
                // The initialization of the loops.
                loops.init(&f);
                // The edges of F.
                let mut added_edges = Vec::new();
                for es in as_des.descendant(fs) {
                    let e = topods::edge(&es)?;
                    if image.has_image(&es) {
                        // E was already cut in another face: its cut edges, oriented as E.
                        for ce in cut_images(image, &e) {
                            loops.add_const_edge(&topods::edge(&ce)?);
                        }
                    } else {
                        let lv: List<Shape> = as_des.descendant(&es).collect();
                        loops.add_edge(&e, &lv);
                        added_edges.push(e);
                    }
                }
                loops.perform()?;
                loops.wires_to_faces()?;
                record_images(&f, &loops, &added_edges, image)?;
                let _ = a_ps1.next(1.);
            }
        }
        loops.get_vertices_for_substitute(&mut self.my_ver_ver_map);
        if self.my_ver_ver_map.is_empty() {
            return Ok(());
        }
        self.substitute_vertices(lf, as_des, image, Some(&a_ps_outer.next(1.)))
    }

    /// `BuildOnContext(LContext, Analyse, AsDes, Image, InSide, Range)`: each context
    /// face (reversed inside) cut by its unchanged edges, the traces of the offsets and
    /// the connection edges between caps; `loops.build_on_context(&ctx, &analyse, &as_des, &mut image, true, &range)?`.
    pub fn build_on_context(
        &mut self,
        l_context: &List<Shape>,
        analyse: &BRepOffsetAnalyse,
        as_des: &BRepAlgoAsDes,
        image: &mut BRepAlgoImage,
        in_side: bool,
        the_range: &ProgressRange,
    ) -> Result<(), Failure> {
        let mut loops = BRepAlgoLoop::new();
        loops.vertices_for_substitute(&self.my_ver_ver_map);
        let mut map_extent = MapOfShape::default();
        let a_ps = ProgressScope::with_literal(
            the_range,
            "Building deepening faces",
            l_context.len() as f64,
            false,
        );
        for fs in l_context.iter() {
            if !a_ps.more() {
                return Ok(());
            }
            let f = topods::face(fs)?;
            let mut m_bound = MapOfShape::default();
            // F is reversed: it is added in myOffC, which is reversed in the result.
            if in_side {
                loops.init(&topods::face(&f.reversed())?);
            } else {
                loops.init(&f);
            }
            // The edges of F, unchanged by definition.
            for ces in Explorer::new(
                &f.oriented(Orientation::Forward),
                ShapeEnum::Edge,
                ShapeEnum::Shape,
            )? {
                let ce = topods::edge(&ces)?;
                m_bound.add(ces.clone());
                if analyse.has_ancestor(&ces) {
                    // The edges of the caps but the connections between caps.
                    if in_side {
                        loops.add_const_edge(&ce);
                    } else {
                        loops.add_const_edge(&topods::edge(&ce.reversed())?);
                    }
                }
            }
            // The traces of the offsets and the connection edges between caps.
            let mut added_edges = Vec::new();
            for es in as_des.descendant(fs) {
                let e = topods::edge(&es)?;
                if image.has_image(&es) {
                    // E was already cut in another face: its cut edges, oriented as E.
                    for mut ce in cut_images(image, &e) {
                        if map_extent.contains(&es) {
                            loops.add_const_edge(&topods::edge(&ce)?);
                            continue;
                        }
                        if !m_bound.contains(&es) {
                            ce.reverse();
                        }
                        if in_side {
                            loops.add_const_edge(&topods::edge(&ce)?);
                        } else {
                            loops.add_const_edge(&topods::edge(&ce.reversed())?);
                        }
                    }
                } else if is_between_corks(&es, as_des, l_context) && as_des.has_descendant(&es) {
                    // A connection between two caps.
                    map_extent.add(es.clone());
                    let lv: List<Shape> = if in_side {
                        as_des.descendant(&es).map(|v| v.reversed()).collect()
                    } else {
                        as_des.descendant(&es).collect()
                    };
                    loops.add_edge(&e, &lv);
                    added_edges.push(e);
                } else if is_between_corks(&es, as_des, l_context) {
                    if in_side {
                        loops.add_const_edge(&e);
                    } else {
                        loops.add_const_edge(&topods::edge(&e.reversed())?);
                    }
                } else if in_side {
                    loops.add_const_edge(&topods::edge(&e.reversed())?);
                } else {
                    loops.add_const_edge(&e);
                }
            }
            loops.perform()?;
            loops.wires_to_faces()?;
            record_images(&f, &loops, &added_edges, image)?;
            let _ = a_ps.next(1.);
        }
        loops.get_vertices_for_substitute(&mut self.my_ver_ver_map);
        if self.my_ver_ver_map.is_empty() {
            return Ok(());
        }
        self.substitute_vertices(l_context, as_des, image, None)
    }

    /// `BuildFaces(LF, AsDes, Image, Range)`: each face not yet rebuilt whose edges were
    /// cut elsewhere rebuilt from those cut edges and its other edges (their vertices
    /// moved to the rebuilt ones); `loops.build_faces(&faces, &as_des, &mut image, &range)?`.
    pub fn build_faces(
        &mut self,
        lf: &List<Shape>,
        as_des: &BRepAlgoAsDes,
        image: &mut BRepAlgoImage,
        the_range: &ProgressRange,
    ) -> Result<(), Failure> {
        let builder = rustycad_tkbrep::brep::BRepBuilder::default();
        let mut loops = BRepAlgoLoop::new();
        loops.vertices_for_substitute(&self.my_ver_ver_map);
        // The loop on all faces.
        let a_ps = ProgressScope::with_literal(the_range, "Building faces", lf.len() as f64, false);
        for fs in lf.iter() {
            if !a_ps.more() {
                return Ok(());
            }
            let _ = a_ps.next(1.);
            let f = topods::face(fs)?;
            loops.init(&f);
            let mut to_rebuild = false;
            let mut added_edges = Vec::new();
            if !image.has_image(fs) {
                // F is not yet rebuilt.
                let le: Vec<Shape> = as_des.descendant(fs).collect();
                // First whether the edges of the face were rebuilt; some vertices of
                // the rebuilt edges coincide with the old ones without being the same.
                let mut monv: DataMap<Shape, Shape, ShapeMapHasher> = DataMap::default();
                for es in &le {
                    let e = topods::edge(es)?;
                    if !image.has_image(es) {
                        continue;
                    }
                    let lce = image.image(es);
                    if lce.len() == 1 && lce[0].is_same(es) {
                        loops.add_const_edge(&topods::edge(&lce[0].oriented(e.orientation()))?);
                        continue;
                    }
                    // F should be rebuilt.
                    to_rebuild = true;
                    for c in &lce {
                        let ce = topods::edge(&c.oriented(e.orientation()))?;
                        let (ov1, ov2) = topexp::vertices(&e, false);
                        let (nv1, nv2) = topexp::vertices(&ce, false);
                        if !ov1.is_same(&nv1) {
                            monv.bind(ov1.as_ref().clone(), nv1.as_ref().clone());
                        }
                        if !ov2.is_same(&nv2) {
                            monv.bind(ov2.as_ref().clone(), nv2.as_ref().clone());
                        }
                        loops.add_const_edge(&ce);
                    }
                }
                if to_rebuild {
                    // The edges not rebuilt on other faces are added, rebuilt when their
                    // vertices were.
                    for es in &le {
                        let e = topods::edge(es)?;
                        let (f_, l_) = BRepTool::range(&e)?;
                        if image.has_image(es) {
                            continue;
                        }
                        let (ov1, ov2) = topexp::vertices(&e, false);
                        let mut lv = List::default();
                        for (ov, orientation, par) in [
                            (&ov1, Orientation::Forward, f_),
                            (&ov2, Orientation::Reversed, l_),
                        ] {
                            if let Some(vv) = monv.seek(ov.as_ref()) {
                                let vv = topods::vertex(&vv.oriented(orientation))?;
                                lv.append(vv.as_ref().clone());
                                let internal = topods::vertex(&vv.oriented(Orientation::Internal))?;
                                builder.update_vertex_on_edge(
                                    &internal,
                                    par,
                                    &e,
                                    BRepTool::vertex_tolerance(&vv)?,
                                )?;
                            }
                        }
                        if lv.is_empty() {
                            loops.add_const_edge(&e);
                        } else {
                            loops.add_edge(&e, &lv);
                            added_edges.push(e);
                        }
                    }
                }
            }
            if to_rebuild {
                // The reconstruction.
                loops.perform()?;
                loops.wires_to_faces()?;
                record_images(&f, &loops, &added_edges, image)?;
            }
        }
        loops.get_vertices_for_substitute(&mut self.my_ver_ver_map);
        if self.my_ver_ver_map.is_empty() {
            return Ok(());
        }
        self.substitute_vertices(lf, as_des, image, None)
    }
}
