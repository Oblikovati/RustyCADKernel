// Rust translation of BRepFeat_Builder::RebuildFaces and RebuildEdge of
// ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_Builder.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The faces of the basis rebuilt on the kept parts of the tool: the splits of their
//! boundary edges whose vertices touch a kept part, an edge cut by removed splits rebuilt
//! on the remaining vertices, the IN and section edges not removed, and the faces built
//! from them by BOPAlgo_BuilderFace.
use super::BRepFeatBuilder;
use crate::errors::null_object;
use rustycad_tkbo::bop_algo::BuilderFace;
use rustycad_tkbo::bop_ds::{DataStructure, Pave, PaveBlock, PaveBlockHandle, ShapeInfo};
use rustycad_tkbo::bop_tools::algo_tools2d::build_pcurve_for_edge_on_face;
use rustycad_tkbo::bop_tools::algo_tools3d::split_seam_on_face;
use rustycad_tkbo::bop_tools::{ShapeSet, is_split_edge_to_reverse, make_split_edge};
use rustycad_tkbo::int_tools::Context;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::ncollection::{List, Map};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;
use std::sync::{Arc, Mutex};

/// Locks a pave block.
fn lock(pb: &PaveBlockHandle) -> std::sync::MutexGuard<'_, PaveBlock> {
    pb.lock().unwrap_or_else(|e| e.into_inner())
}

/// The context of the last run of the builder.
fn context(handle: &Handle<Context>) -> Result<&Context, Failure> {
    handle.get().ok_or_else(|| {
        null_object(
            "BRepFeat_Builder::RebuildFaces",
            "no context; expected a performed builder",
        )
    })
}

/// What one boundary edge brings to its rebuilt face.
struct BoundaryEdge<'a> {
    face: &'a Face,
    removed_splits: MapOfShape,
    kept_splits: List<Shape>,
}

impl BRepFeatBuilder {
    /// The splits of the boundary edge `e` touching the kept shapes (by itself or a
    /// vertex) and the others, or None when `e` has no split.
    fn classify_splits(&self, e: &Shape) -> Result<Option<(List<Shape>, MapOfShape)>, Failure> {
        let Some(images) = self.bop.images().seek(e) else {
            return Ok(None);
        };
        let mut kept = List::default();
        let mut removed = MapOfShape::default();
        for s in images.iter() {
            let in_shapes = self.shapes.contains(s)
                || Explorer::new(s, ShapeEnum::Vertex, ShapeEnum::Shape)?
                    .any(|v| self.shapes.contains(&v));
            if in_shapes {
                kept.append(s.clone());
            } else {
                removed.add(s.clone());
            }
        }
        Ok(Some((kept, removed)))
    }

    /// The splits of a boundary edge oriented on the face: both orientations for an
    /// INTERNAL edge or a closed one (the seam split made first), else the orientation
    /// of the edge, reversed when the split runs against it.
    fn oriented_splits(
        &self,
        ee: &Edge,
        ff: &Face,
        splits: &List<Shape>,
        le: &mut List<Shape>,
    ) -> Result<(), Failure> {
        let an_ori_e = ee.orientation();
        let is_degenerated = BRepTool::degenerated(ee)?;
        let is_closed = BRepTool::is_closed_on_face(ee, ff)?;
        for s in splits.iter() {
            let mut sp = topods::edge(s)?;
            if is_degenerated {
                le.append(sp.oriented(an_ori_e));
                continue;
            }
            if an_ori_e == Orientation::Internal || is_closed {
                let internal = an_ori_e == Orientation::Internal;
                if !internal && !BRepTool::is_closed_on_face(&sp, ff)? {
                    split_seam_on_face(&sp, ff)?;
                }
                le.append(sp.oriented(Orientation::Forward));
                le.append(sp.oriented(Orientation::Reversed));
                continue;
            }
            sp.set_orientation(an_ori_e);
            let mut error = 0;
            if is_split_edge_to_reverse(&sp, ee, context(self.bop.context())?, &mut error)? {
                sp.reverse();
            }
            le.append(sp.into_shape());
        }
        Ok(())
    }

