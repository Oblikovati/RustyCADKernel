// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of the D0/D2 helpers of GCPnts_TangentialDeflection.cxx, of
// GCPnts_TCurveTypes.hxx and of GCPnts_DistFunction(2d).cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The curve operations of the `GCPnts_TangentialDeflection` templates, for its two
//! instantiations: `Adaptor3d_Curve` and `Adaptor2d_Curve2d`. A 2D point or vector is
//! lifted to 3D with a zero Z, as the native `D0`/`D2` overloads do, so the marching
//! arithmetic is the same 3D arithmetic for both.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{self, Direction2, Direction3, Line2, Line3, Point3, Vector3, Xyz};

/// A point and its first two derivatives, lifted to 3D.
pub(crate) type LiftedD2 = (Point3, Vector3, Vector3);

/// The template curve of `GCPnts_TangentialDeflection` (the `TheCurve` parameter).
pub(crate) trait DeflectionCurve {
    /// `D0`, lifted to 3D.
    fn lifted_d0(&self, u: f64) -> Result<Point3, Failure>;
    /// `D2`, lifted to 3D.
    fn lifted_d2(&self, u: f64) -> Result<LiftedD2, Failure>;
    /// `FirstParameter()`.
    fn first(&self) -> Result<f64, Failure>;
    /// `LastParameter()`.
    fn last(&self) -> Result<f64, Failure>;
    /// `GetType()`.
    fn kind(&self) -> Result<CurveType, Failure>;
    /// `Circle().Radius()`.
    fn circle_radius(&self) -> Result<f64, Failure>;
    /// `BSpline()->NbPoles()` and `BSpline()->Degree()`.
    fn bspline_shape(&self) -> Result<(i32, i32), Failure>;
    /// `Bezier()->NbPoles()` and `Bezier()->Degree()`.
    fn bezier_shape(&self) -> Result<(i32, i32), Failure>;
    /// `NbIntervals(GeomAbs_CN)` then `Intervals(Intervs, GeomAbs_CN)` into `[1, N + 1]`.
    fn cn_intervals(&self) -> Result<Array1<'static, f64>, Failure>;
    /// `GCPnts_DistFunction(2d)(C, U1, U2)`: the chord line of `[u1, u2]`.
    fn chord(&self, u1: f64, u2: f64) -> Result<ChordLine, Failure>;
    /// `GCPnts_DistFunction(2d)::Value(X)` without the range check: minus the square
    /// distance from the curve point at `x` to the chord line.
    fn chord_value(&self, chord: &ChordLine, x: f64) -> Result<f64, Failure>;
}

/// The `myLin` of GCPnts_DistFunction (3D) or GCPnts_DistFunction2d (2D).
#[derive(Clone, Copy, Debug)]
pub(crate) enum ChordLine {
    /// `gp_Lin2d` of the 2D instantiation.
    Planar(Line2),
    /// `gp_Lin` of the 3D instantiation.
    Spatial(Line3),
}

/// `Adaptor2d_Curve2d` as the template curve.
pub(crate) struct PlanarCurve<'a>(pub &'a dyn Adaptor2dCurve2d);

/// `Adaptor3d_Curve` as the template curve.
pub(crate) struct SpatialCurve<'a>(pub &'a dyn Adaptor3dCurve);

// `NbIntervals(GeomAbs_CN)` and `Intervals` into a 1-based array.
fn interval_bounds(
    count: i32,
    fill: impl FnOnce(&mut Array1<'static, f64>) -> Result<(), Failure>,
) -> Result<Array1<'static, f64>, Failure> {
    let mut bounds: Array1<'static, f64> =
        Array1::with_bounds(1, count + 1).map_err(|error| error.into_failure("Intervals"))?;
    fill(&mut bounds)?;
    Ok(bounds)
}

