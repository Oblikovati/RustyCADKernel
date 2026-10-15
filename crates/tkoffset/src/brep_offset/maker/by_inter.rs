// Rust translation of BRepOffset_MakeOffset::MakeOffsetFaces, BuildOffsetByInter,
// ReplaceRoots and IntersectEdges of
// ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_MakeOffset.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The offset by intersection: the offset faces extended and intersected, their edges
//! trimmed, the faces cut into loops and the history filled.
use super::statics::{self, EdgesOrigins, not_bound};
use super::{BRepOffsetMakeOffset, normalize_steps};
use crate::brep_offset::inter3d::{OffsetFaces, ShapeShapeMap};
use crate::brep_offset::{
    BRepOffsetError, BRepOffsetInter2d, BRepOffsetInter3d, BRepOffsetOffset, CoincidentVertices,
    CreatedShapes,
};
use rustycad_tkbool::brep_algo::{BRepAlgoAsDes, BRepAlgoImage};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::{IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkfillet::chfi_ds::TypeOfConcavity;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkmath::geom_abs::JoinType;
use rustycad_tktopalgo::brep_lib;

/// The vertices and edges added to the AsDes as descendants of an edge (its vertices,
/// FORWARD then REVERSED) unless it has some.
fn add_edge_vertices(as_des: &BRepAlgoAsDes, coe: &Shape) -> Result<(), Failure> {
    if as_des.has_descendant(coe) {
        return Ok(());
    }
    let (cv1, cv2) = topexp::vertices(&topods::edge(coe)?, false);
    if !cv1.is_null() {
        as_des.add_shared(coe, &cv1.oriented(Orientation::Forward));
    }
    if !cv2.is_null() {
        as_des.add_shared(coe, &cv2.oriented(Orientation::Reversed));
    }
    Ok(())
}

impl BRepOffsetMakeOffset {
    /// `MakeOffsetFaces(theMapSF, Range)`: the offset of each face (its specific offset
    /// or the offset; the images of the tangent edges and their vertices shared), and of
    /// each new face of the analysis (null offset).
    pub(super) fn make_offset_faces(
        &mut self,
        the_map_sf: &mut OffsetFaces,
        the_range: &ProgressRange,
    ) -> Result<(), Failure> {
        let mut shape_tgt = CreatedShapes::default();
        let offset_outside = self.my_offset > 0.;
        let a_lf = brep_lib::sort_faces(&self.my_face_comp)?;
        let a_ps =
            ProgressScope::with_literal(the_range, "Making offset faces", a_lf.len() as f64, false);
        for fs in a_lf.iter() {
            if !a_ps.more() {
                self.my_error = BRepOffsetError::UserBreak;
                return Ok(());
            }
            let _ = a_ps.next(1.);
            let a_f = topods::face(fs)?;
            let a_cur_offset = self
                .my_face_offset
                .seek(fs)
                .copied()
                .unwrap_or(self.my_offset);
            let of = BRepOffsetOffset::from_face_created(
                &a_f,
                a_cur_offset,
                &shape_tgt,
                offset_outside,
                self.my_join,
            )?;
            let mut let_ = Vec::new();
            self.my_analyse
                .edges_on_face(&a_f, TypeOfConcavity::Tangential, &mut let_)?;
            for cur_s in &let_ {
                if shape_tgt.seek(cur_s).is_some() || self.my_analyse.has_generated(cur_s) {
                    continue;
                }
                let cur = topods::edge(cur_s)?;
                let ote = topods::edge(&of.generated(cur_s))?;
                shape_tgt.bind(cur_s.clone(), of.generated(cur_s));
                let (v1, v2) = topexp::vertices(&cur, false);
                let (ov1, ov2) = topexp::vertices(&ote, false);
                for (v, ov) in [(&v1, &ov1), (&v2, &ov2)] {
                    if shape_tgt.seek(v.as_ref()).is_some() {
                        continue;
                    }
                    let mut le = Vec::new();
                    self.my_analyse
                        .edges_on_vertex(v, TypeOfConcavity::Tangential, &mut le)?;
                    let la = self.my_analyse.ancestors(v.as_ref())?;
                    if le.len() == la.len() {
                        shape_tgt.bind(v.as_ref().clone(), ov.as_ref().clone());
                    }
                }
            }
            the_map_sf.bind(fs.clone(), of);
        }
        for fs in self.my_analyse.new_faces().to_vec() {
            let a_f = topods::face(&fs)?;
            let of = BRepOffsetOffset::from_face_created(
                &a_f,
                0.0,
                &shape_tgt,
                offset_outside,
                self.my_join,
            )?;
            the_map_sf.bind(fs, of);
        }
        Ok(())
    }

    /// `BuildOffsetByInter(Range)`: the offset faces extended and intersected (with each
    /// other and with the caps), their edges intersected and trimmed, the extended faces
    /// cut into loops, and the images of the faces, edges and caps filled.
    pub(super) fn build_offset_by_inter(
        &mut self,
        the_range: &ProgressRange,
    ) -> Result<(), Failure> {
        let a_ps_outer = ProgressScope::with_literal(
            the_range,
            "Connect offset faces by intersection",
            100.,
            false,
        );
        // The steps of the method.
        const MAKE_OFFSET_FACES: usize = 0;
        const CONNEX_INT_BY_INT: usize = 1;
        const CONTEXT_INT_BY_INT: usize = 2;
        const INTERSECT_EDGES: usize = 3;
        const COMPLETE_EDGES_INTERSECTION: usize = 4;
        const BUILD_FACES: usize = 5;
        const FILL_HISTORY_FOR_OFFSETS: usize = 6;
        const FILL_HISTORY_FOR_DEEPENINGS: usize = 7;
        let nb_children = ShapeIterator::new(&self.my_face_comp, true, true).count() as f64;
        let a_nb_faces =
            nb_children + self.my_analyse.new_faces().len() as f64 + self.my_faces.extent() as f64;
        let an_offsets_part = (nb_children + self.my_analyse.new_faces().len() as f64) / a_nb_faces;
        let a_deepenings_part = self.my_faces.extent() as f64 / a_nb_faces;
        let mut a_steps = [0.; 8];
        {
            let is_inter = self.my_join == JoinType::Intersection;
            let a_face_inter = if is_inter { 25. } else { 50. };
            let a_build_faces = if is_inter { 50. } else { 25. };
            a_steps[MAKE_OFFSET_FACES] = 5.;
            a_steps[CONNEX_INT_BY_INT] = a_face_inter * an_offsets_part;
            a_steps[CONTEXT_INT_BY_INT] = a_face_inter * a_deepenings_part;
            a_steps[INTERSECT_EDGES] = 10.;
            a_steps[COMPLETE_EDGES_INTERSECTION] = 5.;
            a_steps[BUILD_FACES] = a_build_faces;
            a_steps[FILL_HISTORY_FOR_OFFSETS] = 5. * an_offsets_part;
            a_steps[FILL_HISTORY_FOR_DEEPENINGS] = 5. * a_deepenings_part;
            normalize_steps(100., &mut a_steps);
        }
        // The faces parallel to the initial faces.
        let mut map_sf = OffsetFaces::default();
        self.make_offset_faces(&mut map_sf, &a_ps_outer.next(a_steps[MAKE_OFFSET_FACES]))?;
        if !a_ps_outer.more() {
            self.my_error = BRepOffsetError::UserBreak;
            return Ok(());
        }
        // MES: the offset shapes to their extended shapes; Build: the initial shapes to
        // the offset shapes built by intersection (an edge or a compound of edges).
        let mut mes = ShapeShapeMap::default();
        let mut build = ShapeShapeMap::default();
        let mut failed = List::default();
        let side = State::In;
        let as_des_h = Handle::new(BRepAlgoAsDes::new());
        let as_des = as_des_h.get().expect("AsDes");
        // The extension of the faces and the new intersection edges.
        let extent_context = self.my_offset > 0.;
        let mut inter3 = BRepOffsetInter3d::new(&as_des_h, side, self.my_tol);
        // The intersections of the parallel faces.
        inter3.connex_int_by_int(
            &self.my_face_comp,
            &map_sf,
            &self.my_analyse,
            &mut mes,
            &mut build,
            &mut failed,
            &a_ps_outer.next(a_steps[CONNEX_INT_BY_INT]),
            self.my_is_planar,
        )?;
        if !a_ps_outer.more() {
            self.my_error = BRepOffsetError::UserBreak;
            return Ok(());
        }
        // The intersections with the caps.
        inter3.context_int_by_int(
            &self.my_faces,
            extent_context,
            &map_sf,
            &self.my_analyse,
            &mut mes,
            &mut build,
            &mut failed,
            &a_ps_outer.next(a_steps[CONTEXT_INT_BY_INT]),
            self.my_is_planar,
        )?;
        if !a_ps_outer.more() {
            self.my_error = BRepOffsetError::UserBreak;
            return Ok(());
        }
        let mut a_l_faces: Vec<Shape> =
            Explorer::new(&self.my_face_comp, ShapeEnum::Face, ShapeEnum::Shape)?.collect();
        a_l_faces.extend(self.my_analyse.new_faces().iter().cloned());
        // The extension of the neighbour edges of the new edges and their intersections.
        let as_des2d_h = Handle::new(BRepAlgoAsDes::new());
        let as_des2d = as_des2d_h.get().expect("AsDes2d");
        self.intersect_edges(
            &a_l_faces,
            &mut map_sf,
            &mut mes,
            &build,
            as_des,
            as_des2d,
            &a_ps_outer.next(a_steps[INTERSECT_EDGES]),
        )?;
        if self.my_error != BRepOffsetError::NoError {
            return Ok(());
        }
        // The great restriction of the new edges and the update of the AsDes.
        let mut an_edges_origins = EdgesOrigins::default();
        let mut new_edges = IndexedMapOfShape::default();
        // The edges of the face-face intersections: the edges trimmed by the
        // intersection points with the other edges to the untrimmed ones.
        let mut a_e_trim_e_inf = ShapeShapeMap::default();
        if !statics::trim_edges(
            &self.my_face_comp,
            self.my_offset,
            &self.my_analyse,
            &map_sf,
            &mes,
            &build,
            as_des,
            as_des2d,
            &mut new_edges,
            &mut a_e_trim_e_inf,
            &mut an_edges_origins,
        )? {
            self.my_error = BRepOffsetError::CannotTrimEdges;
            return Ok(());
        }
        // The 2d intersections of the offset faces.
        let mut a_dmvv = CoincidentVertices::default();
        let mut a_faces_origins = ShapeShapeMap::default();
        let mut lfe = List::default();
        let mut imoe = BRepAlgoImage::new();
        statics::get_enlarged_faces(
            &a_l_faces,
            &map_sf,
            &mes,
            &mut a_faces_origins,
            &mut imoe,
            &mut lfe,
        )?;
        let a_ps2d = ProgressScope::unnamed(
            &a_ps_outer.next(a_steps[COMPLETE_EDGES_INTERSECTION]),
            2.,
            false,
        );
        {
            let a_ps2d_offsets =
                ProgressScope::unnamed(&a_ps2d.next(2. * an_offsets_part), lfe.len() as f64, false);
            for nef in lfe.iter() {
                if !a_ps2d_offsets.more() {
                    self.my_error = BRepOffsetError::UserBreak;
                    return Ok(());
                }
                let nef = topods::face(nef)?;
                let a_curr_face_tol = BRepTool::face_tolerance(&nef)?;
                BRepOffsetInter2d::compute(
                    as_des,
                    &nef,
                    &new_edges,
                    a_curr_face_tol,
                    &self.my_edge_int_edges,
                    &mut a_dmvv,
                    &a_ps2d_offsets.next(1.),
                )?;
            }
        }
        // The 2d intersections on the caps.
        {
            let a_ps2d_caps = ProgressScope::unnamed(
                &a_ps2d.next(2. * a_deepenings_part),
                self.my_faces.extent() as f64,
                false,
            );
            for i in 1..=self.my_faces.extent() {
                if !a_ps2d_caps.more() {
                    self.my_error = BRepOffsetError::UserBreak;
                    return Ok(());
                }
                let cork = topods::face(self.my_faces.find_key(i)?)?;
                let a_curr_face_tol = BRepTool::face_tolerance(&cork)?;
                BRepOffsetInter2d::compute(
                    as_des,
                    &cork,
                    &new_edges,
                    a_curr_face_tol,
                    &self.my_edge_int_edges,
                    &mut a_dmvv,
                    &a_ps2d_caps.next(1.),
                )?;
            }
        }
        BRepOffsetInter2d::fuse_vertices(&a_dmvv, as_des, &mut self.my_image_vv)?;
        // The unwinding of the extended faces.
        let mut a_mf_done = MapOfShape::default();
        if self.my_join == JoinType::Intersection && self.my_inter && self.my_is_planar {
            self.build_splits_of_extended_faces(
                &lfe,
                as_des,
                &mut an_edges_origins,
                &mut a_faces_origins,
                &mut a_e_trim_e_inf,
                &mut imoe,
                &a_ps_outer.next(a_steps[BUILD_FACES]),
            )?;
            if self.my_error != BRepOffsetError::NoError {
                return Ok(());
            }
            for s in lfe.iter() {
                a_mf_done.add(s.clone());
            }
        } else {
            let mut image_vv = std::mem::take(&mut self.my_image_vv);
            let r = self.my_make_loops.build(
                &lfe,
                as_des,
                &mut imoe,
                &mut image_vv,
                &a_ps_outer.next(a_steps[BUILD_FACES]),
            );
            self.my_image_vv = image_vv;
            r?;
            if !a_ps_outer.more() {
                self.my_error = BRepOffsetError::UserBreak;
                return Ok(());
            }
        }
        // The update of the data structure for the parallel faces.
        {
            let a_ps_hist = ProgressScope::with_literal(
                &a_ps_outer.next(a_steps[FILL_HISTORY_FOR_OFFSETS]),
                "Fill history for offset faces",
                a_l_faces.len() as f64,
                false,
            );
            for fi in &a_l_faces {
                if !a_ps_hist.more() {
                    self.my_error = BRepOffsetError::UserBreak;
                    return Ok(());
                }
                let _ = a_ps_hist.next(1.);
                self.fill_offset_face_history(fi, &map_sf, &mes, &imoe, as_des, &a_mf_done)?;
            }
        }
        // The history of the edges.
        let mut a_map_edges = MapOfShape::default();
        for a_face_ref in &a_l_faces {
            for an_edge_ref in Explorer::new(
                &a_face_ref.oriented(Orientation::Forward),
                ShapeEnum::Edge,
                ShapeEnum::Shape,
            )? {
                if !a_map_edges.add(an_edge_ref.clone()) {
                    continue;
                }
                self.my_init_offset_edge.set_root(&an_edge_ref);
                if let Some(a_new_shape) = build.seek(&an_edge_ref).cloned() {
                    if a_new_shape.shape_type()? == ShapeEnum::Edge {
                        if imoe.has_image(&a_new_shape) {
                            self.my_init_offset_edge
                                .bind_many(&an_edge_ref, &imoe.image(&a_new_shape))?;
                        } else {
                            self.my_init_offset_edge.bind(&an_edge_ref, &a_new_shape)?;
                        }
                    } else {
                        let mut a_list_new_edge = Vec::new();
                        for a_res_edge in
                            Explorer::new(&a_new_shape, ShapeEnum::Edge, ShapeEnum::Shape)?
                        {
                            if imoe.has_image(&a_res_edge) {
                                a_list_new_edge.extend(imoe.image(&a_res_edge));
                            } else {
                                a_list_new_edge.push(a_res_edge);
                            }
                        }
                        self.my_init_offset_edge
                            .bind_many(&an_edge_ref, &a_list_new_edge)?;
                    }
                } else {
                    // A free boundary.
                    let mut a_new_edge = map_sf
                        .seek(a_face_ref)
                        .ok_or_else(not_bound)?
                        .generated(&an_edge_ref);
                    if let Some(m) = mes.seek(&a_new_edge) {
                        a_new_edge = m.clone();
                    }
                    if imoe.has_image(&a_new_edge) {
                        self.my_init_offset_edge
                            .bind_many(&an_edge_ref, &imoe.image(&a_new_edge))?;
                    } else {
                        self.my_init_offset_edge.bind(&an_edge_ref, &a_new_edge)?;
                    }
                }
            }
        }
        // The update of the data structure for the caps.
        let a_ps_hist2 = ProgressScope::with_literal(
            &a_ps_outer.next(a_steps[FILL_HISTORY_FOR_DEEPENINGS]),
            "Fill history for deepening faces",
            self.my_faces.extent() as f64,
            false,
        );
        for i in 1..=self.my_faces.extent() {
            if !a_ps_hist2.more() {
                self.my_error = BRepOffsetError::UserBreak;
                return Ok(());
            }
            let _ = a_ps_hist2.next(1.);
            let cork = self.my_faces.find_key(i)?.clone();
            for oe in as_des.descendant(&cork) {
                if imoe.has_image(&oe) {
                    for coe in imoe.image(&oe) {
                        self.as_des()
                            .add_shared(&cork, &coe.oriented(oe.orientation()));
                        add_edge_vertices(self.as_des(), &coe)?;
                    }
                } else {
                    self.as_des().add_shared(&cork, &oe);
                    if as_des.has_descendant(&oe) {
                        for d in as_des.descendant(&oe) {
                            self.as_des().add_shared(&oe, &d);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// The history of one parallel face: its image faces (the splits of its extended
    /// face, or the extended or offset face itself) with their edges in the AsDes.
    fn fill_offset_face_history(
        &mut self,
        fi: &Shape,
        map_sf: &OffsetFaces,
        mes: &ShapeShapeMap,
        imoe: &BRepAlgoImage,
        as_des: &BRepAlgoAsDes,
        a_mf_done: &MapOfShape,
    ) -> Result<(), Failure> {
        self.my_init_offset_face.set_root(fi);
        let mut of = map_sf
            .seek(fi)
            .ok_or_else(not_bound)?
            .face()
            .as_ref()
            .clone();
        let Some(m) = mes.seek(&of).cloned() else {
            self.my_init_offset_face.bind(fi, &of)?;
            self.my_image_offset.set_root(&of);
            let mut view = MapOfShape::default();
            for coe in Explorer::new(
                &of.oriented(Orientation::Forward),
                ShapeEnum::Edge,
                ShapeEnum::Shape,
            )? {
                self.as_des().add_shared(&of, &coe);
                if view.add(coe.clone()) {
                    add_edge_vertices(self.as_des(), &coe)?;
                }
            }
            return Ok(());
        };
        of = m;
        if imoe.has_image(&of) {
            let lofe = imoe.image(&of);
            self.my_init_offset_face.bind_many(fi, &lofe)?;
            for ofe in &lofe {
                self.my_image_offset.set_root(ofe);
                let mut view = MapOfShape::default();
                for coe in Explorer::new(
                    &ofe.oriented(Orientation::Forward),
                    ShapeEnum::Edge,
                    ShapeEnum::Shape,
                )? {
                    self.as_des().add_shared(ofe, &coe);
                    if view.add(coe.clone()) {
                        add_edge_vertices(self.as_des(), &coe)?;
                    }
                }
            }
            return Ok(());
        }
        if a_mf_done.contains(&of) {
            return Ok(());
        }
        self.my_init_offset_face.bind(fi, &of)?;
        self.my_image_offset.set_root(&of);
        for oe in as_des.descendant(&of) {
            if imoe.has_image(&oe) {
                for loe in imoe.image(&oe) {
                    let coe = loe.oriented(oe.orientation());
                    self.as_des().add_shared(&of, &coe);
                    add_edge_vertices(self.as_des(), &coe)?;
                }
            } else {
                self.as_des().add_shared(&of, &oe);
                for v in as_des.descendant(&oe) {
                    self.as_des().add_shared(&oe, &v);
                }
            }
        }
        Ok(())
    }

    /// `IntersectEdges(theFaces, theMapSF, theMES, theBuild, theAsDes, theAsDes2d, Range)`:
    /// the 2d intersections of the images of the edges of each face (then of the faces
    /// with edges generated from vertices), and the fusion of the coinciding vertices.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn intersect_edges(
        &mut self,
        the_faces: &[Shape],
        the_map_sf: &mut OffsetFaces,
        the_mes: &mut ShapeShapeMap,
        the_build: &ShapeShapeMap,
        the_as_des: &BRepAlgoAsDes,
        the_as_des2d: &BRepAlgoAsDes,
        the_range: &ProgressRange,
    ) -> Result<(), Failure> {
        let mut a_dmvv = CoincidentVertices::default();
        // The intersections of the edges created from edges.
        let mut a_mfv = IndexedMapOfShape::default();
        let a_ps_outer = ProgressScope::unnamed(the_range, 2., false);
        {
            let a_ps1 = ProgressScope::unnamed(&a_ps_outer.next(1.), the_faces.len() as f64, false);
            for fs in the_faces {
                let a_f = topods::face(fs)?;
                let a_tol_f = BRepTool::face_tolerance(&a_f)?;
                let ofi = the_map_sf.change_seek(fs).ok_or_else(not_bound)?;
                if !BRepOffsetInter2d::connex_int_by_int(
                    &a_f,
                    ofi,
                    the_mes,
                    the_build,
                    the_as_des,
                    the_as_des2d,
                    self.my_offset,
                    a_tol_f,
                    &self.my_analyse,
                    &mut a_mfv,
                    &mut self.my_image_vv,
                    &mut self.my_edge_int_edges,
                    &mut a_dmvv,
                    &a_ps1.next(1.),
                )? {
                    self.my_error = BRepOffsetError::CannotExtentEdge;
                    return Ok(());
                }
                if !a_ps1.more() {
                    self.my_error = BRepOffsetError::UserBreak;
                    return Ok(());
                }
            }
        }
        // The intersections of the edges created from vertices.
        {
            let a_nb_f = a_mfv.extent();
            let a_ps2 = ProgressScope::with_literal(
                &a_ps_outer.next(1.),
                "Intersecting edges created from vertices",
                a_nb_f as f64,
                false,
            );
            for i in 1..=a_nb_f {
                let a_f = topods::face(a_mfv.find_key(i)?)?;
                let a_tol_f = BRepTool::face_tolerance(&a_f)?;
                let ofi = the_map_sf.change_seek(a_f.as_ref()).ok_or_else(not_bound)?;
                BRepOffsetInter2d::connex_int_by_int_in_vert(
                    &a_f,
                    ofi,
                    the_mes,
                    the_build,
                    the_as_des,
                    the_as_des2d,
                    a_tol_f,
                    &self.my_analyse,
                    &mut a_dmvv,
                    &a_ps2.next(1.),
                )?;
                if !a_ps2.more() {
                    self.my_error = BRepOffsetError::UserBreak;
                    return Ok(());
                }
            }
        }
        // The fusion of the vertices on the edges.
        if !BRepOffsetInter2d::fuse_vertices(&a_dmvv, the_as_des2d, &mut self.my_image_vv)? {
            self.my_error = BRepOffsetError::CannotFuseVertices;
        }
        Ok(())
    }

    /// `ReplaceRoots()`: the artificial faces and edges of the analysis replaced in the
    /// InitOffset maps by the original edges and vertices (the edges of the edges of
    /// the face compound, as natively).
    pub(super) fn replace_roots(&mut self) -> Result<(), Failure> {
        let mut view = MapOfShape::default();
        for a_f in Explorer::new(&self.my_face_comp, ShapeEnum::Edge, ShapeEnum::Shape)? {
            for a_e in Explorer::new(&a_f, ShapeEnum::Edge, ShapeEnum::Shape)? {
                if !view.add(a_e.clone()) {
                    continue;
                }
                let a_f_gen = self.my_analyse.generated(&a_e);
                if a_f_gen.is_null() {
                    continue;
                }
                self.my_init_offset_face.replace_root(&a_f_gen, &a_e)?;
                let mut it_v = ShapeIterator::new(&a_e, true, true);
                while it_v.more() {
                    let a_v = it_v.value()?.clone();
                    it_v.next();
                    if !view.add(a_v.clone()) {
                        continue;
                    }
                    let a_e_gen = self.my_analyse.generated(&a_v);
                    if a_e_gen.is_null() {
                        continue;
                    }
                    self.my_init_offset_edge.replace_root(&a_e_gen, &a_v)?;
                }
            }
        }
        Ok(())
    }
}
