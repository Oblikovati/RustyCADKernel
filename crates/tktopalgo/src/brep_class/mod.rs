// Created on: 1992-11-18
// Created by: Remi LEQUETTE
// Copyright (c) 1992-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepClass at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package BRepClass: the classification of a 2d point (or of the projection of
//! a 3d point) against a face, IN, OUT or ON its boundary.
//!
//! The classifiers are the TopClass templates of TKGeomAlgo instantiated on the
//! BRepClass edge, intersector and face explorer (docs/topalgo-design.md section 2):
//! [`FClass2dOfFClassifier`] and [`FacePassiveClassifier`] are
//! `topclass::Classifier2d<BRepClassIntersector>`, [`FClassifier`] is
//! `topclass::FaceClassifier<BRepClassFaceExplorer>`, and
//! [`BRepClassFaceClassifier`] derives from it.
mod edge;
mod face_classifier;
mod face_explorer;
mod face_explorer_build;
mod face_explorer_segment;
mod intersector;
mod intersector_checks;

pub use edge::{BRepClassEdge, BndBoxState};
pub use face_classifier::BRepClassFaceClassifier;
pub use face_explorer::BRepClassFaceExplorer;
pub use intersector::BRepClassIntersector;

use rustycad_tkgeomalgo::topclass::{Classifier2d, FaceClassifier};

/// `BRepClass_FClass2dOfFClassifier`: the TopClass 2d classifier over the BRepClass
/// intersector; `let mut classifier = FClass2dOfFClassifier::new();`.
pub type FClass2dOfFClassifier = Classifier2d<BRepClassIntersector>;

/// `BRepClass_FacePassiveClassifier`: the same TopClass 2d classifier (its header is
/// identical to `BRepClass_FClass2dOfFClassifier`);
/// `let mut classifier = FacePassiveClassifier::new();`.
pub type FacePassiveClassifier = Classifier2d<BRepClassIntersector>;

/// `BRepClass_FClassifier`: the TopClass face classifier over the BRepClass face
/// explorer; `FClassifier::with_point(&mut explorer, &point, 1e-7)?`.
pub type FClassifier = FaceClassifier<BRepClassFaceExplorer>;
