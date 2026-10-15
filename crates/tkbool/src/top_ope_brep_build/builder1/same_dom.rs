//! The faces with same domain of `TopOpeBRepBuild_Builder1` (new builder): the faces
//! of the same domain of an object face filled on it in 2d, their edges oriented on it
//! by the side of their faces.
use super::SD_EDGE_MAP;
use crate::top_ope_brep_build::TopOpeBRepBuildShapeSet;
use crate::top_ope_brep_build::{
    TopOpeBRepBuildBuilder, TopOpeBRepBuildGTopo, TopOpeBRepBuildShellFaceSet,
    TopOpeBRepBuildTools, TopOpeBRepBuildWireEdgeSet,
};
use crate::top_ope_brep_tool::TopOpeBRepToolShapeExplorer;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};

impl TopOpeBRepBuildBuilder {
    /// The Builder1 maps of a same domain fill cleared.
    fn clear_same_dom_maps(&mut self) -> Result<(), Failure> {
        let d = self.b1d_mut()?;
        d.my_processed_parts_out2d.clear(true);
        d.my_processed_parts_on2d.clear(true);
        d.my_map_of_edge_with_face_state.clear(true);
        d.my_source_shapes.clear(true);
        SD_EDGE_MAP.with_borrow_mut(|m| m.clear(true));
        Ok(())
    }

    /// The face of `LOF` oriented as the original face, reversed by the state of the
    /// first of its edges with a face state.
    fn same_dom_face_orientation(&self, face: &Shape, orig_rev: bool) -> Result<Shape, Failure> {
        let mut em = IndexedMapOfShape::default();
        topexp::map_shapes_of_type(face, ShapeEnum::Edge, &mut em)?;
        let states = &self.b1d()?.my_map_of_edge_with_face_state;
        let rev = em
            .iter()
            .find_map(|e| states.find(e).ok().copied())
            .unwrap_or(false);
        let mut f = face.clone();
        if orig_rev {
            f.reverse();
        }
        if rev {
            f.reverse();
        }
        Ok(f)
    }

    /// `GFillFaceSameDomSFS(FOR, LSO2, Gin, SFS)` (Builder1): for an object face, the
    /// faces of its same domain filled on it, its faces made, oriented by the states of
    /// their edges, added as start elements and kept as its splits;
    /// `b.g_fill_face_same_dom_sfs(&f, &l, &g, &mut sfs)?`.
    pub fn g_fill_face_same_dom_sfs(
        &mut self,
        for_: &Shape,
        lso2: &List<Shape>,
        gin: &TopOpeBRepBuildGTopo,
        sfs: &mut TopOpeBRepBuildShellFaceSet,
    ) -> Result<(), Failure> {
        self.clear_same_dom_maps()?;
        if self.hds()?.ds().ancestor_rank(for_) != 1 {
            return Ok(());
        }
        let (tb1, _) = gin.states_on()?;
        let ff = for_.oriented(Orientation::Forward);
        let mut wes = TopOpeBRepBuildWireEdgeSet::new(&ff)?;
        self.g_fill_face_same_dom_wes(for_, lso2, gin, &mut wes)?;
        self.my_edge_avoid.clear();
        self.mark_split(&ff, tb1, true);
        let mut lof = List::default();
        self.gwes_make_faces(&ff, &mut wes, &mut lof)?;
        let orig_rev = for_.orientation() != Orientation::Forward;
        let mut ori_lof = List::default();
        for face in lof.iter() {
            let f = self.same_dom_face_orientation(face, orig_rev)?;
            sfs.add_start_element(&f)?;
            ori_lof.append(f);
        }
        let empty = self.my_empty_shape_list.clone();
        let mut lofs = List::default();
        self.change_split(&ff, tb1).clear();
        self.g_keep_shapes(&ff, &empty, tb1, &ori_lof, &mut lofs)?;
        self.change_split(&ff, tb1).append_list(&mut lofs);
        Ok(())
    }

    /// The same domain closure of a face: the face, then the same domain faces of each
    /// face of the map (added to the same domain map, their edges to the edge map).
    fn same_dom_closure(&mut self, for1: &Shape) -> Result<IndexedMapOfShape, Failure> {
        let mut cur = IndexedMapOfShape::default();
        cur.add(for1.clone());
        let mut i = 1;
        while i <= cur.extent() {
            let sd = self.hds()?.same_domain(cur.find_key(i)?);
            for sdf in sd.iter() {
                cur.add(sdf.clone());
                self.b1d_mut()?.my_same_dom_map.add(sdf.clone());
                SD_EDGE_MAP
                    .with_borrow_mut(|m| topexp::map_shapes_of_type(sdf, ShapeEnum::Edge, m))?;
            }
            i += 1;
        }
        Ok(cur)
    }

