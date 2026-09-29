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
// Rust translation of Geom2dAPI_ExtremaCurveCurve.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dAPI_ExtremaCurveCurve`: the extrema between two trimmed 2D curves.
use super::extrema_results::ExtremaResults;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeombase::extrema::{ExtCC2d, POnCurv2d};
use rustycad_tkmath::gp::Point2;

/// The native default tolerances of `Extrema_ExtCC2d(C1, C2)`.
const TOL_C: f64 = 1.0e-10;

/// `Geom2dAPI_ExtremaCurveCurve`: the extrema of Extrema_ExtCC2d between the curves
/// on the given ranges, with the index of the nearest one. It fails (0 extrema) when
/// Extrema fails or finds none.
///
/// Extrema_ExtCC2d keeps references to its curves, which this object owns, so the
/// object keeps the results of the construction and [`ExtremaCurveCurve::extrema`]
/// rebuilds the (deterministic) Extrema_ExtCC2d on its own adaptors.
///
/// ```
/// use rustycad_tkgeomalgo::geom2d_api::ExtremaCurveCurve;
/// use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Direction2, Point2};
/// let l1: Handle<dyn Geom2dCurve> =
///     Handle::new(Geom2dLine::from_point(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0).unwrap())).cast();
/// let l2: Handle<dyn Geom2dCurve> =
///     Handle::new(Geom2dLine::from_point(Point2::new(0.0, 1.0), Direction2::new(1.0, 1.0).unwrap())).cast();
/// let e = ExtremaCurveCurve::new(&l1, &l2, -5.0, 5.0, -5.0, 5.0).unwrap();
/// assert_eq!(e.nb_extrema(), 1);
/// ```
pub struct ExtremaCurveCurve {
    my_is_done: bool,
    my_index: i32,
    my_ext_cc: ExtremaResults<(POnCurv2d, POnCurv2d)>,
    my_c1: Geom2dAdaptorCurve,
    my_c2: Geom2dAdaptorCurve,
}

