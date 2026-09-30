// Copyright (c) 1991-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of ModelingAlgorithms/TKTopAlgo/MAT/MAT_Graph.cxx (FusionOfBasicElts and FusionOfArcs) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The fusion of two basic elements that the contour repeats (a line doubled by the
//! circuit), with the fusion of the arcs that then separate the same elements.
use super::{GraphState, MatGraph};
use crate::mat::node::arc_ref;
use crate::mat::{MatArc, MatBasicElt, MatNode, MatSide, MatZone};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::errors;

/// The outputs of `FusionOfBasicElts` (native references): whether each pair of
/// arcs was merged and the geometry indices of the merged arcs. A field the member
/// does not write keeps its value, as the native references do.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MatFusion {
    /// `MergeArc1`: the end arc of the first element and the start arc of the second
    /// were merged.
    pub merge_arc1: bool,
    /// `GeomIndexArc1`: the geometry index of the kept arc of the first merge.
    pub geom_index_arc1: i32,
    /// `GeomIndexArc2`: the geometry index of the removed arc of the first merge.
    pub geom_index_arc2: i32,
    /// `MergeArc2`: the end and start arcs of the fused element were merged.
    pub merge_arc2: bool,
    /// `GeomIndexArc3`: the geometry index of the kept arc of the second merge.
    pub geom_index_arc3: i32,
    /// `GeomIndexArc4`: the geometry index of the removed arc of the second merge.
    pub geom_index_arc4: i32,
}

// The two elements an arc separates; a null arc is a native null dereference.
fn elements_of(
    arc: &Handle<MatArc>,
) -> Result<(Handle<MatBasicElt>, Handle<MatBasicElt>), Failure> {
    let arc: &MatArc = arc_ref(arc)?;
    Ok((arc.first_element(), arc.second_element()))
}

// True when two arcs separate the same elements.
fn same_elements(a: &Handle<MatArc>, b: &Handle<MatArc>) -> Result<bool, Failure> {
    let (e1, e2) = elements_of(a)?;
    let (e3, e4) = elements_of(b)?;
    Ok((e1 == e3 || e1 == e4) && (e2 == e3 || e2 == e4))
}

fn node_ref(node: &Handle<MatNode>) -> Result<&MatNode, Failure> {
    node.get().ok_or_else(|| {
        errors::null_object("MAT_Graph: a null node is dereferenced; expected the ends of the arcs")
    })
}

