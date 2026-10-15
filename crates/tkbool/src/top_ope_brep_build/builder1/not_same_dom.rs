//! The faces without same domain of `TopOpeBRepBuild_Builder1` (new builder): their
//! wires and edges kept by the states of the shapes, their ON parts, the faces made.
use crate::top_ope_brep_build::TopOpeBRepBuildShapeSet;
use crate::top_ope_brep_build::{
    TopOpeBRepBuildBuilder, TopOpeBRepBuildGTopo, TopOpeBRepBuildShellFaceSet,
    TopOpeBRepBuildWireEdgeSet,
};
use crate::top_ope_brep_tool::TopOpeBRepToolShapeExplorer;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::ncollection::{IndexedMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};

impl TopOpeBRepBuildBuilder {
    /// `GFillFaceNotSameDomSFS(FOR, LSO2, Gin, SFS)` (Builder1): the forward face filled
    /// (ON parts, kept wires and edges, curves), its faces made and kept as its splits
    /// (with the ON patches when the ON parts are processed), then the split faces
    /// added; `b.g_fill_face_not_same_dom_sfs(&f, &l, &g, &mut sfs)?`.
    pub fn g_fill_face_not_same_dom_sfs(
        &mut self,
        for_: &Shape,
        lso2: &List<Shape>,
        gin: &TopOpeBRepBuildGTopo,
        sfs: &mut TopOpeBRepBuildShellFaceSet,
    ) -> Result<(), Failure> {
        let mut g1 = *gin;
        g1.set_reverse(false);
        let (tb1, _) = g1.states_on()?;
        let ff = for_.oriented(Orientation::Forward);
        let mut wes = TopOpeBRepBuildWireEdgeSet::new(&ff)?;
        self.g_fill_on_parts_wes(for_, &g1, lso2, &mut wes)?;
        let mut edges_on = List::default();
        if self.my_process_on {
            let rev = self.opefus();
            for e in wes.start_elements().iter() {
                edges_on.append(if rev { e.reversed() } else { e.clone() });
            }
            self.my_on_elem_map.clear(true);
        }
        self.g_fill_face_not_same_dom_wes(&ff, lso2, &g1, &mut wes)?;
        self.g_fill_curve_topology_wes(&ff, &g1, &mut wes)?;
        self.my_edge_avoid.clear();
        self.mark_split(&ff, tb1, true);
        let mut lof = List::default();
        self.gwes_make_faces(&ff, &mut wes, &mut lof)?;
        if self.my_process_on && (!edges_on.is_empty() || self.my_on_elem_map.extent() > 0) {
            let mut map_oe = IndexedMap::default();
            for f in lof.iter() {
                for e in Explorer::new(f, ShapeEnum::Edge, ShapeEnum::Shape)? {
                    map_oe.add(e);
                }
            }
            self.fill_on_patches(&edges_on, for_, &map_oe)?;
            self.my_on_elem_map.clear(true);
        }
        let empty = self.my_empty_shape_list.clone();
        let mut lofs = List::default();
        self.change_split(&ff, tb1).clear();
        self.g_keep_shapes(&ff, &empty, tb1, &lof, &mut lofs)?;
        self.change_split(&ff, tb1).append_list(&mut lofs);
        self.g_split_face_sfs(for_, lso2, gin, sfs)
    }

    /// The state of a shape out of the structure kept (or ON with ON processing): the
    /// shape oriented, added as kept (or to the ON elements) and as a source shape;
    /// returns the oriented shape when kept.
    fn keep_by_state(
        &mut self,
        s: &Shape,
        tb1: State,
        rev_ori1: bool,
    ) -> Result<Option<Shape>, Failure> {
        let st = self.hds()?.ds().get_shape_with_state(s).state();
        let keep = st == tb1;
        if !keep && !(self.my_process_on && st == State::On) {
            return Ok(None);
        }
        let so = Self::oriented_by(s, rev_ori1);
        if !keep {
            self.my_on_elem_map.add(so.clone());
        }
        self.b1d_mut()?.my_source_shapes.add(so.clone());
        Ok(keep.then_some(so))
    }

