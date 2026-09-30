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
// Rust translation of ModelingAlgorithms/TKTopAlgo/MAT (MAT_Side.hxx and the package) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package MAT: the topology of the bisecting locus (the medial axis
//! transform), independent of the geometry.
//!
//! The algorithm of MAT2d builds a tree of [`MatBisector`]s between [`MatEdge`]s of a
//! contour, in the lists [`MatListOfBisector`] and [`MatListOfEdge`] (the generic
//! [`MatTList`], docs/topalgo-design.md section 3). [`MatGraph`] turns the roots of the
//! trees into a graph of [`MatArc`]s, [`MatNode`]s and [`MatBasicElt`]s, and
//! [`MatZone`] gives the zone of proximity of a basic element.
mod arc;
mod basic_elt;
mod bisector;
mod edge;
mod graph;
mod node;
mod tlist;
mod zone;

pub use arc::MatArc;
pub use basic_elt::MatBasicElt;
pub use bisector::MatBisector;
pub use edge::MatEdge;
pub use graph::{MatFusion, MatGraph};
pub use node::MatNode;
pub use tlist::{MatTList, MatTListItem, MatTListNode};
pub use zone::MatZone;

/// `MAT_Side`: a side of an arc or of a contour; `MatSide::Left`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MatSide {
    /// `MAT_Left`.
    Left,
    /// `MAT_Right`.
    Right,
}

/// `MAT_ListOfBisector`: the [`MatTList`] of [`MatBisector`]s.
pub type MatListOfBisector = MatTList<MatBisector>;
/// `MAT_ListOfEdge`: the [`MatTList`] of [`MatEdge`]s.
pub type MatListOfEdge = MatTList<MatEdge>;
/// `MAT_TListNodeOfListOfBisector`: the node of a [`MatListOfBisector`].
pub type MatTListNodeOfListOfBisector = MatTListNode<MatBisector>;
/// `MAT_TListNodeOfListOfEdge`: the node of a [`MatListOfEdge`].
pub type MatTListNodeOfListOfEdge = MatTListNode<MatEdge>;
