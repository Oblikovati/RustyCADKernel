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
// Rust translation of FoundationClasses/TKMath/GeomAbs/*.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! GeomAbs enumerations shared by curve, surface and approximation algorithms.
//!
//! Discriminants keep the native declaration order, so `as i32` yields the
//! integer OCCT stores for the corresponding `GeomAbs_*` enumerator.

/// Continuity of a curve or surface (`GeomAbs_Shape`); `Shape::C2 as i32 == 4`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[repr(i32)]
pub enum Shape {
    /// Only geometric continuity of position.
    C0,
    /// Tangent directions are continuous.
    G1,
    /// First derivatives are continuous.
    C1,
    /// Curvature is continuous.
    G2,
    /// Second derivatives are continuous.
    C2,
    /// Third derivatives are continuous.
    C3,
    /// Infinitely continuous.
    CN,
}

/// Kind of curve behind an adaptor (`GeomAbs_CurveType`); `CurveType::Line as i32 == 0`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum CurveType {
    /// Straight line.
    Line,
    /// Circle.
    Circle,
    /// Ellipse.
    Ellipse,
    /// Hyperbola.
    Hyperbola,
    /// Parabola.
    Parabola,
    /// Bezier curve.
    BezierCurve,
    /// B-spline curve.
    BSplineCurve,
    /// Offset curve.
    OffsetCurve,
    /// Any other curve.
    OtherCurve,
}

/// Kind of surface behind an adaptor (`GeomAbs_SurfaceType`); `SurfaceType::Torus as i32 == 4`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum SurfaceType {
    /// Plane.
    Plane,
    /// Cylinder.
    Cylinder,
    /// Cone.
    Cone,
    /// Sphere.
    Sphere,
    /// Torus.
    Torus,
    /// Bezier surface.
    BezierSurface,
    /// B-spline surface.
    BSplineSurface,
    /// Surface of revolution.
    SurfaceOfRevolution,
    /// Surface of linear extrusion.
    SurfaceOfExtrusion,
    /// Offset surface.
    OffsetSurface,
    /// Any other surface.
    OtherSurface,
}

/// Isoparametric direction (`GeomAbs_IsoType`); `IsoType::IsoV as i32 == 1`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum IsoType {
    /// Iso-U curve.
    IsoU,
    /// Iso-V curve.
    IsoV,
    /// Not an isoparametric curve.
    NoneIso,
}

/// Offset corner treatment (`GeomAbs_JoinType`); `JoinType::Intersection as i32 == 2`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum JoinType {
    /// Corners are filled with arcs.
    Arc,
    /// Corners are filled with tangent extensions.
    Tangent,
    /// Corners are filled by intersecting the offset elements.
    Intersection,
}

/// Knot distribution of a B-spline (`GeomAbs_BSplKnotDistribution`); `BSplKnotDistribution::Uniform as i32 == 1`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum BSplKnotDistribution {
    /// Arbitrary knot spacing.
    NonUniform,
    /// Uniform knots with uniform multiplicities.
    Uniform,
    /// Uniform interior with multiplicity degree+1 end knots.
    QuasiUniform,
    /// Every knot has multiplicity degree.
    PiecewiseBezier,
}
