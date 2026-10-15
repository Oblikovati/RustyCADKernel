// Rust translation of BRepOffset_Inter3d::ConnexIntByInt of
// ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_Inter3d.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The intersections of the enlarged offset faces adjacent through convex or concave
//! edges (and, for planar shapes, through vertices only), with a distinct section edge
//! for each separate part sharing a section.
use super::{BRepOffsetInter3d, OffsetFaces, ShapeShapeMap, first_type};
use crate::brep_offset::{BRepOffsetAnalyse, BRepOffsetTool, EnlargeFaceOptions};
use rustycad_tkbo::bop_tools::{make_connexity_compounds, make_split_edge};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Compound, Edge, Face, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::{
    IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape, ShapeMapHasher,
};
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkfillet::chfi_ds::TypeOfConcavity;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};

type ShapeLists = DataMap<Shape, List<Shape>, ShapeMapHasher>;

/// A new compound.
fn compound() -> Shape {
    let mut c = Compound::default();
    BRepBuilder::default().make_compound(&mut c);
    c.into_shape()
}

/// The face of an offset in `MapSF`.
fn offset_face(map_sf: &OffsetFaces, f: &Shape) -> Result<Face, Failure> {
    let o = map_sf.seek(f).ok_or_else(|| {
        Failure::with_message("Standard_NoSuchObject", "NCollection_DataMap::Find")
    })?;
    Ok(o.face().clone())
}

/// A shape bound in a map; Standard_NoSuchObject otherwise.
fn found<'a>(map: &'a ShapeShapeMap, s: &Shape) -> Result<&'a Shape, Failure> {
    map.seek(s)
        .ok_or_else(|| Failure::with_message("Standard_NoSuchObject", "NCollection_DataMap::Find"))
}

/// The enlarged face of an offset face (`MES`), computed with the bounds of its
/// initial face when not yet bound.
fn enlarged(
    mes: &mut ShapeShapeMap,
    of: &Face,
    f: &Face,
    analyse: &BRepOffsetAnalyse,
) -> Result<Face, Failure> {
    if let Some(nf) = mes.seek(of.as_ref()) {
        return topods::face(nf);
    }
    let (enlarge_u, enlarge_vfirst, enlarge_vlast) = BRepOffsetTool::check_bounds(f, analyse)?;
    let mut nf = Face::default();
    let options = EnlargeFaceOptions {
        update_pcurve: true,
        enlarge_u,
        enlarge_vfirst,
        enlarge_vlast,
        ..EnlargeFaceOptions::with_change_geom(true)
    };
    BRepOffsetTool::en_large_face(of, &mut nf, options)?;
    mes.bind(of.as_ref().clone(), nf.as_ref().clone());
    Ok(nf)
}

