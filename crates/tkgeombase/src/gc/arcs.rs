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
// Rust translation of GC_MakeArcOfCircle, GC_MakeArcOfEllipse, GC_MakeArcOfHyperbola and
// GC_MakeArcOfParabola (.hxx/.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The 3D arc algorithms: trimmed Geom conics.
use crate::gce::{self, ErrorType, Root};
use crate::line_extrema::lines_extremum;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::geom::{
    GeomCircle, GeomCurve, GeomEllipse, GeomHyperbola, GeomParabola, GeomTrimmedCurve,
};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{
    Axis2, Circle3, Direction3, Ellipse3, Hyperbola3, Line3, Parabola3, Point3, Vector3,
};

conic_arc!(
    MakeArcOfCircle,
    "GC_MakeArcOfCircle",
    Circle3,
    Point3,
    GeomCircle::from_circ,
    GeomCurve,
    GeomTrimmedCurve
);
conic_arc!(
    MakeArcOfEllipse,
    "GC_MakeArcOfEllipse",
    Ellipse3,
    Point3,
    GeomEllipse::from_elips,
    GeomCurve,
    GeomTrimmedCurve
);
conic_arc!(
    MakeArcOfHyperbola,
    "GC_MakeArcOfHyperbola",
    Hyperbola3,
    Point3,
    GeomHyperbola::from_hypr,
    GeomCurve,
    GeomTrimmedCurve
);
conic_arc!(
    MakeArcOfParabola,
    "GC_MakeArcOfParabola",
    Parabola3,
    Point3,
    GeomParabola::from_parab,
    GeomCurve,
    GeomTrimmedCurve
);

impl MakeArcOfCircle {
    /// GC_MakeArcOfCircle(P1, P2, P3): the arc of the gce circle through the three
    /// points, from P1 (parameter 0, the circle's X axis passes through it) to the
    /// parameter of P3; the status of gce_MakeCirc.
    /// `MakeArcOfCircle::through_points(a, b, c)?`.
    pub fn through_points(p1: Point3, p2: Point3, p3: Point3) -> Result<Self, Failure> {
        let circle = gce::MakeCirc::through_points(p1, p2, p3)?;
        if circle.status() != ErrorType::Done {
            return Ok(Self::failed(circle.status()));
        }
        let circ: Circle3 = circle.value()?;
        let alpha3: f64 = elclib::parameter(&circ, p3)?;
        Self::from_parameters(circ, 0.0, alpha3, true)
    }

    /// GC_MakeArcOfCircle(P1, V, P2): the arc from P1 to P2 tangent to `tangent`
    /// at P1; its center is the middle of the closest points of the chord's
    /// bisector and the normal at P1. The status of gce_MakeLin(P1, P2);
    /// Standard_ConstructionError when the tangent is null or along the chord.
    /// `MakeArcOfCircle::from_tangent(p1, v, p2)?`.
    pub fn from_tangent(p1: Point3, tangent: Vector3, p2: Point3) -> Result<Self, Failure> {
        let chord = gce::MakeLin::from_points(p1, p2)?;
        if chord.status() != ErrorType::Done {
            return Ok(Self::failed(chord.status()));
        }
        let dir: Direction3 = chord.value()?.direction();
        let dbid: Direction3 = Direction3::from_vector(tangent)?;
        let daxe: Direction3 = dbid.crossed(dir)?;
        let dir1: Direction3 = daxe.crossed(dir)?;
        let middle = Point3::new(
            (p1.x() + p2.x()) / 2.0,
            (p1.y() + p2.y()) / 2.0,
            (p1.z() + p2.z()) / 2.0,
        );
        let d: Direction3 = dbid.crossed(daxe)?;
        // Extrema_ExtElC(bis, norm, Precision::Confusion()) has one extremum; the
        // loop keeps it unless its square distance is NaN (the center stays at the
        // origin). Parallel lines are not tested: their extremum points are the origin.
        let extremum = lines_extremum(Line3::new(middle, dir1), Line3::new(p1, d));
        let (pon1, pon2) = extremum.points;
        let center: Point3 = if extremum.square_distance < f64::MAX {
            Point3::from((pon1.xyz() + pon2.xyz()) / 2.0)
        } else {
            Point3::default()
        };
        let radius: f64 = (center.distance(&p1) + center.distance(&p2)) / 2.0;
        let circ = Circle3::new(Axis2::with_x_direction(center, daxe, d)?, radius)?;
        Self::from_points(circ, p1, p2, true)
    }
}
