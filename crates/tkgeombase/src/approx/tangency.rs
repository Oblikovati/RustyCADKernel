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
// Rust translation of ModelingData/TKGeomBase/Approx/Approx_ComputeLine.gxx, Approx_BSplComputeLine.gxx (FirstTangencyVector, LastTangencyVector, SearchFirstLambda, SearchLastLambda, TangencyVector)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The extremity tangents of the multi-line drivers: the members that
//! `Approx_ComputeLine` and `Approx_BSplComputeLine` repeat word for word, apart
//! from the parametrization they call (passed in here).
use crate::app_par_curves::arrays::{line_points, scratch};
use crate::app_par_curves::{Constraint, LeastSquare, LineTool, MultiCurve, MultiCurveEval};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};
use rustycad_tkmath::math::Vector;

/// The extremity of a MultiLine range a tangent is searched at.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LineEnd {
    /// `FirstTangencyVector`: the parabola through `index..=index + 2`, at u = 0.
    First,
    /// `LastTangencyVector`: the parabola through `index - 2..=index`, at u = 1.
    Last,
}

/// The parametrization of the calling driver (`Parameters(Line, firstP, lastP,
/// TheParameters)`).
pub(crate) type ParametersOf<'a, L> =
    &'a dyn Fn(&L, i32, i32, &mut Vector<'static>) -> Result<(), Failure>;

/// `LineTool::Tangency` through the overload the drivers pick: both arrays with 3D
/// and 2D points, else the 2D or the 3D one (false without any point).
pub(crate) fn line_tangency<L: LineTool>(
    line: &L,
    index: i32,
    tangents: &mut Array1<'_, Vector3>,
    tangents2d: &mut Array1<'_, Vector2>,
) -> Result<bool, Failure> {
    let (nb3d, nb2d) = (line.nb_p3d(), line.nb_p2d());
    if nb3d != 0 && nb2d != 0 {
        return line.tangency(index, tangents, tangents2d);
    }
    if nb2d != 0 {
        return line.tangency_2d(index, tangents2d);
    }
    if nb3d != 0 {
        return line.tangency_3d(index, tangents);
    }
    Ok(false)
}

/// `FirstTangencyVector(Line, index, V)` / `LastTangencyVector`: the tangents of
/// the line at `index` into `V(1..)` (3 coordinates per 3D point, then 2 per 2D
/// point), or the derivative of the parabola fitted through the three points
/// starting (ending) there when the line has none.
pub(crate) fn tangency_vector<L: LineTool>(
    line: &L,
    index: i32,
    end: LineEnd,
    v: &mut Vector<'_>,
    parameters: ParametersOf<'_, L>,
) -> Result<(), Failure> {
    let (nb3d, nb2d) = (line.nb_p3d(), line.nb_p2d());
    let mut tab_v: Array1<'static, Vector3> = scratch(nb3d)?;
    let mut tab_v2d: Array1<'static, Vector2> = scratch(nb2d)?;
    if line_tangency(line, index, &mut tab_v, &mut tab_v2d)? {
        return store_tangents(v, [nb3d, nb2d], &tab_v, &tab_v2d);
    }
    let (first, u): (i32, f64) = match end {
        LineEnd::First => (index, 0.0),
        LineEnd::Last => (index - 2, 1.0),
    };
    let mut mypar: Vector<'static> = Vector::with_bounds(first, first + 2)?;
    parameters(line, first, first + 2, &mut mypar)?;
    let pass: Constraint = Constraint::PassPoint;
    let mut lsq = LeastSquare::new(line, first, first + 2, pass, pass, &mypar, 3)?;
    curve_tangents(line, &lsq.bezier_value()?, u, v)
}

// V(j) = the coordinates of the line tangents, 3D from 1 then 2D from 3 NbP3d + 1.
fn store_tangents(
    v: &mut Vector<'_>,
    [nb3d, nb2d]: [i32; 2],
    tab_v: &Array1<'static, Vector3>,
    tab_v2d: &Array1<'static, Vector2>,
) -> Result<(), Failure> {
    if nb3d != 0 {
        let mut j: i32 = 1;
        for t in tab_v.iter() {
            *v.change_value(j)? = t.x();
            *v.change_value(j + 1)? = t.y();
            *v.change_value(j + 2)? = t.z();
            j += 3;
        }
    }
    if nb2d != 0 {
        let mut j: i32 = nb3d * 3 + 1;
        for t in tab_v2d.iter() {
            *v.change_value(j)? = t.x();
            *v.change_value(j + 1)? = t.y();
            j += 2;
        }
    }
    Ok(())
}