    /// `GFillFaceNotSameDomWES(FOR1, LFclass, G1, WES)` (Builder1): the wires of the
    /// forward face out of the structure kept by their state, the others filled by
    /// their edges; `b.g_fill_face_not_same_dom_wes(&f, &l, &g, &mut wes)?`.
    pub fn g_fill_face_not_same_dom_wes(
        &mut self,
        for1: &Shape,
        lfclass: &List<Shape>,
        g1: &TopOpeBRepBuildGTopo,
        wes: &mut TopOpeBRepBuildWireEdgeSet,
    ) -> Result<(), Failure> {
        let (tb1, _) = g1.states_on()?;
        let rev_ori1 = g1.is_to_reverse1()?;
        self.b1d_mut()?.my_source_shapes.clear(true);
        let ff = for1.oriented(Orientation::Forward);
        let mut ex_wire = TopOpeBRepToolShapeExplorer::new(&ff, ShapeEnum::Wire, ShapeEnum::Shape)?;
        while ex_wire.more() {
            let w = ex_wire.current().clone();
            if !self.hds()?.has_shape(&w, true) {
                if let Some(w) = self.keep_by_state(&w, tb1, rev_ori1)? {
                    wes.add_shape(&w)?;
                }
            } else {
                self.g_fill_wire_not_same_dom_wes(&w, lfclass, g1, wes)?;
            }
            ex_wire.next()?;
        }
        Ok(())
    }

    /// `GFillWireNotSameDomWES(W, LSclass, G1, WES)` (Builder1): the edges of the wire
    /// out of the structure kept by their state, the others by their parts;
    /// `b.g_fill_wire_not_same_dom_wes(&w, &l, &g, &mut wes)?`.
    pub fn g_fill_wire_not_same_dom_wes(
        &mut self,
        w: &Shape,
        lsclass: &List<Shape>,
        g1: &TopOpeBRepBuildGTopo,
        wes: &mut TopOpeBRepBuildWireEdgeSet,
    ) -> Result<(), Failure> {
        let (tb1, _) = g1.states_on()?;
        let rev_ori1 = g1.is_to_reverse1()?;
        let mut ex_edge = TopOpeBRepToolShapeExplorer::new(w, ShapeEnum::Edge, ShapeEnum::Shape)?;
        while ex_edge.more() {
            let eor = ex_edge.current().clone();
            if !self.hds()?.has_shape(&eor, true) {
                if let Some(e) = self.keep_by_state(&eor, tb1, rev_ori1)? {
                    wes.add_element(&e)?;
                }
            } else {
                self.g_fill_edge_not_same_dom_wes(&eor, lsclass, g1, wes)?;
            }
            ex_edge.next()?;
        }
        Ok(())
    }

    /// `GFillEdgeNotSameDomWES(EOR, LSclass, G1, WES)` (Builder1): the parts of the edge
    /// on the state as start elements, then its ON parts kept (to keep, degenerated)
    /// or put to the ON elements; `b.g_fill_edge_not_same_dom_wes(&e, &l, &g, &mut wes)?`.
    pub fn g_fill_edge_not_same_dom_wes(
        &mut self,
        eor: &Shape,
        _lsclass: &List<Shape>,
        g1: &TopOpeBRepBuildGTopo,
        wes: &mut TopOpeBRepBuildWireEdgeSet,
    ) -> Result<(), Failure> {
        let (tb1, _) = g1.states_on()?;
        let neworie = Self::orient(eor.orientation(), g1.is_to_reverse1()?);
        let (lse, lse_on) = {
            let ds = self.hds()?.ds();
            let sws = ds.get_shape_with_state(eor);
            (sws.part(tb1), sws.part(State::On))
        };
        for e in lse.iter() {
            let new_e = e.oriented(neworie);
            wes.add_start_element(&new_e)?;
            self.b1d_mut()?.my_source_shapes.add(new_e);
        }
        for e in lse_on.iter() {
            let new_e = e.oriented(neworie);
            if self.b1d()?.my_splits_on_to_keep.contains(&new_e) {
                wes.add_start_element(&new_e)?;
                continue;
            }
            if BRepTool::degenerated(&topods::edge(&new_e)?)? {
                wes.add_start_element(&new_e)?;
                self.b1d_mut()?.my_source_shapes.add(new_e.clone());
            }
            if self.my_process_on {
                self.my_on_elem_map.add(new_e.clone());
                self.b1d_mut()?.my_source_shapes.add(new_e);
            }
        }
        Ok(())
    }
}