    /// `GFillFaceSameDomWES(FOR1, LFclass, G1, WES)` (Builder1): each face of the same
    /// domain closure filled on the base face: its curves, its ON parts, its kept wires
    /// (moved to the base face, edges oriented on it) and the parts of the others;
    /// `b.g_fill_face_same_dom_wes(&f, &l, &g, &mut wes)?`.
    pub fn g_fill_face_same_dom_wes(
        &mut self,
        for1: &Shape,
        _lfclass: &List<Shape>,
        g1: &TopOpeBRepBuildGTopo,
        wes: &mut TopOpeBRepBuildWireEdgeSet,
    ) -> Result<(), Failure> {
        g1.states_on()?;
        topods::face(for1)?;
        self.b1d_mut()?.my_base_face_to_fill = for1.clone();
        let cur = self.same_dom_closure(for1)?;
        for cur_f in cur.iter() {
            let cur_ff = cur_f.oriented(Orientation::Forward);
            topods::face(cur_f)?;
            self.b1d_mut()?.my_sd_face_to_fill = cur_f.clone();
            let iref = self.hds()?.ds().ancestor_rank(&cur_ff);
            let (tb, rev_ori) = Self::rank_states(g1, iref)?;
            let gftw = if iref != 1 && rev_ori {
                g1.copy_permuted()
            } else {
                *g1
            };
            self.g_fill_curve_topology_wes(&cur_ff, &gftw, wes)?;
            self.perform_on_parts(&cur_ff, &cur, g1, wes)?;
            let lsf = self.hds()?.ds().shape_same_domain(&cur_ff).clone();
            let mut ex_wire =
                TopOpeBRepToolShapeExplorer::new(&cur_ff, ShapeEnum::Wire, ShapeEnum::Shape)?;
            while ex_wire.more() {
                let w = ex_wire.current().clone();
                let hasshape = self.hds()?.has_shape(&w, true);
                let sh_st = self.hds()?.ds().get_shape_with_state(&w).state();
                if !hasshape && sh_st != State::On {
                    if sh_st == tb {
                        self.add_same_dom_wire(&w, g1, rev_ori, wes)?;
                    }
                } else {
                    self.g_fill_wire_same_dom_wes(&w, &lsf, g1, wes)?;
                }
                ex_wire.next()?;
            }
        }
        Ok(())
    }

    /// A kept wire of a same domain face: moved to the base face (else a source shape),
    /// its edges oriented on the base face (the reoriented ones as start elements), the
    /// whole oriented wire added when none was reoriented.
    fn add_same_dom_wire(
        &mut self,
        w: &Shape,
        g1: &TopOpeBRepBuildGTopo,
        rev_ori: bool,
        wes: &mut TopOpeBRepBuildWireEdgeSet,
    ) -> Result<(), Failure> {
        let (base, sd) = (
            self.b1d()?.my_base_face_to_fill.clone(),
            self.b1d()?.my_sd_face_to_fill.clone(),
        );
        if base != sd {
            TopOpeBRepBuildTools::update_pcurves(
                &topods::wire(w)?,
                &topods::face(&sd)?,
                &topods::face(&base)?,
            )?;
        } else {
            self.b1d_mut()?.my_source_shapes.add(w.clone());
        }
        let mut use_edges = false;
        for e in Explorer::new(w, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let mut eor = e.clone();
            let state_of_face_ori = self.orientate_edge_on_face(&mut eor, &base, &sd, g1)?;
            if eor.orientation() != e.orientation() {
                use_edges = true;
                wes.add_start_element(&eor)?;
            }
            self.b1d_mut()?
                .my_map_of_edge_with_face_state
                .bind(eor, state_of_face_ori);
        }
        if !use_edges {
            wes.add_shape(&Self::oriented_by(w, rev_ori))?;
        }
        Ok(())
    }

    /// `GFillWireSameDomWES(W, LSclass, G1, WES)` (Builder1): the edges of the wire out
    /// of the structure kept by their state (moved to the base face, oriented on it),
    /// the others by their parts; `b.g_fill_wire_same_dom_wes(&w, &l, &g, &mut wes)?`.
    pub fn g_fill_wire_same_dom_wes(
        &mut self,
        w: &Shape,
        lsclass: &List<Shape>,
        g1: &TopOpeBRepBuildGTopo,
        wes: &mut TopOpeBRepBuildWireEdgeSet,
    ) -> Result<(), Failure> {
        let iref = self.hds()?.ds().ancestor_rank(w);
        let (tb, _) = Self::rank_states(g1, iref)?;
        let mut ex_edge = TopOpeBRepToolShapeExplorer::new(w, ShapeEnum::Edge, ShapeEnum::Shape)?;
        while ex_edge.more() {
            let eor = ex_edge.current().clone();
            if !self.hds()?.has_shape(&eor, true) {
                if self.hds()?.ds().get_shape_with_state(&eor).state() == tb {
                    let mut e = eor.clone();
                    self.move_to_base_face(&e)?;
                    let st = self.orientate_edge_on_face_by_sd(&mut e, g1)?;
                    self.b1d_mut()?
                        .my_map_of_edge_with_face_state
                        .bind(e.clone(), st);
                    wes.add_element(&e)?;
                }
            } else {
                self.g_fill_edge_same_dom_wes(&eor, lsclass, g1, wes)?;
            }
            ex_edge.next()?;
        }
        Ok(())
    }

    /// The edge moved to the base face from the same domain face (a source shape when
    /// they are equal).
    pub(super) fn move_to_base_face(&mut self, e: &Shape) -> Result<(), Failure> {
        let (base, sd) = (
            self.b1d()?.my_base_face_to_fill.clone(),
            self.b1d()?.my_sd_face_to_fill.clone(),
        );
        if sd != base {
            TopOpeBRepBuildTools::update_edge_on_face(
                &topods::edge(e)?,
                &topods::face(&sd)?,
                &topods::face(&base)?,
            )
        } else {
            self.b1d_mut()?.my_source_shapes.add(e.clone());
            Ok(())
        }
    }

    /// `OrientateEdgeOnFace(E, base, SD, G1, state)` with the base and same domain faces
    /// to fill.
    pub(super) fn orientate_edge_on_face_by_sd(
        &self,
        e: &mut Shape,
        g1: &TopOpeBRepBuildGTopo,
    ) -> Result<bool, Failure> {
        let (base, sd) = (
            self.b1d()?.my_base_face_to_fill.clone(),
            self.b1d()?.my_sd_face_to_fill.clone(),
        );
        self.orientate_edge_on_face(e, &base, &sd, g1)
    }
}
