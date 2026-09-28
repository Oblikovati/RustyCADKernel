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
// Rust translation of ModelingData/TKG2d/Geom2d at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The Geom2d package: transient 2D points, vectors, axes, transformations and curves.
pub mod arrays;
pub mod cell;
pub mod errors;
pub mod native;
pub mod spline_data;

mod geometry;
pub use geometry::{
    Geom2dGeometry, dump_base, dump_class_name, dump_field, dump_flag, dump_geometry,
};
mod curve;
pub use curve::{Geom2dCurve, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3, dump_curve};
mod bounded_curve;
pub use bounded_curve::{Geom2dBoundedCurve, dump_bounded_curve};
mod conic;
pub use conic::{Geom2dConic, dump_conic, reverse_conic_position};
mod point;
pub use point::{Geom2dPoint, dump_point};
mod vector;
pub use vector::Geom2dVector;

mod cartesian_point;
pub use cartesian_point::Geom2dCartesianPoint;
mod direction;
pub use direction::Geom2dDirection;
mod vector_with_magnitude;
pub use vector_with_magnitude::Geom2dVectorWithMagnitude;
mod axis_placement;
pub use axis_placement::Geom2dAxisPlacement;
mod transformation;
pub use transformation::Geom2dTransformation;

mod line;
pub use line::Geom2dLine;
mod circle;
pub use circle::Geom2dCircle;
mod ellipse;
pub use ellipse::Geom2dEllipse;
mod hyperbola;
pub use hyperbola::Geom2dHyperbola;
mod parabola;
pub use parabola::Geom2dParabola;
mod trimmed_curve;
pub use trimmed_curve::Geom2dTrimmedCurve;
mod bezier_curve;
pub use bezier_curve::Geom2dBezierCurve;
mod bspline_curve;
pub use bspline_curve::Geom2dBSplineCurve;
mod offset_curve;
pub mod offset_curve_utils;
pub use offset_curve::Geom2dOffsetCurve;
