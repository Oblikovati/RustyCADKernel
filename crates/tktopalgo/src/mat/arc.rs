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
// Rust translation of ModelingAlgorithms/TKTopAlgo/MAT/MAT_Arc.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MAT_Arc`: an arc of the bisecting locus graph, between two nodes and two basic
//! elements, with its neighbours at each node.
use super::{MatBasicElt, MatNode, MatSide};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, WeakHandle};
use rustycad_tkg2d::geom2d::errors;

struct ArcState {
    arc_index: i32,
    geom_index: i32,
    first_element: Handle<MatBasicElt>,
    second_element: Handle<MatBasicElt>,
    first_node: Handle<MatNode>,
    second_node: Handle<MatNode>,
    first_arc_left: WeakHandle<MatArc>,
    first_arc_right: WeakHandle<MatArc>,
    second_arc_right: WeakHandle<MatArc>,
    second_arc_left: WeakHandle<MatArc>,
}

// The end of an arc a node is: its first node, its second one, or neither.
#[derive(Clone, Copy, PartialEq)]
enum End {
    First,
    Second,
}

/// `MAT_Arc`: a transient arc of a [`super::MatGraph`]. The neighbour arcs are raw
/// pointers natively; they are [`WeakHandle`]s here (docs/topalgo-design.md section
/// 3): `HasNeighbour` tests the pointer, `Neighbour` gives a null handle for a
/// destroyed arc where C++ would read a dangling pointer.
/// ```
/// use rustycad_tktopalgo::mat::{MatArc, MatBasicElt};
/// use rustycad_tkernel::standard::Handle;
/// let (e1, e2) = (Handle::new(MatBasicElt::new(1)), Handle::new(MatBasicElt::new(2)));
/// let arc = MatArc::new(1, 7, &e1, &e2);
/// assert_eq!((arc.index(), arc.geom_index()), (1, 7));
/// assert!(arc.first_node().is_null());
/// ```
pub struct MatArc {
    parent: StandardTransient,
    state: SharedState<ArcState>,
}
impl_transient!(MatArc, "MAT_Arc", StandardTransient, parent);

