// Rust translation of BRepOffset_Inter2d::ConnexIntByInt and ConnexIntByIntInVert with
// the static CommonVertex, DefineClosedness, GetEdgesOrientedInFace and UpdateVertex of
// ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_Inter2d.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The intersections of the images of consecutive edges of a face on its offset face
//! (the intersection mode of the offset algorithm), and of the edges generated from its
//! vertices.
use super::{BRepOffsetInter2d, CoincidentVertices, EdgeIntersections, ref_edge_inter, store};
use crate::brep_offset::inter3d::ShapeShapeMap;
use crate::brep_offset::tool::live;
use crate::brep_offset::{BRepOffsetAnalyse, BRepOffsetOffset, BRepOffsetTool, VertexEdges};
use rustycad_tkbool::brep_algo::{BRepAlgoAsDes, BRepAlgoImage};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorCurve2d, BRepAdaptorSurface};
use rustycad_tkbrep::brep_tools::{self, WireExplorer};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Direction2, Vector2};

/// `CommonVertex(E1, E2)`: the common vertex of two edges (in their orientation),
/// the last vertex of the first edge first; null otherwise.
fn common_vertex(e1: &Edge, e2: &Edge) -> Vertex {
    let (v1a, v1b) = topexp::vertices(e1, true);
    let (v2a, v2b) = topexp::vertices(e2, true);
    // The first edge is the current one, the second the next one: the last vertex of
    // the first edge is checked first.
    if v1b.is_same(&v2a) || v1b.is_same(&v2b) {
        return v1b;
    }
    if v1a.is_same(&v2a) || v1a.is_same(&v2b) {
        return v1a;
    }
    Vertex::default()
}

/// `DefineClosedness(theFace)`: 1 when the first really closed edge of the face runs
/// along V (its p-curve parallel to OY), 2 when along U, 0 without one.
fn define_closedness(the_face: &Face) -> Result<i32, Failure> {
    for e in Explorer::new(the_face.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)? {
        let an_edge = topods::edge(&e)?;
        if brep_tools::is_really_closed(&an_edge, the_face)? {
            let pc = BRepTool::curve_on_face(&an_edge, the_face)?;
            let a_tangent = live(&pc.pcurve, "pcurve")?.dn(pc.first, 1)?;
            let a_cross_prod1 = a_tangent.crossed(Vector2::from(Direction2::X));
            let a_cross_prod2 = a_tangent.crossed(Vector2::from(Direction2::Y));
            if a_cross_prod2.abs() < a_cross_prod1.abs() {
                // The p-curve is parallel to OY.
                return Ok(1);
            }
            return Ok(2);
        }
    }
    Ok(0)
}

