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
// Rust translation of ModelingData/TKGeomBase/Approx/Approx_MCurvesToBSpCurve.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Approx_MCurvesToBSpCurve`: the B-spline MultiCurve joining a sequence of Bezier
//! MultiCurves end to end.
use crate::app_par_curves::arrays::scratch;
use crate::app_par_curves::{MultiBSpCurve, MultiCurve, MultiCurveEval, MultiPoint};
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::convert::{
    CompBezierCurves2dToBSplineCurve2d, CompBezierCurvesToBSplineCurve,
};
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkmath::plib::PolePoint;

/// `Approx_MCurvesToBSpCurve`: the first curve of the sequence (the first 3D curve,
/// else the first 2D curve) is joined by `Convert_CompBezierCurves*ToBSplineCurve*`,
/// which chooses the degree, the knots and the continuity; the other curves are
/// raised to that degree and their poles copied span by span, dropping the pole
/// each junction shares.
///
/// ```ignore
/// let mut join = MCurvesToBSpCurve::new();
/// join.append(&first);
/// join.append(&second);
/// join.perform()?;
/// let spline = join.value();
/// ```
#[derive(Clone, Default)]
pub struct MCurvesToBSpCurve {
    spline: MultiBSpCurve,
    done: bool,
    curves: Vec<MultiCurve>,
}

/// The degree, knots and multiplicities the Convert join of the first curve gives.
struct Junction {
    degree: i32,
    knots: Array1<'static, f64>,
    mults: Array1<'static, i32>,
}

impl MCurvesToBSpCurve {
    /// `Approx_MCurvesToBSpCurve()`: an empty sequence; `MCurvesToBSpCurve::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Reset()`: clears the sequence (the result stays); `join.reset()`.
    pub fn reset(&mut self) {
        self.done = false;
        self.curves.clear();
    }

    /// `Append(MC)`: appends a copy sharing the poles of `mc`; `join.append(&curve)`.
    pub fn append(&mut self, mc: &MultiCurve) {
        self.curves.push(mc.clone());
    }

    /// `Perform()`: joins the appended curves; `join.perform()?`.
    pub fn perform(&mut self) -> Result<(), Failure> {
        let curves: Vec<MultiCurve> = self.curves.clone();
        self.perform_sequence(&curves)
    }

    /// `Perform(TheSeq)`: joins `sequence`; an empty sequence is
    /// Standard_OutOfRange (the native `TheSeq.Value(0)`);
    /// `join.perform_sequence(&curves)?`.
    pub fn perform_sequence(&mut self, sequence: &[MultiCurve]) -> Result<(), Failure> {
        self.spline = if sequence.len() == 1 {
            single_curve(&sequence[0])?
        } else {
            joined_curves(sequence)?
        };
        self.done = true;
        Ok(())
    }

    /// `Value()`: the joined curve; `join.value()`.
    pub fn value(&self) -> &MultiBSpCurve {
        &self.spline
    }

    /// `ChangeValue()`: the joined curve (the native member returns a const
    /// reference too); `join.change_value()`.
    pub fn change_value(&mut self) -> &MultiBSpCurve {
        &self.spline
    }
}

// One curve: the knots [0, 1] with the multiplicities Degree + 1.
fn single_curve(cu: &MultiCurve) -> Result<MultiBSpCurve, Failure> {
    let deg: i32 = cu.degree()?;
    let mut knots: Array1<'static, f64> = scratch(2)?;
    let mut mults: Array1<'static, i32> = scratch(2)?;
    (knots[1], knots[2]) = (0.0, 1.0);
    (mults[1], mults[2]) = (deg + 1, deg + 1);
    Ok(MultiBSpCurve::from_multi_curve(cu, &knots, &mults))
}

fn sequence_curve(sequence: &[MultiCurve], index: i32) -> Result<&MultiCurve, Failure> {
    usize::try_from(index - 1)
        .ok()
        .and_then(|slot| sequence.get(slot))
        .ok_or_else(|| {
            Failure::out_of_range(&format!(
                "NCollection_Sequence::Value: index {index}; expected 1..={}",
                sequence.len()
            ))
        })
}

