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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntRes2d at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package IntRes2d: the results of the intersection of two 2D curves.
//!
//! [`Domain`] bounds the parameter range of a curve, [`Transition`] describes how
//! the curves cross at a point, [`IntersectionPoint`] and [`IntersectionSegment`]
//! are the results, and [`Intersection`] is the root class of every 2D curve-curve
//! intersection algorithm (IntCurve, Geom2dInt): the algorithms embed it and reach
//! its accessors through `Deref`. Its protected members (`append_point`, `insert`,
//! `set_values`...) are public here so that algorithms of other crates can derive
//! from it, as native classes of other toolkits do.
mod domain;
mod failures;
mod intersection;
mod intersection_point;
mod intersection_segment;
mod transition;

pub use domain::Domain;
pub(crate) use failures::{construction_error, domain_error, not_done};
pub use intersection::Intersection;
pub use intersection_point::IntersectionPoint;
pub use intersection_segment::IntersectionSegment;
pub use transition::Transition;

/// `IntRes2d_Position`: where an intersection lies on a curve.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Position {
    /// `IntRes2d_Head`: at the first point of the domain.
    Head,
    /// `IntRes2d_Middle`: inside the domain.
    Middle,
    /// `IntRes2d_End`: at the last point of the domain.
    End,
}

/// `IntRes2d_Situation`: the side of a TOUCH transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Situation {
    /// `IntRes2d_Inside`: the curve stays inside the other one.
    Inside,
    /// `IntRes2d_Outside`: the curve stays outside the other one.
    Outside,
    /// `IntRes2d_Unknown`: the second derivatives cannot decide.
    Unknown,
}

/// `IntRes2d_TypeTrans`: the kind of transition at an intersection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TypeTrans {
    /// `IntRes2d_In`: the curve enters the other one.
    In,
    /// `IntRes2d_Out`: the curve leaves the other one.
    Out,
    /// `IntRes2d_Touch`: the curve touches the other one without crossing.
    Touch,
    /// `IntRes2d_Undecided`: the curves are superposed.
    Undecided,
}