/// `GetEdgesOrientedInFace(theShape, theFace, theAsDes, theSeqEdges)`: the edges of the
/// shape as descendants of the face (with their orientation there), chained from the
/// first free start (or, for a closed set on a closed face, from the vertex of the
/// greatest jump across the seam).
fn get_edges_oriented_in_face(
    the_shape: &Shape,
    the_face: &Face,
    the_as_des: &BRepAlgoAsDes,
) -> Result<Vec<Shape>, Failure> {
    let a_edges: Vec<Shape> = the_as_des.descendant(the_face.as_ref()).collect();
    let mut the_seq_edges = Vec::new();
    for an_edge in Explorer::new(the_shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if let Some(e) = a_edges.iter().find(|x| x.is_same(&an_edge)) {
            the_seq_edges.push(e.clone());
        }
    }
    if the_seq_edges.len() == 1 {
        return Ok(the_seq_edges);
    }
    let mut a_ve_map = IndexedDataMapOfShapeListOfShape::default();
    for e in &the_seq_edges {
        topexp::map_shapes_and_ancestors(e, ShapeEnum::Vertex, ShapeEnum::Edge, &mut a_ve_map)?;
    }
    let mut a_first_vertex = Shape::default();
    let mut a_first_edge = Shape::default();
    for ii in 1..=a_ve_map.extent() {
        let a_vertex = a_ve_map.find_key(ii)?;
        let a_elist = a_ve_map.find_from_index(ii)?;
        if a_elist.len() == 1 {
            let an_edge = topods::edge(a_elist.first().expect("one edge"))?;
            let (a_v1, _) = topexp::vertices(&an_edge, true);
            if a_v1.is_same(a_vertex) {
                a_first_vertex = a_vertex.clone();
                a_first_edge = an_edge.as_ref().clone();
                break;
            }
        }
    }
    if a_first_edge.is_null() {
        // A closed set of edges.
        let ind_coord = define_closedness(the_face)?;
        if ind_coord != 0 {
            let mut a_max_delta = 0.;
            for ii in 1..=a_ve_map.extent() {
                let a_vertex = topods::vertex(a_ve_map.find_key(ii)?)?;
                let a_elist = a_ve_map.find_from_index(ii)?;
                let an_edge1 = topods::edge(a_elist.first().expect("edge"))?;
                let an_edge2 = topods::edge(a_elist.last().expect("edge"))?;
                let a_param1 = BRepTool::parameter(&a_vertex, &an_edge1)?;
                let a_param2 = BRepTool::parameter(&a_vertex, &an_edge2)?;
                let a_pnt1 =
                    BRepAdaptorCurve2d::with_edge_on_face(&an_edge1, the_face)?.value(a_param1)?;
                let a_pnt2 =
                    BRepAdaptorCurve2d::with_edge_on_face(&an_edge2, the_face)?.value(a_param2)?;
                let a_delta = (a_pnt1.coord(ind_coord)? - a_pnt2.coord(ind_coord)?).abs();
                if a_delta > a_max_delta {
                    a_max_delta = a_delta;
                    a_first_vertex = a_vertex.as_ref().clone();
                }
            }
            let a_elist = a_ve_map.find_from_key(&a_first_vertex)?;
            for s in a_elist.iter() {
                let an_edge = topods::edge(s)?;
                let (a_v1, _) = topexp::vertices(&an_edge, true);
                if a_v1.is_same(&a_first_vertex) {
                    a_first_edge = s.clone();
                    break;
                }
            }
        }
    }
    let a_nb_edges = the_seq_edges.len();
    the_seq_edges.clear();
    the_seq_edges.push(a_first_edge.clone());
    let mut an_edge = topods::edge(&a_first_edge)?;
    loop {
        let a_last_vertex = topexp::last_vertex(&an_edge, true);
        if a_last_vertex.is_same(&a_first_vertex) {
            break;
        }
        let a_elist = a_ve_map.find_from_key(a_last_vertex.as_ref())?;
        if a_elist.len() == 1 {
            break;
        }
        let (first, last) = (
            a_elist.first().expect("edge"),
            a_elist.last().expect("edge"),
        );
        an_edge = if first.is_same(an_edge.as_ref()) {
            topods::edge(last)?
        } else {
            topods::edge(first)?
        };
        the_seq_edges.push(an_edge.as_ref().clone());
        if the_seq_edges.len() == a_nb_edges {
            break;
        }
    }
    Ok(the_seq_edges)
}

/// `UpdateVertex(V, OE, NE, TolConf)`: the vertex at an end of the old edge given its
/// parameter on the new edge when that end lies on it; whether it does.
fn update_vertex(v: &Vertex, oe: &Edge, ne: &Edge, tol_conf: f64) -> Result<bool, Failure> {
    let oc = BRepAdaptorCurve::with_edge(oe)?;
    let nc = BRepAdaptorCurve::with_edge(ne)?;
    let (of, ol) = (oc.first_parameter()?, oc.last_parameter()?);
    let (nf, nl) = (nc.first_parameter()?, nc.last_parameter()?);
    let mut u = 0.;
    let par_tol = precision::p_confusion();
    let p = BRepTool::pnt(v)?;
    let mut ok = false;
    for end in [of, ol] {
        if p.distance(&oc.value(end)?) < tol_conf
            && end >= nf + par_tol
            && end <= nl + par_tol
            && p.distance(&nc.value(end)?) < tol_conf
        {
            ok = true;
            u = end;
        }
    }
    if ok {
        let internal = topods::vertex(&v.oriented(Orientation::Internal))?;
        BRepBuilder::default().update_vertex_on_edge(
            &internal,
            u,
            ne,
            BRepTool::edge_tolerance(ne)?,
        )?;
    }
    Ok(ok)
}

/// An edge of a map; Standard_NoSuchObject otherwise.
fn bound(map: &ShapeShapeMap, s: &Shape) -> Result<Shape, Failure> {
    map.seek(s)
        .cloned()
        .ok_or_else(|| Failure::with_message("Standard_NoSuchObject", "NCollection_DataMap::Find"))
}

/// The offset face of the face, or its enlarged face when bound.
fn offset_face(ofi: &BRepOffsetOffset, mes: &ShapeShapeMap) -> Result<Face, Failure> {
    let fio = ofi.face().as_ref().clone();
    topods::face(&mes.seek(&fio).cloned().unwrap_or(fio))
}