impl BRepOffsetInter3d {
    /// `ConnexIntByInt(SI, MapSF, Analyse, MES, Build, Failed, Range, bIsPlanar)`: the
    /// sections of the enlarged offset faces across each convex or concave edge (and,
    /// for planar shapes, of the faces meeting at a vertex only), the section edges of
    /// each edge or vertex in `Build` (a compound), the shapes without section in
    /// `Failed`; a section shared by separate parts is split into distinct edges;
    /// `inter.connex_int_by_int(&si, &map_sf, &analyse, &mut mes, &mut build, &mut failed, &range, false)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn connex_int_by_int(
        &mut self,
        si: &Shape,
        map_sf: &OffsetFaces,
        analyse: &BRepOffsetAnalyse,
        mes: &mut ShapeShapeMap,
        build: &mut ShapeShapeMap,
        failed: &mut List<Shape>,
        the_range: &ProgressRange,
        b_is_planar: bool,
    ) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let mut ve_map = IndexedMapOfShape::default();
        let mut cur_side;
        topexp::map_shapes_of_type(si, ShapeEnum::Edge, &mut ve_map)?;
        // The vertices to treat.
        let a_ps_outer = ProgressScope::unnamed(the_range, 10., false);
        let mut a_nb = 0;
        if b_is_planar {
            a_nb = ve_map.extent();
            for i in 1..=a_nb {
                let a_f_gen = analyse.generated(ve_map.find_key(i)?);
                if !a_f_gen.is_null() {
                    topexp::map_shapes_of_type(&a_f_gen, ShapeEnum::Edge, &mut ve_map)?;
                }
            }
            topexp::map_shapes_of_type(si, ShapeEnum::Vertex, &mut ve_map)?;
            for nf in analyse.new_faces() {
                topexp::map_shapes_of_type(nf, ShapeEnum::Vertex, &mut ve_map)?;
            }
        }
        let (mut a_dmvlf1, mut a_dmvlf2, mut a_dm_int_ff) = (
            ShapeLists::default(),
            ShapeLists::default(),
            ShapeLists::default(),
        );
        let mut a_dm_int_e = IndexedDataMapOfShapeListOfShape::default();
        if b_is_planar {
            // The internal edges of the faces, skipped while pairing the faces through
            // vertices.
            let mut a_dmfei: DataMap<Shape, MapOfShape, ShapeMapHasher> = DataMap::default();
            for a_fx in Explorer::new(si, ShapeEnum::Face, ShapeEnum::Shape)? {
                let mut a_mei = MapOfShape::default();
                for a_ex in Explorer::new(&a_fx, ShapeEnum::Edge, ShapeEnum::Shape)? {
                    if a_ex.orientation() != Orientation::Forward
                        && a_ex.orientation() != Orientation::Reversed
                    {
                        a_mei.add(a_ex);
                    }
                }
                if !a_mei.is_empty() {
                    a_dmfei.bind(a_fx, a_mei);
                }
            }
            // The faces connected through the vertices.
            let a_nb_all = ve_map.extent();
            for i in a_nb + 1..=a_nb_all {
                if !a_ps_outer.more() {
                    return Ok(());
                }
                let a_s = ve_map.find_key(i)?.clone();
                if a_s.shape_type()? != ShapeEnum::Vertex {
                    continue;
                }
                // The faces connected to the vertex.
                let mut a_lf: Vec<Shape> = Vec::new();
                for e in analyse.ancestors(&a_s)? {
                    for f in analyse.ancestors(e)? {
                        if !a_lf.iter().any(|x| x == f) {
                            a_lf.push(f.clone());
                        }
                    }
                }
                if a_lf.len() < 2 {
                    continue;
                }
                // The pairs of faces whose common vertex is alone (not on shared edges).
                let (mut a_lf1, mut a_lf2) = (List::default(), List::default());
                for (k, a_fv1) in a_lf.iter().enumerate() {
                    let mut a_me = MapOfShape::default();
                    let p_f1_internal = a_dmfei.seek(a_fv1);
                    let Some(p_le1) = analyse.descendants(a_fv1, false) else {
                        continue;
                    };
                    let mut broken = false;
                    for a_e in p_le1 {
                        if p_f1_internal.is_some_and(|m| m.contains(a_e)) {
                            broken = true;
                            break;
                        }
                        let mut it = ShapeIterator::new(a_e, true, true);
                        while it.more() {
                            if a_s.is_same(it.value()?) {
                                a_me.add(a_e.clone());
                                break;
                            }
                            it.next();
                        }
                    }
                    if broken {
                        continue;
                    }
                    for a_fv2 in &a_lf[k + 1..] {
                        let p_f2_internal = a_dmfei.seek(a_fv2);
                        let Some(p_le2) = analyse.descendants(a_fv2, false) else {
                            continue;
                        };
                        let mut stopped = false;
                        for a_ev2 in p_le2 {
                            if !a_me.contains(a_ev2) {
                                continue;
                            }
                            if p_f2_internal.is_some_and(|m| m.contains(a_ev2)) {
                                // No intersection of faces connected by an internal edge.
                                stopped = true;
                                break;
                            }
                            if analyse.has_ancestor(a_ev2) && analyse.ancestors(a_ev2)?.len() == 2 {
                                // The faces are intersected through the edge.
                                stopped = true;
                                break;
                            }
                        }
                        if !stopped {
                            a_lf1.append(a_fv1.clone());
                            a_lf2.append(a_fv2.clone());
                        }
                    }
                }
                if !a_lf1.is_empty() {
                    a_dmvlf1.bind(a_s.clone(), a_lf1);
                    a_dmvlf2.bind(a_s, a_lf2);
                }
            }
        }
        let a_nb = ve_map.extent();
        let a_ps_inter = ProgressScope::with_literal(
            &a_ps_outer.next(8.),
            "Intersecting offset faces",
            a_nb as f64,
            false,
        );
        for i in 1..=a_nb {
            if !a_ps_inter.more() {
                return Ok(());
            }
            let a_s = ve_map.find_key(i)?.clone();
            let _ = a_ps_inter.next(1.);
            let mut e = Edge::default();
            let (a_lf1, a_lf2);
            if a_s.shape_type()? == ShapeEnum::Edge {
                // The faces connected by the edge.
                e = topods::edge(&a_s)?;
                let Some(ot) = first_type(analyse, &e)? else {
                    continue;
                };
                if ot != TypeOfConcavity::Convex && ot != TypeOfConcavity::Concave {
                    continue;
                }
                cur_side = if ot == TypeOfConcavity::Concave {
                    State::In
                } else {
                    State::Out
                };
                // The edge is of the proper type: the adjacent faces.
                let anc = analyse.ancestors(&a_s)?;
                if anc.len() != 2 {
                    continue;
                }
                a_lf1 = vec![anc[0].clone()];
                a_lf2 = vec![anc[1].clone()];
            } else {
                let (Some(l1), Some(l2)) = (a_dmvlf1.seek(&a_s), a_dmvlf2.seek(&a_s)) else {
                    continue;
                };
                a_lf1 = l1.iter().cloned().collect::<Vec<_>>();
                a_lf2 = l2.iter().cloned().collect::<Vec<_>>();
                cur_side = self.my_side;
            }
            for (f1s, f2s) in a_lf1.iter().zip(a_lf2.iter()) {
                let f1 = topods::face(f1s)?;
                let f2 = topods::face(f2s)?;
                let of1 = offset_face(map_sf, f1s)?;
                let of2 = offset_face(map_sf, f2s)?;
                let nf1 = enlarged(mes, &of1, &f1, analyse)?;
                let nf2 = enlarged(mes, &of2, &f2, analyse)?;
                if !self.is_done(&nf1, &nf2) {
                    let (mut l_int1, mut l_int2) = (List::default(), List::default());
                    BRepOffsetTool::inter3d(
                        &nf1,
                        &nf2,
                        &mut l_int1,
                        &mut l_int2,
                        cur_side,
                        &e,
                        &f1,
                        &f2,
                    )?;
                    self.set_done(&nf1, &nf2);
                    if !l_int1.is_empty() {
                        self.store(&nf1, &nf2, &l_int1, &l_int2);
                        let mut c = compound();
                        if let Some(a_se) = build.seek(&a_s) {
                            for a_ne in Explorer::new(a_se, ShapeEnum::Edge, ShapeEnum::Shape)? {
                                builder.add(&mut c, &a_ne)?;
                            }
                        }
                        for a_ne in l_int1.iter() {
                            builder.add(&mut c, a_ne)?;
                            // The connection of the new edge to the shape it comes from
                            // and to the faces that created it.
                            let index = a_dm_int_e.add(a_ne.clone(), List::default());
                            a_dm_int_e.change_from_index(index)?.append(a_s.clone());
                            let p_lff = a_dm_int_ff.bound(a_ne.clone(), List::default());
                            p_lff.append(f1s.clone());
                            p_lff.append(f2s.clone());
                        }
                        build.bind(a_s.clone(), c);
                    } else {
                        failed.append(a_s.clone());
                    }
                } else {
                    let a_l_int1: Vec<Shape> = self.as_des().descendant(nf1.as_ref()).collect();
                    let a_l_int2: Vec<Shape> = self.as_des().descendant(nf2.as_ref()).collect();
                    if !a_l_int1.is_empty() {
                        let mut c = compound();
                        if let Some(a_se) = build.seek(&a_s) {
                            for a_ne in Explorer::new(a_se, ShapeEnum::Edge, ShapeEnum::Shape)? {
                                builder.add(&mut c, &a_ne)?;
                            }
                        }
                        for an_e1 in &a_l_int1 {
                            for an_e2 in &a_l_int2 {
                                if an_e1.is_same(an_e2) {
                                    builder.add(&mut c, an_e1)?;
                                    let index = a_dm_int_e.find_index(an_e1);
                                    if index > 0 {
                                        a_dm_int_e.change_from_index(index)?.append(a_s.clone());
                                    }
                                }
                            }
                        }
                        build.bind(a_s.clone(), c);
                    } else {
                        failed.append(a_s.clone());
                    }
                }
            }
        }
        // A distinct intersection for each separate part sharing a section.
        let a_nb = a_dm_int_e.extent();
        let a_ps_post_treat = ProgressScope::with_literal(
            &a_ps_outer.next(2.),
            "Creating unique intersection",
            a_nb as f64,
            false,
        );
        for i in 1..=a_nb {
            if !a_ps_post_treat.more() {
                return Ok(());
            }
            let _ = a_ps_post_treat.next(1.);
            let a_ls: Vec<Shape> = a_dm_int_e.find_from_index(i)?.iter().cloned().collect();
            if a_ls.len() < 2 {
                continue;
            }
            self.unique_intersections(i, &a_ls, &a_dm_int_e, &a_dm_int_ff, map_sf, mes, build)?;
        }
        Ok(())
    }

    /// The post-treatment of one section edge shared by several shapes `aLS`: the
    /// connexity blocks of those shapes (joined by the edges of the creating faces that
    /// touch two of them); every block past the first gets its own copy of the edge.
    #[allow(clippy::too_many_arguments)]
    fn unique_intersections(
        &mut self,
        i: usize,
        a_ls: &[Shape],
        a_dm_int_e: &IndexedDataMapOfShapeListOfShape,
        a_dm_int_ff: &ShapeLists,
        map_sf: &OffsetFaces,
        mes: &ShapeShapeMap,
        build: &mut ShapeShapeMap,
    ) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        // The section edge and the faces that created it.
        let a_e = topods::edge(a_dm_int_e.find_key(i)?)?;
        let a_lff = a_dm_int_ff.seek(a_e.as_ref()).ok_or_else(|| {
            Failure::with_message("Standard_NoSuchObject", "NCollection_DataMap::Find")
        })?;
        let a_f1 = a_lff.first().cloned().unwrap_or_default();
        let a_f2 = a_lff.last().cloned().unwrap_or_default();
        // The vertices and the compound of edges of the parts.
        let mut a_lv = Vec::new();
        let mut a_ce = compound();
        let mut a_ms = MapOfShape::default();
        for a_s in a_ls {
            a_ms.add(a_s.clone());
            if a_s.shape_type()? == ShapeEnum::Edge {
                builder.add(&mut a_ce, a_s)?;
            } else {
                a_lv.push(a_s.clone());
            }
        }
        // The edges of the creating faces connecting the shared parts.
        let mut a_me_connection = MapOfShape::default();
        for a_f in [&a_f1, &a_f2] {
            for a_ef in Explorer::new(a_f, ShapeEnum::Edge, ShapeEnum::Shape)? {
                if a_ms.contains(&a_ef) || a_me_connection.contains(&a_ef) {
                    continue;
                }
                let (a_v1, a_v2) = topexp::vertices(&topods::edge(&a_ef)?, false);
                // The parts the edge is connected to.
                let mut i_counter = 0;
                for a_s in a_ls {
                    for a_v in Explorer::new(a_s, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                        if a_v.is_same(a_v1.as_ref()) || a_v.is_same(a_v2.as_ref()) {
                            i_counter += 1;
                            break;
                        }
                    }
                }
                if i_counter >= 2 {
                    builder.add(&mut a_ce, &a_ef)?;
                    a_me_connection.add(a_ef);
                }
            }
        }
        let mut a_lcbe = Vec::new();
        make_connexity_compounds(&a_ce, ShapeEnum::Vertex, ShapeEnum::Edge, &mut a_lcbe)?;
        // The blocks of the vertices alone.
        let mut a_lcbv = Vec::new();
        for a_v in &a_lv {
            let mut placed = false;
            for a_cb in a_lcbe.iter_mut() {
                let contains = Explorer::new(a_cb, ShapeEnum::Vertex, ShapeEnum::Shape)?
                    .any(|x| a_v.is_same(&x));
                if contains {
                    builder.add(a_cb, a_v)?;
                    placed = true;
                    break;
                }
            }
            if !placed {
                let mut a_cv = compound();
                builder.add(&mut a_cv, a_v)?;
                a_lcbv.push(a_cv);
            }
        }
        a_lcbe.extend(a_lcbv);
        if a_lcbe.len() == 1 {
            return Ok(());
        }
        let a_nf1 = found(mes, offset_face(map_sf, &a_f1)?.as_ref())?.clone();
        let a_nf2 = found(mes, offset_face(map_sf, &a_f2)?.as_ref())?.clone();
        for a_cb in &a_lcbe[1..] {
            // A new edge with another TEdge.
            let (a_v1, a_v2) = topexp::vertices(&a_e, false);
            let (a_t1, a_t2) = BRepTool::range(&a_e)?;
            let mut a_new_edge = Edge::default();
            make_split_edge(&a_e, &a_v1, a_t1, &a_v2, a_t2, &mut a_new_edge)?;
            self.as_des().add_shared(&a_nf1, a_new_edge.as_ref());
            self.as_des().add_shared(&a_nf2, a_new_edge.as_ref());
            let mut it = ShapeIterator::new(a_cb, true, true);
            while it.more() {
                let a_s = it.value()?.clone();
                it.next();
                if a_me_connection.contains(&a_s) {
                    continue;
                }
                let a_ci = found(build, &a_s)?.clone();
                let mut a_new_ci = compound();
                for a_sx in Explorer::new(&a_ci, ShapeEnum::Edge, ShapeEnum::Shape)? {
                    if !a_sx.is_same(a_e.as_ref()) {
                        builder.add(&mut a_new_ci, &a_sx)?;
                    }
                }
                builder.add(&mut a_new_ci, a_new_edge.as_ref())?;
                build.bind(a_s, a_new_ci);
            }
        }
        Ok(())
    }
}
