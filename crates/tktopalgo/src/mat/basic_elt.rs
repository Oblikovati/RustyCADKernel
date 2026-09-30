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
// Rust translation of ModelingAlgorithms/TKTopAlgo/MAT/MAT_BasicElt.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MAT_BasicElt`: a basic element of the figure (an item of the contour) and the
//! arcs that start and end its zone of proximity.
use super::MatArc;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient, WeakHandle};

struct BasicEltState {
    start_left_arc: WeakHandle<MatArc>,
    end_left_arc: WeakHandle<MatArc>,
    index: i32,
    geom_index: i32,
}

/// `MAT_BasicElt`: a transient basic element of a [`super::MatGraph`]; its start and
/// end arcs are raw pointers natively, [`rustycad_tkernel::standard::WeakHandle`]s here
/// (docs/topalgo-design.md section 3).
/// ```
/// use rustycad_tktopalgo::mat::MatBasicElt;
/// let elt = MatBasicElt::new(3);
/// assert_eq!((elt.index(), elt.geom_index()), (3, 0));
/// assert!(elt.start_arc().is_null());
/// ```
pub struct MatBasicElt {
    parent: StandardTransient,
    state: SharedState<BasicEltState>,
}
impl_transient!(MatBasicElt, "MAT_BasicElt", StandardTransient, parent);

impl MatBasicElt {
    /// `MAT_BasicElt(anInteger)`: the element of index `an_integer`, geometry index 0;
    /// `MatBasicElt::new(1)`.
    pub fn new(an_integer: i32) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(BasicEltState {
                start_left_arc: WeakHandle::default(),
                end_left_arc: WeakHandle::default(),
                index: an_integer,
                geom_index: 0,
            }),
        }
    }
    /// `StartArc()`: the arc that starts the zone (null when unset or destroyed);
    /// `elt.start_arc()`.
    pub fn start_arc(&self) -> Handle<MatArc> {
        self.state.read().start_left_arc.upgrade()
    }
    /// `EndArc()`: the arc that ends the zone; `elt.end_arc()`.
    pub fn end_arc(&self) -> Handle<MatArc> {
        self.state.read().end_left_arc.upgrade()
    }
    /// `Index()`: the index in the basic elements of the graph; `elt.index()`.
    pub fn index(&self) -> i32 {
        self.state.read().index
    }
    /// `GeomIndex()`: the index of the geometry of the element; `elt.geom_index()`.
    pub fn geom_index(&self) -> i32 {
        self.state.read().geom_index
    }
    /// `SetStartArc(anArc)`; `elt.set_start_arc(&arc)`.
    pub fn set_start_arc(&self, an_arc: &Handle<MatArc>) {
        self.state.write().start_left_arc = an_arc.downgrade();
    }
    /// `SetEndArc(anArc)`; `elt.set_end_arc(&arc)`.
    pub fn set_end_arc(&self, an_arc: &Handle<MatArc>) {
        self.state.write().end_left_arc = an_arc.downgrade();
    }
    /// `SetIndex(anInteger)`; `elt.set_index(2)`.
    pub fn set_index(&self, an_integer: i32) {
        self.state.write().index = an_integer;
    }
    /// `SetGeomIndex(anInteger)`; `elt.set_geom_index(2)`.
    pub fn set_geom_index(&self, an_integer: i32) {
        self.state.write().geom_index = an_integer;
    }
}
