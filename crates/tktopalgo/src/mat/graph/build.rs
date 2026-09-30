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
// Rust translation of ModelingAlgorithms/TKTopAlgo/MAT/MAT_Graph.cxx (Perform, MakeArc and UpDateNodes) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The construction of the graph from the roots of the bisector trees.
use super::{GraphState, MatGraph};
use crate::mat::node::arc_ref;
use crate::mat::{MatArc, MatBasicElt, MatBisector, MatEdge, MatListOfBisector, MatNode, MatSide};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::errors;

// The number of the edge a bisector separates; a null edge is a native crash.
fn edge_number(edge: &Handle<MatEdge>) -> Result<i32, Failure> {
    edge.get().map(MatEdge::edge_number).ok_or_else(|| {
        errors::null_object("MAT_Graph: a bisector has a null edge; expected its two edges")
    })
}

impl GraphState {
    // `TheBasicElts(key)`.
    fn basic_elt(&self, key: i32) -> Result<Handle<MatBasicElt>, Failure> {
        self.basic_elts.find(&key).cloned()
    }
}

impl MatGraph {
    /// `Perform(SemiInfinite, TheRoots, NbBasicElts, NbArcs)`: the basic elements
    /// `1..=NbBasicElts`, then the arcs of the trees of the roots (a root node per
    /// root at infinity when `SemiInfinite`, one root node shared by the roots
    /// otherwise), then the first node of every arc. The maps keep what they held,
    /// as natively; `graph.perform(false, &roots, 4, 5)?`.
    pub fn perform(
        &self,
        semi_infinite: bool,
        the_roots: &Handle<MatListOfBisector>,
        nb_basic_elts: i32,
        nb_arcs: i32,
    ) -> Result<(), Failure> {
        let mut state = self.state.write();
        let nb_roots: i32 = if semi_infinite { the_roots.number() } else { 1 };
        state.number_of_infinite_nodes = if semi_infinite { nb_roots } else { 0 };
        state.number_of_arcs = nb_arcs;
        state.number_of_basic_elts = nb_basic_elts;
        state.number_of_nodes = nb_roots + nb_arcs;
        let mut ind_tab_nodes: i32 = state.number_of_nodes;
        let mut ind_tab_arcs: i32 = 1;
        for i in 1..=nb_basic_elts {
            state.basic_elts.bind(i, Handle::new(MatBasicElt::new(i)));
            state.basic_elt(i)?.set_geom_index(i);
        }
        let mut builder = ArcBuilder {
            state: &mut state,
            ind_tab_arcs: &mut ind_tab_arcs,
        };
        if semi_infinite {
            builder.infinite_roots(the_roots, &mut ind_tab_nodes)?;
        } else {
            builder.single_root(the_roots, &mut ind_tab_nodes)?;
        }
        up_date_nodes(&mut state, &mut ind_tab_nodes)
    }
}

// The recursion of MakeArc over the graph maps.
struct ArcBuilder<'s> {
    state: &'s mut GraphState,
    ind_tab_arcs: &'s mut i32,
}