    /// The edges of the boundary of the face `ff` (FORWARD) for its rebuild.
    fn boundary_edges(
        &mut self,
        structure: &mut DataStructure,
        ff: &Face,
        me_split: &mut MapOfShape,
    ) -> Result<List<Shape>, Failure> {
        let mut le = List::default();
        for x in Explorer::new(ff, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let ee = topods::edge(&x)?;
            let Some((mut kept, removed)) = self.classify_splits(&x)? else {
                le.append(x);
                continue;
            };
            if kept.is_empty() {
                le.append(x);
                continue;
            }
            let mut splits = self.bop.images().find(&x)?.clone();
            if !removed.is_empty() {
                if splits.len() == 2 {
                    le.append(x);
                    continue;
                }
                if me_split.add(x.clone()) {
                    let mut edge = BoundaryEdge {
                        face: ff,
                        removed_splits: removed,
                        kept_splits: std::mem::take(&mut kept),
                    };
                    self.rebuild_edge_of(structure, &x, &mut edge)?;
                    splits = edge.kept_splits.clone();
                    if let Some(images) = self.bop.images_mut().change_seek(&x) {
                        *images = edge.kept_splits;
                    }
                    if splits.len() == 1 {
                        le.append(x);
                        continue;
                    }
                }
            }
            self.oriented_splits(&ee, ff, &splits, &mut le)?;
        }
        Ok(le)
    }

    /// The IN and section edges of the face `i` not removed, in both orientations.
    fn inner_edges(
        &self,
        structure: &DataStructure,
        i: i32,
        le: &mut List<Shape>,
    ) -> Result<(), Failure> {
        let fi = structure.face_info(i)?;
        for map in [fi.pave_blocks_in(), fi.pave_blocks_section()] {
            for pb in map.iter() {
                let n_sp = lock(pb).edge();
                let sp = structure.shape(n_sp)?;
                if self.removed.contains(sp) {
                    continue;
                }
                le.append(sp.oriented(Orientation::Forward));
                le.append(sp.oriented(Orientation::Reversed));
            }
        }
        Ok(())
    }

    /// The splits of the face `af` built from `le`, those equal to a kept face replaced by
    /// it (same domain), with the orientation of the face; the face unbound without split.
    fn bind_face_splits(
        &mut self,
        af: &Shape,
        ff: &Face,
        le: &List<Shape>,
        mst: &mut Map<ShapeSet>,
    ) -> Result<(), Failure> {
        let mut bf = BuilderFace::new();
        bf.set_face(ff);
        bf.set_shapes(le);
        bf.perform()?;
        let mut lf_im = List::default();
        for fr in bf.areas().iter() {
            let mut st = ShapeSet::new();
            st.add(fr, ShapeEnum::Edge)?;
            let flag_sd = mst.contains(&st);
            let sx = mst.added(st).shape().oriented(af.orientation());
            lf_im.append(sx.clone());
            let origins = self.bop.origins_mut();
            if !origins.is_bound(&sx) {
                origins.bind(sx.clone(), List::default());
            }
            if let Some(l) = origins.change_seek(&sx) {
                l.append(af.clone());
            }
            if flag_sd {
                self.bop.shapes_sd_mut().bind(fr.clone(), sx);
            }
        }
        if lf_im.is_empty() {
            self.bop.images_mut().unbind(af);
        } else if let Some(images) = self.bop.images_mut().change_seek(af) {
            *images = lf_im;
        }
        Ok(())
    }

    /// The face sets of the kept faces.
    fn kept_face_sets(&self) -> Result<Map<ShapeSet>, Failure> {
        let mut mst: Map<ShapeSet> = Map::default();
        for s in self.shapes.iter() {
            if s.shape_type()? == ShapeEnum::Face {
                let mut st = ShapeSet::new();
                st.add(s, ShapeEnum::Edge)?;
                mst.add(st);
            }
        }
        Ok(mst)
    }