/// `TangencyVector(Line, C, U, V)` (Approx_BSplComputeLine): the first derivative
/// of every curve of `c` at `u` into `V(1..)`.
pub(crate) fn curve_tangents<L: LineTool>(
    line: &L,
    c: &MultiCurve,
    u: f64,
    v: &mut Vector<'_>,
) -> Result<(), Failure> {
    let (nb3d, nb2d) = (line.nb_p3d(), line.nb_p2d());
    let mut j: i32 = 1;
    for i in 1..=nb3d {
        let (_, d): (Point3, Vector3) = c.d1_3d(i, u)?;
        *v.change_value(j)? = d.x();
        *v.change_value(j + 1)? = d.y();
        *v.change_value(j + 2)? = d.z();
        j += 3;
    }
    j = nb3d * 3 + 1;
    for i in nb3d + 1..=nb3d + nb2d {
        let (_, d): (Point2, Vector2) = c.d1_2d(i, u)?;
        *v.change_value(j)? = d.x();
        *v.change_value(j + 1)? = d.y();
        j += 2;
    }
    Ok(())
}

/// `SearchFirstLambda` (`[index, index + 1]`) / `SearchLastLambda` (`[index - 1,
/// index]`) of Approx_ComputeLine: `dq/dw = lambda V = (p2 - p1) / (u2 - u1)` on
/// the first 3D point (else the first 2D point), signed by `(p2 - p1) . V`.
pub(crate) fn search_lambda<L: LineTool>(
    line: &L,
    parameters: &Vector<'_>,
    v: &Vector<'_>,
    [i1, i2]: [i32; 2],
) -> Result<f64, Failure> {
    let (nb3d, nb2d) = (line.nb_p3d(), line.nb_p2d());
    let (mut tab_p1, mut tab_p2) = (scratch::<Point3>(nb3d)?, scratch::<Point3>(nb3d)?);
    let (mut tab_p12d, mut tab_p22d) = (scratch::<Point2>(nb2d)?, scratch::<Point2>(nb2d)?);
    line_points(line, i1, &mut tab_p1, &mut tab_p12d)?;
    line_points(line, i2, &mut tab_p2, &mut tab_p22d)?;
    let du: f64 = *parameters.value(i2)? - *parameters.value(i1)?;
    let low: i32 = v.lower();
    let (length, dot, speed): (f64, f64, f64) = if nb3d != 0 {
        let chord: Vector3 = Vector3::between(tab_p1[1], tab_p2[1]);
        let my_v = Vector3::new(*v.value(low)?, *v.value(low + 1)?, *v.value(low + 2)?);
        (chord.magnitude(), chord.dot(my_v), my_v.magnitude())
    } else {
        let chord: Vector2 = Vector2::between(tab_p12d[1], tab_p22d[1]);
        let my_v = Vector2::new(*v.value(low)?, *v.value(low + 1)?);
        (chord.magnitude(), chord.dot(my_v), my_v.magnitude())
    };
    let lambda: f64 = length / (speed * du);
    let sign: f64 = if dot > 0.0 { 1.0 } else { -1.0 };
    Ok(sign * lambda)
}

/// The cumulated chords of `Parameters(Line, firstP, lastP, TheParameters)` for
/// the chord-length (and centripetal: the square root of each chord) types:
/// `TheParameters(i)` for `i > firstP`, from `TheParameters(firstP)` as set by the
/// caller; the chord of a multipoint is the norm over all its points.
pub(crate) fn cumulated_chords<L: LineTool>(
    line: &L,
    [first_p, last_p]: [i32; 2],
    centripetal: bool,
    parameters: &mut Vector<'_>,
) -> Result<(), Failure> {
    let (nb3d, nb2d) = (line.nb_p3d(), line.nb_p2d());
    let (mut tab_p, mut tab_pp) = (scratch::<Point3>(nb3d)?, scratch::<Point3>(nb3d)?);
    let (mut tab_p2d, mut tab_pp2d) = (scratch::<Point2>(nb2d)?, scratch::<Point2>(nb2d)?);
    for i in first_p + 1..=last_p {
        line_points(line, i - 1, &mut tab_p, &mut tab_p2d)?;
        line_points(line, i, &mut tab_pp, &mut tab_pp2d)?;
        let mut dist: f64 = 0.0;
        for j in 1..=nb3d {
            dist += tab_pp[j].square_distance(&tab_p[j]);
        }
        for j in 1..=nb2d {
            dist += tab_pp2d[j].square_distance(tab_p2d[j]);
        }
        dist = dist.sqrt();
        let step: f64 = if centripetal { dist.sqrt() } else { dist };
        *parameters.change_value(i)? = *parameters.value(i - 1)? + step;
    }
    Ok(())
}