impl ArcBuilder<'_> {
    // Several entries at infinity: a root node at infinity per root.
    fn infinite_roots(
        &mut self,
        roots: &Handle<MatListOfBisector>,
        ind_tab_nodes: &mut i32,
    ) -> Result<(), Failure> {
        roots.first();
        while roots.more() {
            let current_arc: Handle<MatArc> = self.make_arc(&roots.current()?)?;
            let extremite: Handle<MatNode> =
                Handle::new(MatNode::new(0, &current_arc, precision::infinite()));
            extremite.set_index(*ind_tab_nodes);
            current_arc.set_second_node(&extremite);
            self.state.nodes.bind(*ind_tab_nodes, extremite);
            roots.next()?;
            *ind_tab_nodes -= 1;
        }
        Ok(())
    }

    // One entry: the root node of the first root, shared by the arcs of every root,
    // which are neighbours around it.
    fn single_root(
        &mut self,
        roots: &Handle<MatListOfBisector>,
        ind_tab_nodes: &mut i32,
    ) -> Result<(), Failure> {
        roots.first();
        let first_root: Handle<MatBisector> = roots.current()?;
        let first_arc: Handle<MatArc> = self.make_arc(&first_root)?;
        let dist_ext: f64 = edge_ref(&first_root.first_edge())?.distance();
        let extremite: Handle<MatNode> =
            Handle::new(MatNode::new(first_root.end_point(), &first_arc, dist_ext));
        extremite.set_index(*ind_tab_nodes);
        first_arc.set_second_node(&extremite);
        self.state.nodes.bind(*ind_tab_nodes, extremite.clone());
        *ind_tab_nodes -= 1;
        let mut previous_arc: Handle<MatArc> = first_arc.clone();
        let mut current_arc: Handle<MatArc> = first_arc.clone();
        roots.next()?;
        while roots.more() {
            current_arc = self.make_arc(&roots.current()?)?;
            current_arc.set_second_node(&extremite);
            current_arc.set_neighbour(MatSide::Left, &extremite, &previous_arc)?;
            previous_arc.set_neighbour(MatSide::Right, &extremite, &current_arc)?;
            previous_arc = current_arc.clone();
            roots.next()?;
        }
        first_arc.set_neighbour(MatSide::Left, &extremite, &current_arc)?;
        current_arc.set_neighbour(MatSide::Right, &extremite, &first_arc)
    }

    // `MakeArc(aBisector, TheBasicElts, TheArcs, IndTabArcs)`: the arc of the bisector
    // and, recursively, the arcs of the bisectors of its list around its first node.
    fn make_arc(&mut self, a_bisector: &Handle<MatBisector>) -> Result<Handle<MatArc>, Failure> {
        let first_elt = self
            .state
            .basic_elt(edge_number(&a_bisector.first_edge())?)?;
        let second_elt = self
            .state
            .basic_elt(edge_number(&a_bisector.second_edge())?)?;
        let current_arc: Handle<MatArc> = Handle::new(MatArc::new(
            *self.ind_tab_arcs,
            a_bisector.bisector_number(),
            &first_elt,
            &second_elt,
        ));
        let mut dist_ext: f64 = a_bisector.dist_issue_point();
        if dist_ext == precision::infinite() {
            dist_ext = 1.0;
        }
        let extremite: Handle<MatNode> = Handle::new(MatNode::new(
            a_bisector.issue_point(),
            &current_arc,
            dist_ext,
        ));
        current_arc.set_first_node(&extremite);
        let bisector_list: Handle<MatListOfBisector> = a_bisector.list();
        bisector_list.first();
        if !bisector_list.more() {
            // An arc on the contour.
            second_elt.set_start_arc(&current_arc);
            first_elt.set_end_arc(&current_arc);
        } else {
            self.make_child_arcs(&bisector_list, &current_arc, &extremite)?;
        }
        current_arc.set_index(*self.ind_tab_arcs);
        self.state
            .arcs
            .bind(*self.ind_tab_arcs, current_arc.clone());
        *self.ind_tab_arcs += 1;
        Ok(current_arc)
    }

    // The arcs of the bisectors of `list`, neighbours around `extremite` with the arc
    // of their parent.
    fn make_child_arcs(
        &mut self,
        list: &Handle<MatListOfBisector>,
        current_arc: &Handle<MatArc>,
        extremite: &Handle<MatNode>,
    ) -> Result<(), Failure> {
        let mut prev_arc: Handle<MatArc> = current_arc.clone();
        let mut next_arc: Handle<MatArc> = Handle::default();
        while list.more() {
            next_arc = self.make_arc(&list.current()?)?;
            next_arc.set_second_node(extremite);
            next_arc.set_neighbour(MatSide::Left, extremite, &prev_arc)?;
            arc_ref(&prev_arc)?.set_neighbour(MatSide::Right, extremite, &next_arc)?;
            prev_arc = next_arc.clone();
            list.next()?;
        }
        current_arc.set_neighbour(MatSide::Left, extremite, &next_arc)?;
        arc_ref(&next_arc)?.set_neighbour(MatSide::Right, extremite, current_arc)
    }
}

fn edge_ref(edge: &Handle<MatEdge>) -> Result<&MatEdge, Failure> {
    edge.get().ok_or_else(|| {
        errors::null_object(
            "MAT_Graph::Perform: the first root has a null edge; expected its first edge",
        )
    })
}

// `UpDateNodes(IndTabNodes)`: the first node of every arc is indexed, stored, and
// linked to its arc.
fn up_date_nodes(state: &mut GraphState, ind_tab_nodes: &mut i32) -> Result<(), Failure> {
    for i in 1..=state.number_of_arcs {
        let arc: Handle<MatArc> = state.arcs.find(&i)?.clone();
        let bout: Handle<MatNode> = arc_ref(&arc)?.first_node();
        state.nodes.bind(*ind_tab_nodes, bout.clone());
        let node: &MatNode = bout.get().ok_or_else(|| {
            errors::null_object(
                "MAT_Graph::UpDateNodes: an arc has no first node; expected MakeArc",
            )
        })?;
        node.set_index(*ind_tab_nodes);
        *ind_tab_nodes -= 1;
        node.set_linked_arc(&arc);
    }
    Ok(())
}