    /// `RebuildFaces()`: the images of the tool restricted to the kept shapes, and the
    /// split faces of the basis rebuilt on the kept edges; `builder.rebuild_faces(&mut ds)?`.
    pub fn rebuild_faces(&mut self, structure: &mut DataStructure) -> Result<(), Failure> {
        let mut mst = self.kept_face_sets()?;
        let mut me_split = MapOfShape::default();
        for i in 0..structure.nb_source_shapes() {
            let info_shape = structure.shape_info(i)?.shape().clone();
            if structure.rank(i) == 1 {
                let shapes = &self.shapes;
                if let Some(images) = self.bop.images_mut().change_seek(&info_shape) {
                    images.retain(|s| shapes.contains(s));
                }
                continue;
            }
            if structure.shape_info(i)?.shape_type() != ShapeEnum::Face
                || !self.bop.images().is_bound(&info_shape)
            {
                continue;
            }
            let ff = topods::face(&info_shape.oriented(Orientation::Forward))?;
            let mut le = self.boundary_edges(structure, &ff, &mut me_split)?;
            self.inner_edges(structure, i, &mut le)?;
            // build new faces
            self.bind_face_splits(&info_shape, &ff, &le, &mut mst)?;
        }
        Ok(())
    }

    /// `RebuildEdge(theE, theF, theME, aLEIm)`: the edge `the_e` split again without the
    /// vertices of the removed splits `the_me` (its own vertices kept), the new pieces made
    /// on the face `the_f` and appended to the data structure; `a_lim` gets the pieces;
    /// `builder.rebuild_edge(&mut ds, &e, &f, &removed, &mut pieces)?`.
    pub fn rebuild_edge(
        &mut self,
        structure: &mut DataStructure,
        the_e: &Shape,
        the_f: &Face,
        the_me: &MapOfShape,
        a_lim: &mut List<Shape>,
    ) -> Result<(), Failure> {
        let mut edge = BoundaryEdge {
            face: the_f,
            removed_splits: the_me.clone(),
            kept_splits: std::mem::take(a_lim),
        };
        self.rebuild_edge_of(structure, the_e, &mut edge)?;
        *a_lim = edge.kept_splits;
        Ok(())
    }

    /// The original vertices of the edge `n_e` (their same-domain vertex when any).
    fn original_vertices(
        structure: &DataStructure,
        n_e: i32,
    ) -> Result<(Map<i32>, Map<i32>), Failure> {
        let mut mv: Map<i32> = Map::default();
        let mut mv_or: Map<i32> = Map::default();
        for &n_v in structure.shape_info(n_e)?.sub_shapes() {
            let mut n_vsd = 0;
            let n_vx = if structure.has_shape_sd_into(n_v, &mut n_vsd) {
                n_vsd
            } else {
                n_v
            };
            mv.add(n_vx);
            mv_or.add(n_vx);
        }
        Ok((mv, mv_or))
    }

    /// The new pave block of `n_e` holding as extra paves the ends of the old blocks at
    /// the vertices kept; the old blocks not removed are returned.
    fn kept_paves(
        structure: &DataStructure,
        lpb: &[PaveBlockHandle],
        me: &MapOfShape,
        mut mv: Map<i32>,
        mv_or: &Map<i32>,
    ) -> Result<(PaveBlock, Vec<PaveBlockHandle>), Failure> {
        // 2. collect vertices that should be removed to aMI map.
        let mut pb_new = PaveBlock::default();
        let mut mi: Map<i32> = Map::default();
        let mut mpb: Vec<PaveBlockHandle> = Vec::new();
        for pb in lpb {
            let n_e1 = lock(pb).edge();
            if me.contains(structure.shape(n_e1)?) {
                let (n_v1, n_v2) = lock(pb).indices();
                mi.add(n_v1);
                mi.add(n_v2);
            } else {
                mpb.push(pb.clone());
            }
        }
        // 3. collect vertices that split the source shape.
        for pb in lpb {
            let (n_v1, n_v2) = lock(pb).indices();
            if !mi.contains(&n_v1) {
                mv.add(n_v1);
            }
            if !mi.contains(&n_v2) {
                mv.add(n_v2);
            }
        }
        // 4. collect ext paves.
        let mut m_add: Map<i32> = Map::default();
        for pb in lpb {
            let block = lock(pb);
            let (n_v1, n_v2) = block.indices();
            for (n_v, pave) in [(n_v1, block.pave1()), (n_v2, block.pave2())] {
                if mv.contains(&n_v) && (m_add.add(n_v) || mv_or.contains(&n_v)) {
                    pb_new.change_ext_paves().push(pave);
                }
            }
        }
        Ok((pb_new, mpb))
    }