fn joined_curves(sequence: &[MultiCurve]) -> Result<MultiBSpCurve, Failure> {
    let nbcu: i32 = sequence.len() as i32;
    let last: MultiPoint = sequence_curve(sequence, nbcu)?.value(1)?;
    let (nb3d, nb2d) = (last.nb_points(), last.nb_points2d());
    let (junction, tab_mu) = if nb3d != 0 {
        join_first::<Point3>(sequence, [nb3d, nb2d], 1)?
    } else if nb2d != 0 {
        join_first::<Point2>(sequence, [nb3d, nb2d], 1 + nb3d)?
    } else {
        empty_join()?
    };
    let thefirst: i32 = if nb3d != 0 { 1 } else { 2 };
    copy_other_curves(sequence, &junction, &tab_mu, [nb3d, nb2d], thefirst)?;
    Ok(MultiBSpCurve::from_multi_points(
        &tab_mu,
        &junction.knots,
        &junction.mults,
    ))
}

// Neither 3D nor 2D points: no conversion runs, so the counts and the degree stay 0.
fn empty_join() -> Result<(Junction, Array1<'static, MultiPoint>), Failure> {
    let junction = Junction {
        degree: 0,
        knots: Array1::with_bounds(1, 0).map_err(|e| array_failure(e, "Approx"))?,
        mults: Array1::with_bounds(1, 0).map_err(|e| array_failure(e, "Approx"))?,
    };
    let tab_mu = Array1::with_bounds(1, 0).map_err(|e| array_failure(e, "Approx"))?;
    Ok((junction, tab_mu))
}