impl ExtremaCurveCurve {
    /// `Geom2dAPI_ExtremaCurveCurve(C1, C2, U1min, U1max, U2min, U2max)`: the
    /// adaptor exceptions (null curve, reversed range) are returned as `Err`.
    /// `ExtremaCurveCurve::new(&c1, &c2, 0.0, 1.0, 0.0, 1.0)?`.
    pub fn new(
        c1: &Handle<dyn Geom2dCurve>,
        c2: &Handle<dyn Geom2dCurve>,
        u1min: f64,
        u1max: f64,
        u2min: f64,
        u2max: f64,
    ) -> Result<Self, Failure> {
        let my_c1 = Geom2dAdaptorCurve::with_range(c1, u1min, u1max)?;
        let my_c2 = Geom2dAdaptorCurve::with_range(c2, u2min, u2max)?;
        let my_ext_cc = snapshot(&ExtCC2d::with_curves(&my_c1, &my_c2, TOL_C, TOL_C)?)?;
        let my_is_done: bool = my_ext_cc.done && my_ext_cc.nb_ext()? > 0;
        let my_index: i32 = if my_is_done { my_ext_cc.nearest()? } else { 0 };
        Ok(Self {
            my_is_done,
            my_index,
            my_ext_cc,
            my_c1,
            my_c2,
        })
    }
    /// `NbExtrema()`: the number of extrema, 0 when the computation failed.
    /// `extrema.nb_extrema()`.
    pub fn nb_extrema(&self) -> i32 {
        if self.my_is_done {
            self.my_ext_cc.nb
        } else {
            0
        }
    }
    /// `operator int()`: Extrema_ExtCC2d::NbExt (StdFail_NotDone when Extrema
    /// failed). `extrema.as_int()?`.
    pub fn as_int(&self) -> Result<i32, Failure> {
        self.my_ext_cc.nb_ext()
    }
    /// `Points(Index, P1, P2)`: the ends of the extremum; Standard_OutOfRange outside
    /// `1..=NbExtrema()`. `let (p1, p2) = extrema.points(1)?;`
    pub fn points(&self, index: i32) -> Result<(Point2, Point2), Failure> {
        let (pc1, pc2) = self.pair(index, "Geom2dAPI_ExtremaCurveCurve::Points")?;
        Ok((*pc1.value(), *pc2.value()))
    }
    /// `Parameters(Index, U1, U2)`: the parameters of the ends of the extremum;
    /// Standard_OutOfRange outside `1..=NbExtrema()`. `let (u1, u2) = extrema.parameters(1)?;`
    pub fn parameters(&self, index: i32) -> Result<(f64, f64), Failure> {
        let (pc1, pc2) = self.pair(index, "Geom2dAPI_ExtremaCurveCurve::Parameters")?;
        Ok((pc1.parameter(), pc2.parameter()))
    }
    /// `Distance(Index)`: the length of the extremum; Standard_OutOfRange outside
    /// `1..=NbExtrema()`. `extrema.distance(1)?`.
    pub fn distance(&self, index: i32) -> Result<f64, Failure> {
        super::check_index(
            index,
            self.nb_extrema(),
            "Geom2dAPI_ExtremaCurveCurve:Distance",
        )?;
        Ok(self.my_ext_cc.square_distance(index)?.sqrt())
    }
    /// `NearestPoints(P1, P2)`: the ends of the shortest extremum; StdFail_NotDone
    /// when the computation failed. `let (p1, p2) = extrema.nearest_points()?;`
    pub fn nearest_points(&self) -> Result<(Point2, Point2), Failure> {
        self.check_done("Geom2dAPI_ExtremaCurveCurve:NearestPoints")?;
        self.points(self.my_index)
    }
    /// `LowerDistanceParameters(U1, U2)`; StdFail_NotDone when the computation
    /// failed. `let (u1, u2) = extrema.lower_distance_parameters()?;`
    pub fn lower_distance_parameters(&self) -> Result<(f64, f64), Failure> {
        self.check_done("Geom2dAPI_ExtremaCurveCurve:LowerDistanceParameters")?;
        self.parameters(self.my_index)
    }
    /// `LowerDistance()`; StdFail_NotDone when the computation failed.
    /// `extrema.lower_distance()?`.
    pub fn lower_distance(&self) -> Result<f64, Failure> {
        self.check_done("Geom2dAPI_ExtremaCurveCurve:LowerDistance")?;
        Ok(self.my_ext_cc.square_distance(self.my_index)?.sqrt())
    }
    /// `operator double()`: LowerDistance(). `extrema.as_real()?`.
    pub fn as_real(&self) -> Result<f64, Failure> {
        self.lower_distance()
    }
    /// `Extrema()`: the Extrema_ExtCC2d of the two adaptors (rebuilt on them; its
    /// results are those of the construction). `extrema.extrema()?.nb_ext()?`.
    pub fn extrema(&self) -> Result<ExtCC2d<'_>, Failure> {
        ExtCC2d::with_curves(&self.my_c1, &self.my_c2, TOL_C, TOL_C)
    }

    fn pair(&self, index: i32, member: &str) -> Result<(POnCurv2d, POnCurv2d), Failure> {
        super::check_index(index, self.nb_extrema(), member)?;
        self.my_ext_cc.item(index)
    }
    fn check_done(&self, member: &str) -> Result<(), Failure> {
        if !self.my_is_done {
            return Err(super::not_done(member));
        }
        Ok(())
    }
}

// The results of an Extrema_ExtCC2d, with its exceptions.
fn snapshot(ext: &ExtCC2d<'_>) -> Result<ExtremaResults<(POnCurv2d, POnCurv2d)>, Failure> {
    if !ext.is_done() {
        return Ok(ExtremaResults::not_done("Extrema_ExtCC2d"));
    }
    let nb: i32 = ext.nb_ext()?;
    let mut results = ExtremaResults::done("Extrema_ExtCC2d", nb);
    for n in 1..=nb {
        results.push(ext.points(n), ext.square_distance(n));
    }
    Ok(results)
}
