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
// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepClass3d at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package BRepClass3d: the classification of a point against a solid, IN, OUT
//! or ON its boundary, by the parity of the crossings of a segment from the point.
//!
//! docs/topalgo-design.md section 5 fixes the mapping: [`BRepClass3dSolidExplorer`]
//! owns the face intersectors and the UB-tree (`NCollection_UBTree<int, Bnd_Box>`)
//! of the edges and vertices, which the two selectors of BRepClass3d_BndBoxTree.hxx
//! query; [`BRepClass3dSClassifier`] classifies with an explorer and
//! [`BRepClass3dSolidClassifier`] with its own one.
mod bnd_box_tree;
mod find_point;
mod intersector3d;
mod other_segment;
mod outer_shell;
mod point_in_face;
mod sclassifier;
mod sclassifier_perform;
mod solid_classifier;
mod solid_explorer;
mod transitions;

pub use bnd_box_tree::{
    BRepClass3dBndBoxTreeSelectorLine, BRepClass3dBndBoxTreeSelectorPoint, EdgeParam, VertParam,
};
pub use find_point::FacePointSample;
pub use intersector3d::BRepClass3dIntersector3d;
pub use outer_shell::BRepClass3d;
pub use point_in_face::UvRectangle;
pub use sclassifier::BRepClass3dSClassifier;
pub use solid_classifier::BRepClass3dSolidClassifier;
pub use solid_explorer::{BRepClass3dMapOfInter, BRepClass3dSolidExplorer};