impl BRepOffsetInter2d {
    /// `ConnexIntByInt(FI, OFI, MES, Build, theAsDes, AsDes2d, Offset, Tol, Analyse,
    /// FacesWithVerts, theImageVV, theEdgeIntEdges, theDMVV, Range)`: the offset edges
    /// of the edges at vertices with intersection edges extended (in `MES`), then at
    /// each vertex of the wires of the face the intersection of the images of its two
    /// edges on the offset face (the intersection edges, else the extended offset edge),
    /// or the vertex put on the extended edges; false when an extension fails;
    /// `BRepOffsetInter2d::connex_int_by_int(&fi, &mut ofi, &mut mes, &build, &as_des, &as_des2d, 1.0, 1e-7, &analyse, &mut fwv, &mut image, &mut ei, &mut dmvv, &range)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn connex_int_by_int(
        fi: &Face,
        ofi: &mut BRepOffsetOffset,
        mes: &mut ShapeShapeMap,
        build: &ShapeShapeMap,
        the_as_des: &BRepAlgoAsDes,
        as_des2d: &BRepAlgoAsDes,
        offset: f64,
        tol: f64,
        analyse: &BRepOffsetAnalyse,
        faces_with_verts: &mut IndexedMapOfShape,
        the_image_vv: &mut BRepAlgoImage,
        the_edge_int_edges: &mut EdgeIntersections,
        the_dmvv: &mut CoincidentVertices,
        the_range: &ProgressRange,
    ) -> Result<bool, Failure> {
        let mut mve = VertexEdges::default();
        BRepOffsetTool::map_vertex_edges(fi.as_ref(), &mut mve)?;
        let a_ps = ProgressScope::with_literal(
            the_range,
            "Intersecting edges obtained as intersection of faces",
            1.,
            true,
        );
        // The extension of the edges.
        let mut it = mve.cursor();
        while it.more() {
            if !a_ps.more() {
                return Ok(false);
            }
            let l = it.value()?;
            it.next();
            let ya_build = l.iter().any(|s| build.seek(s).is_some());
            if !ya_build {
                continue;
            }
            for s in l.iter() {
                let ei = topods::edge(s)?;
                if ei.orientation() != Orientation::Forward
                    && ei.orientation() != Orientation::Reversed
                {
                    continue;
                }
                let oe = topods::edge(&ofi.generated(ei.as_ref()))?;
                if mes.seek(oe.as_ref()).is_none() && build.seek(ei.as_ref()).is_none() {
                    let mut ne = Edge::default();
                    if !Self::extent_edge(&oe, &mut ne, offset)? {
                        return Ok(false);
                    }
                    mes.bind(oe.as_ref().clone(), ne.as_ref().clone());
                }
            }
        }
        let fio = offset_face(ofi, mes)?;
        let b_asurf = BRepAdaptorSurface::with_face(&fio, true)?;
        for ws in Explorer::new(
            &fi.oriented(Orientation::Forward),
            ShapeEnum::Wire,
            ShapeEnum::Shape,
        )? {
            if !a_ps.more() {
                return Ok(false);
            }
            let w = topods::wire(&ws.oriented(Orientation::Forward))?;
            let mut wexp = WireExplorer::with_wire_on_face(
                &w,
                &topods::face(&fi.oriented(Orientation::Forward))?,
            )?;
            let _ = a_ps.next(1.);
            if !wexp.more() {
                // The explorer has no edge.
                continue;
            }
            let first_e = wexp.current().clone();
            let mut cur_e = first_e.clone();
            let mut end = false;
            while !end {
                wexp.next()?;
                let mut next_e = if wexp.more() {
                    wexp.current().clone()
                } else {
                    end = true;
                    first_e.clone()
                };
                if cur_e.is_same(&next_e) {
                    continue;
                }
                let vref = common_vertex(&cur_e, &next_e);
                cur_e = analyse.edge_replacement(fi, &cur_e).clone();
                next_e = analyse.edge_replacement(fi, &next_e).clone();
                let ceo = topods::edge(&ofi.generated(cur_e.as_ref()))?;
                let neo = topods::edge(&ofi.generated(next_e.as_ref()))?;
                // The intersection of the images of CurE and NextE.
                let (cur_b, next_b) = (build.seek(cur_e.as_ref()), build.seek(next_e.as_ref()));
                let choice = if cur_b.is_some() && next_b.is_some() {
                    1
                } else if cur_b.is_some() && mes.seek(neo.as_ref()).is_some() {
                    2
                } else if next_b.is_some() && mes.seek(ceo.as_ref()).is_some() {
                    3
                } else {
                    0
                };
                if choice == 0 {
                    let v = common_vertex(&ceo, &neo);
                    if !v.is_null() {
                        for o in [&ceo, &neo] {
                            if let Some(me) = mes.seek(o.as_ref()).cloned() {
                                update_vertex(&v, o, &topods::edge(&me)?, tol)?;
                                as_des2d.add_shared(&me, v.as_ref());
                            }
                        }
                    }
                    cur_e = wexp.current().clone();
                    continue;
                }
                let (ne1seq, ne2seq, mut an_or1, mut an_or2) = match choice {
                    1 => (
                        get_edges_oriented_in_face(
                            &bound(build, cur_e.as_ref())?,
                            &fio,
                            the_as_des,
                        )?,
                        get_edges_oriented_in_face(
                            &bound(build, next_e.as_ref())?,
                            &fio,
                            the_as_des,
                        )?,
                        Orientation::Reversed,
                        Orientation::Forward,
                    ),
                    2 => (
                        get_edges_oriented_in_face(
                            &bound(build, cur_e.as_ref())?,
                            &fio,
                            the_as_des,
                        )?,
                        vec![bound(mes, neo.as_ref())?.oriented(next_e.orientation())],
                        Orientation::Reversed,
                        Orientation::Forward,
                    ),
                    _ => (
                        get_edges_oriented_in_face(
                            &bound(build, next_e.as_ref())?,
                            &fio,
                            the_as_des,
                        )?,
                        vec![bound(mes, ceo.as_ref())?.oriented(cur_e.orientation())],
                        Orientation::Forward,
                        Orientation::Reversed,
                    ),
                };
                // NE1 and NE2 can be compounds of edges.
                let missing = || {
                    Failure::with_message("Standard_NoSuchObject", "NCollection_Sequence::Value")
                };
                let (a_e1, a_e2) = if choice == 1 || choice == 2 {
                    (
                        ne1seq.last().ok_or_else(missing)?,
                        ne2seq.first().ok_or_else(missing)?,
                    )
                } else {
                    (
                        ne1seq.first().ok_or_else(missing)?,
                        ne2seq.last().ok_or_else(missing)?,
                    )
                };
                let (a_e1, a_e2) = (topods::edge(a_e1)?, topods::edge(a_e2)?);
                if a_e1.orientation() == Orientation::Reversed {
                    an_or1 = top_abs::reverse(an_or1);
                }
                if a_e2.orientation() == Orientation::Reversed {
                    an_or2 = top_abs::reverse(an_or2);
                }
                ref_edge_inter(
                    &fio,
                    &b_asurf,
                    &a_e1,
                    &a_e2,
                    (an_or1, an_or2),
                    as_des2d,
                    tol,
                    true,
                    &vref,
                    the_image_vv,
                    the_dmvv,
                )?;
                the_edge_int_edges
                    .try_emplaced(a_e1.as_ref().clone(), List::default)
                    .append(a_e2.as_ref().clone());
                the_edge_int_edges
                    .try_emplaced(a_e2.as_ref().clone(), List::default)
                    .append(a_e1.as_ref().clone());
                // Whether some of the offset edges were generated from the common vertex.
                if build.seek(vref.as_ref()).is_some() {
                    faces_with_verts.add(fi.as_ref().clone());
                }
                cur_e = wexp.current().clone();
            }
        }
        Ok(true)
    }

    /// `ConnexIntByIntInVert(FI, OFI, MES, Build, AsDes, AsDes2d, Tol, Analyse, theDMVV,
    /// Range)`: at each vertex of the face with edges generated from it, those edges
    /// intersected with the images of the two edges of the vertex and with each other
    /// on the offset face (a coinciding edge trimmed as the image);
    /// `BRepOffsetInter2d::connex_int_by_int_in_vert(&fi, &mut ofi, &mut mes, &build, &as_des, &as_des2d, 1e-7, &analyse, &mut dmvv, &range)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn connex_int_by_int_in_vert(
        fi: &Face,
        ofi: &mut BRepOffsetOffset,
        mes: &mut ShapeShapeMap,
        build: &ShapeShapeMap,
        as_des: &BRepAlgoAsDes,
        as_des2d: &BRepAlgoAsDes,
        tol: f64,
        analyse: &BRepOffsetAnalyse,
        the_dmvv: &mut CoincidentVertices,
        the_range: &ProgressRange,
    ) -> Result<(), Failure> {
        let fio = offset_face(ofi, mes)?;
        let mut a_me = MapOfShape::default();
        for e in as_des.descendant(fio.as_ref()) {
            a_me.add(e);
        }
        let b_asurf = BRepAdaptorSurface::with_face(&fio, true)?;
        let a_ps = ProgressScope::with_literal(
            the_range,
            "Intersecting edges created from vertices",
            1.,
            true,
        );
        for ws in Explorer::new(
            &fi.oriented(Orientation::Forward),
            ShapeEnum::Wire,
            ShapeEnum::Shape,
        )? {
            if !a_ps.more() {
                return Ok(());
            }
            let w = topods::wire(&ws.oriented(Orientation::Forward))?;
            let mut wexp = WireExplorer::with_wire_on_face(
                &w,
                &topods::face(&fi.oriented(Orientation::Forward))?,
            )?;
            let _ = a_ps.next(1.);
            if !wexp.more() {
                continue;
            }
            let first_e = wexp.current().clone();
            let mut cur_e = first_e.clone();
            let mut end = false;
            while !end {
                wexp.next()?;
                let mut next_e = if wexp.more() {
                    wexp.current().clone()
                } else {
                    end = true;
                    first_e.clone()
                };
                if cur_e.is_same(&next_e) {
                    continue;
                }
                let vref = common_vertex(&cur_e, &next_e);
                let Some(ne3) = build.seek(vref.as_ref()).cloned() else {
                    cur_e = next_e;
                    continue;
                };
                cur_e = analyse.edge_replacement(fi, &cur_e).clone();
                next_e = analyse.edge_replacement(fi, &next_e).clone();
                let ceo = topods::edge(&ofi.generated(cur_e.as_ref()))?;
                let neo = topods::edge(&ofi.generated(next_e.as_ref()))?;
                let (cur_b, next_b) = (
                    build.seek(cur_e.as_ref()).cloned(),
                    build.seek(next_e.as_ref()).cloned(),
                );
                let (ne1, ne2) = if let (Some(b1), Some(b2)) = (&cur_b, &next_b) {
                    (b1.clone(), b2.clone())
                } else if let (Some(b1), Some(m)) = (&cur_b, mes.seek(neo.as_ref())) {
                    (b1.clone(), m.clone())
                } else if let (Some(b2), Some(m)) = (&next_b, mes.seek(ceo.as_ref())) {
                    (b2.clone(), m.clone())
                } else {
                    cur_e = wexp.current().clone();
                    continue;
                };
                let ors = (Orientation::External, Orientation::External);
                // The intersection of the edges generated from the vertex with the edges
                // of the face.
                let ne3_edges: Vec<Shape> =
                    Explorer::new(&ne3, ShapeEnum::Edge, ShapeEnum::Shape)?.collect();
                for (k, e3s) in ne3_edges.iter().enumerate() {
                    if !a_me.contains(e3s) {
                        continue;
                    }
                    let a_e3 = topods::edge(e3s)?;
                    // The intersections with the first, then the second edge.
                    for ne in [&ne1, &ne2] {
                        for e1s in Explorer::new(ne, ShapeEnum::Edge, ShapeEnum::Shape)? {
                            let a_e1 = topods::edge(&e1s)?;
                            let mut an_empty_image = BRepAlgoImage::new();
                            let b_coincide = ref_edge_inter(
                                &fio,
                                &b_asurf,
                                &a_e1,
                                &a_e3,
                                ors,
                                as_des2d,
                                tol,
                                true,
                                &vref,
                                &mut an_empty_image,
                                the_dmvv,
                            )?;
                            if b_coincide {
                                // A coincidence: E3 trimmed as E1.
                                let lv: Vec<Shape> = as_des2d.descendant(a_e1.as_ref()).collect();
                                store(&a_e3, &lv, tol, true, as_des2d, the_dmvv)?;
                            }
                        }
                    }
                    // The intersections of the edges generated from the vertex together:
                    // E3 with the ones after its first occurrence.
                    let first = ne3_edges.iter().position(|x| a_e3.is_same(x)).unwrap_or(k);
                    for e3_next in &ne3_edges[first + 1..] {
                        if a_me.contains(e3_next) {
                            let mut an_empty_image = BRepAlgoImage::new();
                            ref_edge_inter(
                                &fio,
                                &b_asurf,
                                &topods::edge(e3_next)?,
                                &a_e3,
                                ors,
                                as_des2d,
                                tol,
                                true,
                                &vref,
                                &mut an_empty_image,
                                the_dmvv,
                            )?;
                        }
                    }
                }
                cur_e = wexp.current().clone();
            }
        }
        Ok(())
    }
}