// True when an end of either arc is on the figure.
fn on_figure(a: &Handle<MatArc>, b: &Handle<MatArc>) -> Result<bool, Failure> {
    for arc in [a, b] {
        let arc: &MatArc = arc_ref(arc)?;
        for node in [arc.first_node(), arc.second_node()] {
            if node_ref(&node)?.on_basic_elt() {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

impl MatGraph {
    /// `FusionOfBasicElts(IndexElt1, IndexElt2, MergeArc1, GeomIndexArc1,
    /// GeomIndexArc2, MergeArc2, GeomIndexArc3, GeomIndexArc4)`: the element
    /// `index_elt2` is removed and its arcs separate `index_elt1` instead; the arcs
    /// that then separate the same elements are merged (`fusion` tells which);
    /// `graph.fusion_of_basic_elts(1, 5, &mut fusion)?`.
    pub fn fusion_of_basic_elts(
        &self,
        index_elt1: i32,
        index_elt2: i32,
        fusion: &mut MatFusion,
    ) -> Result<(), Failure> {
        let mut state = self.state.write();
        let elt1: Handle<MatBasicElt> = state.basic_elts.find(&index_elt1)?.clone();
        let elt2: Handle<MatBasicElt> = state.basic_elts.find(&index_elt2)?.clone();
        if elt1 == elt2 {
            return Ok(());
        }
        state.move_zone(&elt2, &elt1)?;
        let ea1: Handle<MatArc> = elt1.end_arc();
        let sa2: Handle<MatArc> = elt2.start_arc();
        fusion.merge_arc1 = false;
        if same_elements(&ea1, &sa2)? {
            state.fuse_by_index(&ea1, &sa2)?;
            fusion.merge_arc1 = true;
            fusion.geom_index_arc1 = ea1.geom_index();
            fusion.geom_index_arc2 = sa2.geom_index();
        }
        // The end of Elt1 becomes the end of Elt2.
        elt1.set_end_arc(&elt2.end_arc());
        state.fuse_ends(&elt1, fusion)?;
        state.basic_elts.unbind(&elt2.index());
        state.number_of_basic_elts -= 1;
        Ok(())
    }
}

impl GraphState {
    // The arcs of the zone of `from` separate `to` instead.
    fn move_zone(
        &mut self,
        from: &Handle<MatBasicElt>,
        to: &Handle<MatBasicElt>,
    ) -> Result<(), Failure> {
        let zone2 = MatZone::with_basic_elt(from)?;
        for i in 1..=zone2.number_of_arcs() {
            let arc: Handle<MatArc> = zone2.arc_on_frontier(i)?;
            let stored: Handle<MatArc> = self.arcs.find(&arc.index())?.clone();
            if arc.first_element() == *from {
                stored.set_first_element(to);
            } else {
                stored.set_second_element(to);
            }
        }
        Ok(())
    }

    // The end arc and the start arc of `elt1` may separate the same elements: they
    // are merged unless one of their nodes is on the figure (a closed contour of one
    // element; two elements keep them).
    fn fuse_ends(
        &mut self,
        elt1: &Handle<MatBasicElt>,
        fusion: &mut MatFusion,
    ) -> Result<(), Failure> {
        let sa1: Handle<MatArc> = elt1.start_arc();
        let ea1: Handle<MatArc> = elt1.end_arc();
        if ea1 == sa1 {
            return Ok(());
        }
        let same: bool = same_elements(&ea1, &sa1)?;
        let on_fig: bool = on_figure(&ea1, &sa1)?;
        fusion.merge_arc2 = false;
        if same && !on_fig {
            self.fuse_by_index(&ea1, &sa1)?;
            fusion.merge_arc2 = true;
            fusion.geom_index_arc3 = ea1.geom_index();
            fusion.geom_index_arc4 = sa1.geom_index();
        }
        Ok(())
    }

    // `FusionOfArcs(theArcs(A1->Index()), theArcs(A2->Index()))`.
    fn fuse_by_index(&mut self, a1: &Handle<MatArc>, a2: &Handle<MatArc>) -> Result<(), Failure> {
        let arc1: Handle<MatArc> = self.arcs.find(&a1.index())?.clone();
        let arc2: Handle<MatArc> = self.arcs.find(&a2.index())?.clone();
        self.fusion_of_arcs(&arc1, &arc2)
    }

    // `FusionOfArcs(Arc1, Arc2)`: two arcs separating the same elements become one:
    // `arc1` runs from the second node of `arc2` to its own second node, and `arc2`
    // is removed with the old first nodes.
    fn fusion_of_arcs(
        &mut self,
        arc1: &Handle<MatArc>,
        arc2: &Handle<MatArc>,
    ) -> Result<(), Failure> {
        let old_node1: Handle<MatNode> = arc1.first_node();
        let old_node2: Handle<MatNode> = arc2.first_node();
        let join: Handle<MatNode> = arc2.second_node();
        arc1.set_first_node(&join);
        if !node_ref(&join)?.infinite() {
            let l_neighbour: Handle<MatArc> = arc2.neighbour(&join, MatSide::Left)?;
            let r_neighbour: Handle<MatArc> = arc2.neighbour(&join, MatSide::Right)?;
            arc1.set_first_arc(MatSide::Left, &l_neighbour);
            arc1.set_first_arc(MatSide::Right, &r_neighbour);
            self.arcs
                .find(&arc_ref(&l_neighbour)?.index())?
                .set_neighbour(MatSide::Right, &join, arc1)?;
            self.arcs
                .find(&arc_ref(&r_neighbour)?.index())?
                .set_neighbour(MatSide::Left, &join, arc1)?;
        } else {
            arc1.set_first_arc(MatSide::Left, &Handle::default());
            arc1.set_first_arc(MatSide::Right, &Handle::default());
        }
        node_ref(&arc1.first_node())?.set_linked_arc(arc1);
        for old in [&old_node1, &old_node2] {
            let index: i32 = node_ref(old)?.index();
            if self.nodes.is_bound(&index) {
                self.nodes.unbind(&index);
                self.number_of_nodes -= 1;
            }
        }
        // The removed arc stays alive through `arc2` for the updates below.
        self.arcs.unbind(&arc2.index());
        self.number_of_arcs -= 1;
        for element in [arc2.first_element(), arc2.second_element()] {
            let index: i32 = element.get().map(MatBasicElt::index).ok_or_else(|| {
                errors::null_object(
                    "MAT_Graph::FusionOfArcs: the arc has a null element; expected two elements",
                )
            })?;
            let be: Handle<MatBasicElt> = self.basic_elts.find(&index)?.clone();
            if be.start_arc() == *arc2 {
                be.set_start_arc(arc1);
            }
            if be.end_arc() == *arc2 {
                be.set_end_arc(arc1);
            }
        }
        Ok(())
    }
}