/// The dimension-specific members of the join: the MultiCurve and MultiPoint
/// accessors and the Convert class (`conv` or `conv2d`).
trait JoinPoint: PolePoint {
    fn curve(cu: &MultiCurve, index: i32, poles: &mut Array1<'_, Self>) -> Result<(), Failure>;
    fn set(mp: &MultiPoint, index: i32, point: Self) -> Result<(), Failure>;
    fn join(curves: &[Array1<'static, Self>])
    -> Result<(Junction, Array1<'static, Self>), Failure>;
}

macro_rules! impl_join_point {
    ($point:ty, $conv:ty, $curve:ident, $set:ident) => {
        impl JoinPoint for $point {
            fn curve(
                cu: &MultiCurve,
                index: i32,
                poles: &mut Array1<'_, Self>,
            ) -> Result<(), Failure> {
                cu.$curve(index, poles)
            }
            fn set(mp: &MultiPoint, index: i32, point: Self) -> Result<(), Failure> {
                mp.$set(index, point)
            }
            fn join(
                curves: &[Array1<'static, Self>],
            ) -> Result<(Junction, Array1<'static, Self>), Failure> {
                let mut conv: $conv = <$conv>::default();
                curves.iter().for_each(|poles| conv.add_curve(poles));
                conv.perform()?;
                let mut knots: Array1<'static, f64> = scratch(conv.nb_knots())?;
                let mut mults: Array1<'static, i32> = scratch(conv.nb_knots())?;
                let mut poles: Array1<'static, Self> = scratch(conv.nb_poles())?;
                conv.knots_and_mults(&mut knots, &mut mults)?;
                conv.poles(&mut poles)?;
                let junction = Junction {
                    degree: conv.degree(),
                    knots,
                    mults,
                };
                Ok((junction, poles))
            }
        }
    };
}
impl_join_point!(Point3, CompBezierCurvesToBSplineCurve, curve_3d, set_point);
impl_join_point!(
    Point2,
    CompBezierCurves2dToBSplineCurve2d,
    curve_2d,
    set_point2d
);

// Converts curve `index` of every MultiCurve and stores the spline poles in new
// MultiPoints (the other points stay at their default value until copied).
fn join_first<P: JoinPoint>(
    sequence: &[MultiCurve],
    [nb3d, nb2d]: [i32; 2],
    index: i32,
) -> Result<(Junction, Array1<'static, MultiPoint>), Failure> {
    let mut curves: Vec<Array1<'static, P>> = Vec::with_capacity(sequence.len());
    for cu in sequence {
        let mut poles: Array1<'static, P> = scratch(cu.nb_poles())?;
        P::curve(cu, index, &mut poles)?;
        curves.push(poles);
    }
    let (junction, poles) = P::join(&curves)?;
    let mut tab_mu: Array1<'static, MultiPoint> = scratch(poles.length())?;
    for (j, pole) in (1..).zip(poles.iter()) {
        let mp: MultiPoint = MultiPoint::with_counts(nb3d, nb2d)?;
        P::set(&mp, index, *pole)?;
        tab_mu[j] = mp;
    }
    Ok((junction, tab_mu))
}

// `first..=last`: the poles of span `i` the spline keeps (a C1 junction shares one
// pole with the previous span, a C0 one none).
fn span_range(junction: &Junction, i: i32, nbcu: i32) -> Result<[i32; 2], Failure> {
    let deg: i32 = junction.degree;
    let mult = |k: i32| {
        junction
            .mults
            .value(k)
            .copied()
            .map_err(|e| array_failure(e, "Approx_MCurvesToBSpCurve::Perform"))
    };
    let mut last: i32 = if mult(i + 1)? == deg { deg + 1 } else { deg };
    if i == nbcu {
        last = deg + 1;
    }
    let first: i32 = if i != 1 && (mult(i)? == deg - 1 || mult(i)? == deg) {
        2
    } else {
        1
    };
    Ok([first, last])
}

fn copy_other_curves(
    sequence: &[MultiCurve],
    junction: &Junction,
    tab_mu: &Array1<'static, MultiPoint>,
    [nb3d, nb2d]: [i32; 2],
    thefirst: i32,
) -> Result<(), Failure> {
    let nbcu: i32 = sequence.len() as i32;
    // kpol lives across the spans, as natively: without a second 3D curve (or 2D
    // curve) kpoles3d (kpoles2d) takes the value the other loop left.
    let (mut kpol, mut kpoles3d, mut kpoles2d): (i32, i32, i32) = (1, 1, 1);
    for (i, cu) in (1..).zip(sequence) {
        let range: [i32; 2] = span_range(junction, i, nbcu)?;
        for j in 2..=nb3d {
            kpol = copy_span::<Point3>(cu, junction.degree, j, range, tab_mu, kpoles3d)?;
        }
        kpoles3d = kpol;
        for j in thefirst..=nb2d {
            kpol = copy_span::<Point2>(cu, junction.degree, j + nb3d, range, tab_mu, kpoles2d)?;
        }
        kpoles2d = kpol;
    }
    Ok(())
}

// The poles `first..=last` of curve `index` of `cu`, raised to `deg`, into the
// MultiPoints from `kpol`; returns the next free MultiPoint.
fn copy_span<P: JoinPoint>(
    cu: &MultiCurve,
    deg: i32,
    index: i32,
    [first, last]: [i32; 2],
    tab_mu: &Array1<'static, MultiPoint>,
    mut kpol: i32,
) -> Result<i32, Failure> {
    let mut the_poles: Array1<'static, P> = scratch(cu.nb_poles())?;
    P::curve(cu, index, &mut the_poles)?;
    let points: Array1<'static, P> = if deg - cu.degree()? > 0 {
        let mut raised: Array1<'static, P> = scratch(deg + 1)?;
        bsplclib::increase_degree_bezier(deg, &the_poles, None, &mut raised, None)?;
        raised
    } else {
        // `Points = ThePoles`: NCollection_Array1 assignment takes the other bounds.
        the_poles
    };
    for k in first..=last {
        let point: P = *points
            .value(k)
            .map_err(|e| array_failure(e, "Approx_MCurvesToBSpCurve::Perform"))?;
        let mp: &MultiPoint = tab_mu
            .value(kpol)
            .map_err(|e| array_failure(e, "Approx_MCurvesToBSpCurve::Perform"))?;
        P::set(mp, index, point)?;
        kpol += 1;
    }
    Ok(kpol)
}
