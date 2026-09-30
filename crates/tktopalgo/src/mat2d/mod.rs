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
// Rust translation of ModelingAlgorithms/TKTopAlgo/MAT2d at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package MAT2d: the bisecting locus (the medial axis) of a 2D figure.
//!
//! A figure is a sequence of lines, each a sequence of trimmed curves
//! ([`Mat2dFigure`]). [`Mat2dCutCurve`] cuts the curves at their extrema of curvature
//! and inflections; [`Mat2dCircuit`] links the lines into one circuit of items
//! (curves and corner points) along the connexions of a [`Mat2dMiniPath`];
//! [`Mat2dTool2d`] holds the geometry (the bisectors of the `bisector` package) and
//! [`Mat2dMat2d`] computes the trees of `mat` bisectors (docs/topalgo-design.md
//! section 3).
mod bi_int;
mod circuit;
mod connexion;
mod cut_curve;
#[allow(clippy::module_inception)]
mod mat2d;
mod mini_path;
mod seq;
mod tool2d;

pub use bi_int::Mat2dBiInt;
pub use circuit::Mat2dCircuit;
pub use connexion::Mat2dConnexion;
pub use cut_curve::Mat2dCutCurve;
pub use mat2d::Mat2dMat2d;
pub use mini_path::Mat2dMiniPath;
pub use tool2d::Mat2dTool2d;

use crate::mat::MatEdge;
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dGeometry, errors};

/// A line of a figure: its items (`NCollection_Sequence<occ::handle<Geom2d_Geometry>>`).
pub type Mat2dLine = Sequence<Handle<dyn Geom2dGeometry>>;
/// A figure: its lines (`NCollection_Sequence<NCollection_Sequence<...>>`).
pub type Mat2dFigure = Sequence<Mat2dLine>;

/// The number of the edge of a MAT bisector (`FirstEdge()->EdgeNumber()`); a null
/// edge is a native null dereference.
pub(crate) fn mat_edge_number(edge: &Handle<MatEdge>) -> Result<i32, Failure> {
    edge.get().map(MatEdge::edge_number).ok_or_else(|| {
        errors::null_object("MAT2d: a bisector has a null edge; expected its two edges")
    })
}
