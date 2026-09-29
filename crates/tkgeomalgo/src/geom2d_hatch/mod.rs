// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/Geom2dHatch at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package Geom2dHatch: the hatching of 2d curves by oriented Geom2d boundary
//! curves, and the classification of a 2d point against those curves.
//!
//! [`Hatcher`] intersects its hatchings with its elements ([`Intersector`], a
//! Geom2dInt_GInter) and cuts them into HatchGen domains. [`Elements`] is the face
//! explorer and [`Intersector`] the segment intersector of the TopClass templates:
//! [`FClass2dOfClassifier`] and [`Classifier`] are their instances
//! (docs/geomalgo-design.md section 8). Curves are `Geom2dAdaptor_Curve` values
//! owned by the elements and hatchings; the constructors take them by value.
mod domain_walk;
mod element;
mod elements;
mod hatcher;
mod hatcher_domains;
mod hatcher_dump;
mod hatcher_transition;
mod hatcher_trim;
mod hatching;
mod intersector;

pub use element::Element;
pub use elements::Elements;
pub use hatcher::Hatcher;
pub use hatching::Hatching;
pub use intersector::Intersector;

use crate::topclass::{Classifier2d, FaceClassifier};

/// `Geom2dHatch_FClass2dOfClassifier`: TopClass_Classifier2d over [`Intersector`].
pub type FClass2dOfClassifier = Classifier2d<Intersector>;
/// `Geom2dHatch_Classifier`: TopClass_FaceClassifier over [`Elements`].
pub type Classifier = FaceClassifier<Elements>;
