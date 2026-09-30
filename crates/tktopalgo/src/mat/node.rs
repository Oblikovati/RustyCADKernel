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
// Rust translation of ModelingAlgorithms/TKTopAlgo/MAT/MAT_Node.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MAT_Node`: a node of the bisecting locus graph, at a distance from the figure.
use super::{MatArc, MatBasicElt, MatSide};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, WeakHandle};
use rustycad_tkg2d::geom2d::errors;

struct NodeState {
    node_index: i32,
    geom_index: i32,
    a_linked_arc: WeakHandle<MatArc>,
    distance: f64,
}

/// `MAT_Node`: a transient node of a [`super::MatGraph`], with one of its arcs (a raw
/// pointer natively, a [`WeakHandle`] here, docs/topalgo-design.md section 3); the
/// other arcs are found by turning around the node on the left.
/// ```
/// use rustycad_tktopalgo::mat::MatNode;
/// use rustycad_tkernel::standard::Handle;
/// let node = MatNode::new(4, &Handle::default(), 0.0);
/// assert!(node.on_basic_elt() && !node.infinite());
/// ```
pub struct MatNode {
    parent: StandardTransient,
    state: SharedState<NodeState>,
}
impl_transient!(MatNode, "MAT_Node", StandardTransient, parent);

impl MatNode {
    /// `MAT_Node(GeomIndex, LinkedArc, Distance)`: index 0; `MatNode::new(1, &arc, 2.0)`.
    pub fn new(geom_index: i32, linked_arc: &Handle<MatArc>, distance: f64) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(NodeState {
                node_index: 0,
                geom_index,
                a_linked_arc: linked_arc.downgrade(),
                distance,
            }),
        }
    }
    /// `GeomIndex()`: the index of the point of the node; `node.geom_index()`.
    pub fn geom_index(&self) -> i32 {
        self.state.read().geom_index
    }
    /// `Index()`: the index in the nodes of the graph; `node.index()`.
    pub fn index(&self) -> i32 {
        self.state.read().node_index
    }
    // `occ::handle<MAT_Node> Me = this`: the node is compared by address.
    fn me(&self) -> *const () {
        std::ptr::from_ref(self).cast::<()>()
    }
    // `occ::handle<MAT_Arc> LA((MAT_Arc*)aLinkedArc)`; a null or destroyed arc is a
    // native null (or dangling) dereference, Standard_NullObject here.
    fn linked(&self, member: &str) -> Result<Handle<MatArc>, Failure> {
        let arc: Handle<MatArc> = self.state.read().a_linked_arc.upgrade();
        if arc.is_null() {
            return Err(errors::null_object(&format!(
                "MAT_Node::{member}: no linked arc; expected SetLinkedArc"
            )));
        }
        Ok(arc)
    }
    /// `LinkedArcs(S)`: the arcs of the node, from the linked arc turning on the left;
    /// `node.linked_arcs(&mut arcs)?`.
    pub fn linked_arcs(&self, s: &mut Sequence<Handle<MatArc>>) -> Result<(), Failure> {
        s.clear(None);
        let la: Handle<MatArc> = self.linked("LinkedArcs")?;
        s.append(la.clone());
        if !la.has_neighbour_at(self.me(), MatSide::Left)? {
            return Ok(());
        }
        let mut ca: Handle<MatArc> = la.neighbour_at(self.me(), MatSide::Left)?;
        while ca != la {
            s.append(ca.clone());
            ca = arc_ref(&ca)?.neighbour_at(self.me(), MatSide::Left)?;
        }
        Ok(())
    }
    /// `NearElts(S)`: the basic elements equidistant from the node, the two of one arc
    /// on two; `node.near_elts(&mut elts)?`.
    pub fn near_elts(&self, s: &mut Sequence<Handle<MatBasicElt>>) -> Result<(), Failure> {
        s.clear(None);
        let la: Handle<MatArc> = self.linked("NearElts")?;
        s.append(la.first_element());
        s.append(la.second_element());
        if !la.has_neighbour_at(self.me(), MatSide::Left)? {
            return Ok(());
        }
        let mut ca: Handle<MatArc> = la.neighbour_at(self.me(), MatSide::Left)?;
        let mut pair: bool = false;
        while ca != la {
            let arc: &MatArc = arc_ref(&ca)?;
            if pair {
                s.append(arc.first_element());
                s.append(arc.second_element());
            } else {
                pair = true;
            }
            ca = arc.neighbour_at(self.me(), MatSide::Left)?;
        }
        Ok(())
    }
    /// `Distance()`: the distance of the node to the figure; `node.distance()`.
    pub fn distance(&self) -> f64 {
        self.state.read().distance
    }
    /// `PendingNode()`: true when the node has one arc; `node.pending_node()?`.
    pub fn pending_node(&self) -> Result<bool, Failure> {
        let la: Handle<MatArc> = self.linked("PendingNode")?;
        Ok(!la.has_neighbour_at(self.me(), MatSide::Left)?)
    }
    /// `OnBasicElt()`: true when the node is on the figure (distance 0);
    /// `node.on_basic_elt()`.
    pub fn on_basic_elt(&self) -> bool {
        self.distance() == 0.0
    }
    /// `Infinite()`: true when the distance is `Precision::Infinite()`; `node.infinite()`.
    pub fn infinite(&self) -> bool {
        self.distance() == precision::infinite()
    }
    /// `SetIndex(anIndex)`; `node.set_index(2)`.
    pub fn set_index(&self, an_index: i32) {
        self.state.write().node_index = an_index;
    }
    /// `SetLinkedArc(anArc)`; `node.set_linked_arc(&arc)`.
    pub fn set_linked_arc(&self, an_arc: &Handle<MatArc>) {
        self.state.write().a_linked_arc = an_arc.downgrade();
    }
}

// The arc a neighbour handle holds; a null one is a native null dereference.
pub(super) fn arc_ref(arc: &Handle<MatArc>) -> Result<&MatArc, Failure> {
    arc.get().ok_or_else(|| {
        errors::null_object("MAT_Arc: a null neighbour arc is dereferenced; expected a linked arc")
    })
}