impl MatArc {
    /// `MAT_Arc(ArcIndex, GeomIndex, FirstElement, SecondElement)`: no nodes and no
    /// neighbours yet; `MatArc::new(1, 1, &e1, &e2)`.
    pub fn new(
        arc_index: i32,
        geom_index: i32,
        first_element: &Handle<MatBasicElt>,
        second_element: &Handle<MatBasicElt>,
    ) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(ArcState {
                arc_index,
                geom_index,
                first_element: first_element.clone(),
                second_element: second_element.clone(),
                first_node: Handle::default(),
                second_node: Handle::default(),
                first_arc_left: WeakHandle::default(),
                first_arc_right: WeakHandle::default(),
                second_arc_right: WeakHandle::default(),
                second_arc_left: WeakHandle::default(),
            }),
        }
    }
    /// `Index()`: the index in the arcs of the graph; `arc.index()`.
    pub fn index(&self) -> i32 {
        self.state.read().arc_index
    }
    /// `GeomIndex()`: the index of the geometry of the arc; `arc.geom_index()`.
    pub fn geom_index(&self) -> i32 {
        self.state.read().geom_index
    }
    /// `FirstElement()`; `arc.first_element()`.
    pub fn first_element(&self) -> Handle<MatBasicElt> {
        self.state.read().first_element.clone()
    }
    /// `SecondElement()`; `arc.second_element()`.
    pub fn second_element(&self) -> Handle<MatBasicElt> {
        self.state.read().second_element.clone()
    }
    /// `FirstNode()`; `arc.first_node()`.
    pub fn first_node(&self) -> Handle<MatNode> {
        self.state.read().first_node.clone()
    }
    /// `SecondNode()`; `arc.second_node()`.
    pub fn second_node(&self) -> Handle<MatNode> {
        self.state.read().second_node.clone()
    }
    /// `TheOtherNode(aNode)`: the other end of the arc; Standard_DomainError when
    /// `a_node` is not an end; `arc.the_other_node(&node)?`.
    pub fn the_other_node(&self, a_node: &Handle<MatNode>) -> Result<Handle<MatNode>, Failure> {
        let state = self.state.read();
        match end_of(&state, a_node.object_address()) {
            Some(End::First) => Ok(state.second_node.clone()),
            Some(End::Second) => Ok(state.first_node.clone()),
            None => Err(not_an_end("TheOtherNode")),
        }
    }
    /// `HasNeighbour(aNode, aSide)`: true when an arc is linked at the end `a_node` on
    /// the side `a_side`; Standard_DomainError when `a_node` is not an end;
    /// `arc.has_neighbour(&node, MatSide::Left)?`.
    pub fn has_neighbour(
        &self,
        a_node: &Handle<MatNode>,
        a_side: MatSide,
    ) -> Result<bool, Failure> {
        self.has_neighbour_at(a_node.object_address(), a_side)
    }
    /// `Neighbour(aNode, aSide)`: the arc linked at the end `a_node` on the side
    /// `a_side` (null when there is none); Standard_DomainError when `a_node` is not
    /// an end; `arc.neighbour(&node, MatSide::Left)?`.
    pub fn neighbour(
        &self,
        a_node: &Handle<MatNode>,
        a_side: MatSide,
    ) -> Result<Handle<MatArc>, Failure> {
        self.neighbour_at(a_node.object_address(), a_side)
    }
    // HasNeighbour of the node at `address` (a node can ask about itself).
    pub(super) fn has_neighbour_at(
        &self,
        address: *const (),
        a_side: MatSide,
    ) -> Result<bool, Failure> {
        let state = self.state.read();
        let end: End = end_of(&state, address).ok_or_else(|| not_an_end("HasNeighbour"))?;
        Ok(!link(&state, end, a_side).is_null())
    }
    // Neighbour of the node at `address`.
    pub(super) fn neighbour_at(
        &self,
        address: *const (),
        a_side: MatSide,
    ) -> Result<Handle<MatArc>, Failure> {
        let state = self.state.read();
        let end: End = end_of(&state, address).ok_or_else(|| not_an_end("Neighbour"))?;
        Ok(link(&state, end, a_side).upgrade())
    }
    /// `SetIndex(anInteger)`; `arc.set_index(2)`.
    pub fn set_index(&self, an_integer: i32) {
        self.state.write().arc_index = an_integer;
    }
    /// `SetGeomIndex(anInteger)`; `arc.set_geom_index(2)`.
    pub fn set_geom_index(&self, an_integer: i32) {
        self.state.write().geom_index = an_integer;
    }
    /// `SetFirstElement(aBasicElt)`; `arc.set_first_element(&elt)`.
    pub fn set_first_element(&self, a_basic_elt: &Handle<MatBasicElt>) {
        self.state.write().first_element = a_basic_elt.clone();
    }
    /// `SetSecondElement(aBasicElt)`; `arc.set_second_element(&elt)`.
    pub fn set_second_element(&self, a_basic_elt: &Handle<MatBasicElt>) {
        self.state.write().second_element = a_basic_elt.clone();
    }
    /// `SetFirstNode(aNode)`; `arc.set_first_node(&node)`.
    pub fn set_first_node(&self, a_node: &Handle<MatNode>) {
        self.state.write().first_node = a_node.clone();
    }
    /// `SetSecondNode(aNode)`; `arc.set_second_node(&node)`.
    pub fn set_second_node(&self, a_node: &Handle<MatNode>) {
        self.state.write().second_node = a_node.clone();
    }
    /// `SetFirstArc(aSide, anArc)`: the neighbour at the first node;
    /// `arc.set_first_arc(MatSide::Left, &other)`.
    pub fn set_first_arc(&self, a_side: MatSide, an_arc: &Handle<MatArc>) {
        *link_mut(&mut self.state.write(), End::First, a_side) = an_arc.downgrade();
    }
    /// `SetSecondArc(aSide, anArc)`: the neighbour at the second node;
    /// `arc.set_second_arc(MatSide::Right, &other)`.
    pub fn set_second_arc(&self, a_side: MatSide, an_arc: &Handle<MatArc>) {
        *link_mut(&mut self.state.write(), End::Second, a_side) = an_arc.downgrade();
    }
    /// `SetNeighbour(aSide, aNode, anArc)`: the neighbour at the end `a_node`;
    /// Standard_DomainError when `a_node` is not an end;
    /// `arc.set_neighbour(MatSide::Left, &node, &other)?`.
    pub fn set_neighbour(
        &self,
        a_side: MatSide,
        a_node: &Handle<MatNode>,
        an_arc: &Handle<MatArc>,
    ) -> Result<(), Failure> {
        let mut state = self.state.write();
        let end: End =
            end_of(&state, a_node.object_address()).ok_or_else(|| not_an_end("SetNeighbour"))?;
        *link_mut(&mut state, end, a_side) = an_arc.downgrade();
        Ok(())
    }
}

// The end that the node at `address` is; the first node wins when both are it.
fn end_of(state: &ArcState, address: *const ()) -> Option<End> {
    if state.first_node.object_address() == address {
        return Some(End::First);
    }
    if state.second_node.object_address() == address {
        return Some(End::Second);
    }
    None
}

fn link(state: &ArcState, end: End, side: MatSide) -> &WeakHandle<MatArc> {
    match (end, side) {
        (End::First, MatSide::Left) => &state.first_arc_left,
        (End::First, MatSide::Right) => &state.first_arc_right,
        (End::Second, MatSide::Left) => &state.second_arc_left,
        (End::Second, MatSide::Right) => &state.second_arc_right,
    }
}

fn link_mut(state: &mut ArcState, end: End, side: MatSide) -> &mut WeakHandle<MatArc> {
    match (end, side) {
        (End::First, MatSide::Left) => &mut state.first_arc_left,
        (End::First, MatSide::Right) => &mut state.first_arc_right,
        (End::Second, MatSide::Left) => &mut state.second_arc_left,
        (End::Second, MatSide::Right) => &mut state.second_arc_right,
    }
}

// Standard_DomainError of a node that is not an end of the arc.
fn not_an_end(member: &str) -> Failure {
    errors::domain(&format!(
        "MAT_Arc::{member}: the node is not an end of the arc; expected its first or second node"
    ))
}