impl DeflectionCurve for PlanarCurve<'_> {
    fn lifted_d0(&self, u: f64) -> Result<Point3, Failure> {
        let point = self.0.d0(u)?;
        Ok(Point3::new(point.x(), point.y(), 0.0))
    }
    fn lifted_d2(&self, u: f64) -> Result<LiftedD2, Failure> {
        let (point, d1, d2) = self.0.d2(u)?;
        Ok((
            Point3::new(point.x(), point.y(), 0.0),
            Vector3::new(d1.x(), d1.y(), 0.0),
            Vector3::new(d2.x(), d2.y(), 0.0),
        ))
    }
    fn first(&self) -> Result<f64, Failure> {
        self.0.first_parameter()
    }
    fn last(&self) -> Result<f64, Failure> {
        self.0.last_parameter()
    }
    fn kind(&self) -> Result<CurveType, Failure> {
        self.0.get_type()
    }
    fn circle_radius(&self) -> Result<f64, Failure> {
        Ok(self.0.circle()?.radius())
    }
    fn bspline_shape(&self) -> Result<(i32, i32), Failure> {
        let curve = self.0.bspline()?;
        let curve = curve.get().ok_or_else(null_spline)?;
        Ok((curve.nb_poles(), curve.degree()))
    }
    fn bezier_shape(&self) -> Result<(i32, i32), Failure> {
        let curve = self.0.bezier()?;
        let curve = curve.get().ok_or_else(null_spline)?;
        Ok((curve.nb_poles(), curve.degree()))
    }
    fn cn_intervals(&self) -> Result<Array1<'static, f64>, Failure> {
        let count: i32 = self.0.nb_intervals(Shape::CN)?;
        interval_bounds(count, |bounds| self.0.intervals(bounds, Shape::CN))
    }
    fn chord(&self, u1: f64, u2: f64) -> Result<ChordLine, Failure> {
        let first = self.0.value(u1)?;
        let second = self.0.value(u2)?;
        let second = if first.square_distance(second) > gp::resolution() {
            second
        } else {
            // For #28812
            self.0.d0(u1 + 0.01 * (u2 - u1))?
        };
        let direction = Direction2::from_xy(second.xy() - first.xy())?;
        Ok(ChordLine::Planar(Line2::new(first, direction)))
    }
    fn chord_value(&self, chord: &ChordLine, x: f64) -> Result<f64, Failure> {
        let ChordLine::Planar(line) = chord else {
            return Err(Failure::new(
                "GCPnts_DistFunction2d: a 3D chord line; expected gp_Lin2d",
            ));
        };
        Ok(-line.square_distance(self.0.value(x)?))
    }
}

impl DeflectionCurve for SpatialCurve<'_> {
    fn lifted_d0(&self, u: f64) -> Result<Point3, Failure> {
        self.0.d0(u)
    }
    fn lifted_d2(&self, u: f64) -> Result<LiftedD2, Failure> {
        self.0.d2(u)
    }
    fn first(&self) -> Result<f64, Failure> {
        self.0.first_parameter()
    }
    fn last(&self) -> Result<f64, Failure> {
        self.0.last_parameter()
    }
    fn kind(&self) -> Result<CurveType, Failure> {
        self.0.get_type()
    }
    fn circle_radius(&self) -> Result<f64, Failure> {
        Ok(self.0.circle()?.radius())
    }
    fn bspline_shape(&self) -> Result<(i32, i32), Failure> {
        let curve = self.0.bspline()?;
        let curve = curve.get().ok_or_else(null_spline)?;
        Ok((curve.nb_poles(), curve.degree()))
    }
    fn bezier_shape(&self) -> Result<(i32, i32), Failure> {
        let curve = self.0.bezier()?;
        let curve = curve.get().ok_or_else(null_spline)?;
        Ok((curve.nb_poles(), curve.degree()))
    }
    fn cn_intervals(&self) -> Result<Array1<'static, f64>, Failure> {
        let count: i32 = self.0.nb_intervals(Shape::CN)?;
        interval_bounds(count, |bounds| self.0.intervals(bounds, Shape::CN))
    }
    fn chord(&self, u1: f64, u2: f64) -> Result<ChordLine, Failure> {
        let first = self.0.value(u1)?;
        let second = self.0.value(u2)?;
        let second = if first.square_distance(&second) > gp::resolution() {
            second
        } else {
            // For #28812
            self.0.d0(u1 + 0.01 * (u2 - u1))?
        };
        let direction: Direction3 = Direction3::from_xyz(second.xyz() - first.xyz())?;
        Ok(ChordLine::Spatial(Line3::new(first, direction)))
    }
    fn chord_value(&self, chord: &ChordLine, x: f64) -> Result<f64, Failure> {
        let ChordLine::Spatial(line) = chord else {
            return Err(Failure::new(
                "GCPnts_DistFunction: a 2D chord line; expected gp_Lin",
            ));
        };
        Ok(-line.square_distance(self.0.value(x)?))
    }
}

// Natively a null spline handle is dereferenced; the port reports it.
fn null_spline() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "GCPnts_TangentialDeflection: the adaptor returned a null spline handle; expected a curve",
        "",
    )
}

/// `EstimAngl`: the sine of the angle between `p1 -> pm` and `pm -> p2`, 0 for a
/// degenerate pair.
pub(crate) fn estimate_angle(p1: Point3, pm: Point3, p2: Point3) -> f64 {
    let v1: Vector3 = Vector3::between(p1, pm);
    let v2: Vector3 = Vector3::between(pm, p2);
    let length: f64 = v1.magnitude() * v2.magnitude();
    if length > gp::resolution() {
        return v1.cross_magnitude(v2) / length;
    }
    0.0
}

/// `P.XYZ() - Q.XYZ()`.
pub(crate) fn difference(p: Point3, q: Point3) -> Xyz {
    p.xyz() - q.xyz()
}
