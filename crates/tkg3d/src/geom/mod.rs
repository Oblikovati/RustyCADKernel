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
// Rust translation of ModelingData/TKG3d/Geom at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The Geom package: transient 3D points, vectors, axes, transformations, curves
//! and surfaces.
pub mod errors;
pub mod protected;

mod geometry;
pub use geometry::{GeomGeometry, dump_geometry};
mod point;
pub use point::GeomPoint;
mod vector;
pub use vector::GeomVector;
mod axis_placement;
pub use axis_placement::GeomAxisPlacement;
mod curve;
pub use curve::{GeomCurve, GeomCurveD1, GeomCurveD2, GeomCurveD3, dump_curve};
mod bounded_curve;
pub use bounded_curve::{GeomBoundedCurve, dump_bounded_curve};
mod conic;
pub use conic::{GeomConic, dump_conic, reverse_conic_position};
mod surface;
pub use surface::{
    GeomSurface, GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3, SurfaceBounds, check_dn_orders,
    dump_surface,
};
mod bounded_surface;
pub use bounded_surface::GeomBoundedSurface;
mod elementary_surface;
pub use elementary_surface::{
    GeomElementarySurface, dump_elementary_surface, elementary_u_reverse, elementary_v_reverse,
};
mod swept_surface;
pub use swept_surface::{GeomSweptSurface, dump_swept_surface};

mod cartesian_point;
pub use cartesian_point::GeomCartesianPoint;
mod direction;
pub use direction::GeomDirection;
mod vector_with_magnitude;
pub use vector_with_magnitude::GeomVectorWithMagnitude;
mod axis1_placement;
pub use axis1_placement::GeomAxis1Placement;
mod axis2_placement;
pub use axis2_placement::GeomAxis2Placement;
mod transformation;
pub use transformation::GeomTransformation;

mod line;
pub use line::GeomLine;
mod circle;
pub use circle::GeomCircle;
mod ellipse;
pub use ellipse::GeomEllipse;
mod hyperbola;
pub use hyperbola::GeomHyperbola;
mod parabola;
pub use parabola::GeomParabola;
mod trimmed_curve;
pub use trimmed_curve::GeomTrimmedCurve;
pub mod offset_curve_utils;

mod elementary_results;
pub mod extrusion_utils;
mod parametric;
mod plane;
mod quadric;
pub mod revolution_utils;
pub use plane::GeomPlane;
mod cylindrical_surface;
pub use cylindrical_surface::GeomCylindricalSurface;
mod conical_surface;
pub use conical_surface::GeomConicalSurface;
mod spherical_surface;
pub use spherical_surface::GeomSphericalSurface;
mod toroidal_surface;
pub use toroidal_surface::GeomToroidalSurface;
mod surface_of_linear_extrusion;
pub use surface_of_linear_extrusion::GeomSurfaceOfLinearExtrusion;
mod surface_of_revolution;
pub use surface_of_revolution::GeomSurfaceOfRevolution;

mod bezier_curve;
pub use bezier_curve::GeomBezierCurve;
mod bspline_curve;
pub use bspline_curve::GeomBSplineCurve;
mod bezier_surface;
pub use bezier_surface::GeomBezierSurface;