    /// The edge of an old pave block with the same vertices and range as `pb`.
    fn old_edge(
        structure: &DataStructure,
        mpb: &[PaveBlockHandle],
        (pave1, pave2): (Pave, Pave),
    ) -> Result<Option<Shape>, Failure> {
        let ((n_v1, t1), (n_v2, t2)) = (pave1.contents(), pave2.contents());
        for pb1 in mpb {
            let block = lock(pb1);
            let (n_v11, n_v21) = block.indices();
            let (t11, t21) = block.range();
            if n_v1 == n_v11 && n_v2 == n_v21 && t1 == t11 && t2 == t21 {
                return Ok(Some(structure.shape(block.edge())?.clone()));
            }
        }
        Ok(None)
    }

    /// The rebuild of one edge cut by removed splits.
    fn rebuild_edge_of(
        &mut self,
        structure: &mut DataStructure,
        the_e: &Shape,
        edge: &mut BoundaryEdge<'_>,
    ) -> Result<(), Failure> {
        // 1. collect origin vertices to aMV map.
        let n_e = structure.index(the_e);
        let (mv, mv_or) = Self::original_vertices(structure, n_e)?;
        let lpb = structure.change_pave_blocks(n_e)?.clone();
        let (mut pb_new, mpb) =
            Self::kept_paves(structure, &lpb, &edge.removed_splits, mv, &mv_or)?;
        let mut ae = topods::edge(the_e)?;
        ae.set_orientation(Orientation::Forward);
        edge.kept_splits.clear();
        // 5. split edge by new set of vertices.
        pb_new.set_original_edge(n_e);
        let mut blocks = Vec::new();
        pb_new.update(&mut blocks, false);
        let handles: Vec<PaveBlockHandle> = blocks
            .into_iter()
            .map(|b| Arc::new(Mutex::new(b)))
            .collect();
        *structure.change_pave_blocks(n_e)? = handles.clone();
        // The native shape info is shared by every new piece: its box accumulates.
        let mut si = ShapeInfo::default();
        si.set_shape_type(ShapeEnum::Edge);
        for pb in &handles {
            let paves = {
                let b = lock(pb);
                (b.pave1(), b.pave2())
            };
            // check if it is the old pave block.
            if let Some(old) = Self::old_edge(structure, &mpb, paves)? {
                edge.kept_splits.append(old);
                continue;
            }
            let ((n_v1, t1), (n_v2, t2)) = (paves.0.contents(), paves.1.contents());
            let v1 = topods::vertex(&structure.shape(n_v1)?.oriented(Orientation::Forward))?;
            let v2 = topods::vertex(&structure.shape(n_v2)?.oriented(Orientation::Reversed))?;
            let mut sp = Edge::default();
            make_split_edge(&ae, &v1, t1, &v2, t2, &mut sp)?;
            build_pcurve_for_edge_on_face(&sp, edge.face, Some(context(self.bop.context())?))?;
            si.set_shape(&sp);
            BRepBndLib::add(&sp, si.change_box(), true)?;
            let n_sp = structure.append_shape_info(&si);
            lock(pb).set_edge(n_sp);
            edge.kept_splits.append(sp.into_shape());
        }
        Ok(())
    }
}
