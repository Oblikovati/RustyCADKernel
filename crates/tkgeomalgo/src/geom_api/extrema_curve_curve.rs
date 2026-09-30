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
// Rust translation of GeomAPI_ExtremaCurveCurve.hxx/.lxx/.cxx (the constructors,
// Init and the consultation) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomAPI_ExtremaCurveCurve`: the extrema between two 3D curves.
use super::{check_done, check_index};
use crate::geom2d_api::extrema_results::ExtremaResults;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeombase::extrema::{ExtCC, POnCurv};
use rustycad_tkmath::gp::Point3;

/// The tolerances of the default `Extrema_ExtCC()` member (its native defaults).
const DEFAULT_TOL: f64 = 1.0e-10;

/// The arguments of one Init: the curves and, for the second overload, the ranges.
#[derive(Clone)]
struct CurveCurveInit {
    c1: Handle<dyn GeomCurve>,
    c2: Handle<dyn GeomCurve>,
    ranges: Option<((f64, f64), (f64, f64))>,
}

/// What the members read from the Extrema_ExtCC after the last Init.
pub(super) struct CurveCurveResults {
    pub(super) extrema: ExtremaResults<(POnCurv, POnCurv)>,
    pub(super) is_parallel: Result<bool, Failure>,
    pub(super) trimmed: ([f64; 4], [Point3; 4]),
}

/// `GeomAPI_ExtremaCurveCurve`: the extrema of Extrema_ExtCC between two curves,
/// with the index of the nearest one, and the minimal distance between the trimmed
/// curves (the `Total*` members, which add the ends of the curves).
///
/// Natively the object keeps one Extrema_ExtCC from Init to Init, and Extrema_ExtCC
/// keeps its Extrema_ECC and its end points across Performs (the general case appends
/// to the solutions of an earlier Init). Extrema_ExtCC borrows the adaptors this object
/// owns, so the object records its Inits and replays them on one Extrema_ExtCC over its
/// adaptors (reloaded by each Init as natively), which rebuilds the native state.
///
/// ```
/// use rustycad_tkgeomalgo::geom_api::ExtremaCurveCurve;
/// use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Direction3, Point3};
/// let l1: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(
///     Point3::new(0.0, 0.0, 0.0), Direction3::new(1.0, 0.0, 0.0).unwrap())).cast();
/// let l2: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(
///     Point3::new(0.0, 0.0, 1.0), Direction3::new(0.0, 1.0, 0.0).unwrap())).cast();
/// let e = ExtremaCurveCurve::with_curves(&l1, &l2).unwrap();
/// assert_eq!(e.lower_distance().unwrap(), 1.0);
/// ```
pub struct ExtremaCurveCurve {
    pub(super) my_is_done: bool,
    pub(super) my_index: i32,
    pub(super) my_ext_cc: CurveCurveResults,
    pub(super) my_c1: GeomAdaptorCurve,
    pub(super) my_c2: GeomAdaptorCurve,
    pub(super) my_total_ext: bool,
    pub(super) my_is_infinite: bool,
    pub(super) my_total_dist: f64,
    pub(super) my_total_points: [Point3; 2],
    pub(super) my_total_pars: [f64; 2],
    inits: Vec<CurveCurveInit>,
}

impl Default for ExtremaCurveCurve {
    fn default() -> Self {
        let ext_cc: ExtCC<'_> = ExtCC::new(DEFAULT_TOL, DEFAULT_TOL);
        Self {
            my_is_done: false,
            my_index: 0,
            my_ext_cc: results_of(&ext_cc),
            my_c1: GeomAdaptorCurve::default(),
            my_c2: GeomAdaptorCurve::default(),
            my_total_ext: false,
            my_is_infinite: false,
            my_total_dist: 0.0,
            my_total_points: [Point3::default(); 2],
            my_total_pars: [0.0; 2],
            inits: Vec::new(),
        }
    }
}

impl ExtremaCurveCurve {
    /// `GeomAPI_ExtremaCurveCurve()`: no extrema. `ExtremaCurveCurve::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `GeomAPI_ExtremaCurveCurve(C1, C2)`: on the whole curves.
    /// `ExtremaCurveCurve::with_curves(&c1, &c2)?`.
    pub fn with_curves(
        c1: &Handle<dyn GeomCurve>,
        c2: &Handle<dyn GeomCurve>,
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.init(c1, c2)?;
        Ok(extrema)
    }
    /// `GeomAPI_ExtremaCurveCurve(C1, C2, U1min, U1max, U2min, U2max)`.
    /// `ExtremaCurveCurve::with_ranges(&c1, &c2, (0.0, 1.0), (0.0, 1.0))?`.
    pub fn with_ranges(
        c1: &Handle<dyn GeomCurve>,
        c2: &Handle<dyn GeomCurve>,
        u1: (f64, f64),
        u2: (f64, f64),
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.init_ranges(c1, c2, u1, u2)?;
        Ok(extrema)
    }
    /// `Init(C1, C2)`: Extrema_ExtCC on the whole curves with the tolerance
    /// Precision::PConfusion(). `extrema.init(&c1, &c2)?`.
    pub fn init(
        &mut self,
        c1: &Handle<dyn GeomCurve>,
        c2: &Handle<dyn GeomCurve>,
    ) -> Result<(), Failure> {
        self.record_init(CurveCurveInit {
            c1: c1.clone(),
            c2: c2.clone(),
            ranges: None,
        })
    }
    /// `Init(C1, C2, U1min, U1max, U2min, U2max)`: Extrema_ExtCC on the ranges; the
    /// adaptors still hold the whole curves, which the `Total*` members read, as
    /// natively. `extrema.init_ranges(&c1, &c2, (0.0, 1.0), (0.0, 1.0))?`.
    pub fn init_ranges(
        &mut self,
        c1: &Handle<dyn GeomCurve>,
        c2: &Handle<dyn GeomCurve>,
        u1: (f64, f64),
        u2: (f64, f64),
    ) -> Result<(), Failure> {
        self.record_init(CurveCurveInit {
            c1: c1.clone(),
            c2: c2.clone(),
            ranges: Some((u1, u2)),
        })
    }

    // One native Init: the replayed Extrema_ExtCC, then the nearest extremum.
    fn record_init(&mut self, init: CurveCurveInit) -> Result<(), Failure> {
        self.my_total_ext = false;
        self.inits.push(init);
        let results: CurveCurveResults = {
            let ext_cc: ExtCC<'_> = self.replay()?;
            results_of(&ext_cc)
        };
        self.my_ext_cc = results;
        let extrema = &self.my_ext_cc.extrema;
        self.my_is_done = extrema.done && extrema.nb_ext()? > 0;
        if self.my_is_done {
            self.my_index = extrema.nearest()?;
        }
        Ok(())
    }
    // Every Init on one Extrema_ExtCC over the adaptors, as the native member lives
    // through them; an earlier Init that raised stops where it raised natively, and
    // the exception of the last one is returned.
    fn replay(&self) -> Result<ExtCC<'_>, Failure> {
        let mut ext_cc: ExtCC<'_> = ExtCC::new(DEFAULT_TOL, DEFAULT_TOL);
        let Some((last, earlier)) = self.inits.split_last() else {
            return Ok(ext_cc);
        };
        for init in earlier {
            // The exception of an earlier Init already reached its caller.
            let _ = self.run_init(&mut ext_cc, init);
        }
        self.run_init(&mut ext_cc, last)?;
        Ok(ext_cc)
    }
    // `myC1.Load(C1); myC2.Load(C2); myExtCC.Initialize(...); myExtCC.Perform();`.
    fn run_init<'s>(
        &'s self,
        ext_cc: &mut ExtCC<'s>,
        init: &CurveCurveInit,
    ) -> Result<(), Failure> {
        let tol: f64 = precision::p_confusion();
        self.my_c1.load(&init.c1)?;
        self.my_c2.load(&init.c2)?;
        match init.ranges {
            None => ext_cc.initialize(&self.my_c1, &self.my_c2, tol, tol)?,
            Some((u1, u2)) => {
                ext_cc.initialize_ranges(&self.my_c1, &self.my_c2, u1, u2, (tol, tol))?
            }
        }
        ext_cc.perform()
    }
}

impl ExtremaCurveCurve {
    /// `NbExtrema()`: 0 when the computation failed. `extrema.nb_extrema()`.
    pub fn nb_extrema(&self) -> i32 {
        if self.my_is_done {
            self.my_ext_cc.extrema.nb
        } else {
            0
        }
    }
    /// `operator int()`: Extrema_ExtCC::NbExt (StdFail_NotDone when Extrema failed).
    /// `extrema.as_int()?`.
    pub fn as_int(&self) -> Result<i32, Failure> {
        self.my_ext_cc.extrema.nb_ext()
    }
    /// `Points(Index, P1, P2)`: Standard_OutOfRange outside `1..=NbExtrema()`.
    /// `let (p1, p2) = extrema.points(1)?;`
    pub fn points(&self, index: i32) -> Result<(Point3, Point3), Failure> {
        let (pc1, pc2) = self.pair(index, "GeomAPI_ExtremaCurveCurve::Points")?;
        Ok((*pc1.value(), *pc2.value()))
    }
    /// `Parameters(Index, U1, U2)`: Standard_OutOfRange outside `1..=NbExtrema()`.
    /// `let (u1, u2) = extrema.parameters(1)?;`
    pub fn parameters(&self, index: i32) -> Result<(f64, f64), Failure> {
        let (pc1, pc2) = self.pair(index, "GeomAPI_ExtremaCurveCurve::Parameters")?;
        Ok((pc1.parameter(), pc2.parameter()))
    }
    /// `Distance(Index)`: Standard_OutOfRange outside `1..=NbExtrema()`.
    /// `extrema.distance(1)?`.
    pub fn distance(&self, index: i32) -> Result<f64, Failure> {
        check_index(
            index,
            self.nb_extrema(),
            "GeomAPI_ExtremaCurveCurve::Distance",
        )?;
        Ok(self.my_ext_cc.extrema.square_distance(index)?.sqrt())
    }
    /// `IsParallel()`: Extrema_ExtCC::IsParallel (StdFail_NotDone when Extrema failed).
    /// `extrema.is_parallel()?`.
    pub fn is_parallel(&self) -> Result<bool, Failure> {
        self.my_ext_cc.is_parallel.clone()
    }
    /// `NearestPoints(P1, P2)`: StdFail_NotDone when the computation failed.
    /// `let (p1, p2) = extrema.nearest_points()?;`
    pub fn nearest_points(&self) -> Result<(Point3, Point3), Failure> {
        check_done(self.my_is_done, "GeomAPI_ExtremaCurveCurve::NearestPoints")?;
        self.points(self.my_index)
    }
    /// `LowerDistanceParameters(U1, U2)`: StdFail_NotDone when the computation
    /// failed. `let (u1, u2) = extrema.lower_distance_parameters()?;`
    pub fn lower_distance_parameters(&self) -> Result<(f64, f64), Failure> {
        check_done(
            self.my_is_done,
            "GeomAPI_ExtremaCurveCurve::LowerDistanceParameters",
        )?;
        self.parameters(self.my_index)
    }
    /// `LowerDistance()`: StdFail_NotDone when the computation failed.
    /// `extrema.lower_distance()?`.
    pub fn lower_distance(&self) -> Result<f64, Failure> {
        check_done(self.my_is_done, "GeomAPI_ExtremaCurveCurve::LowerDistance")?;
        Ok(self
            .my_ext_cc
            .extrema
            .square_distance(self.my_index)?
            .sqrt())
    }
    /// `operator double()`: LowerDistance(). `extrema.as_real()?`.
    pub fn as_real(&self) -> Result<f64, Failure> {
        self.lower_distance()
    }
    /// `Extrema()`: the Extrema_ExtCC of the adaptors, rebuilt by replaying the Inits
    /// (a default one before any Init). `extrema.extrema()?.is_done()`.
    pub fn extrema(&self) -> Result<ExtCC<'_>, Failure> {
        self.replay()
    }

    pub(super) fn pair(&self, index: i32, member: &str) -> Result<(POnCurv, POnCurv), Failure> {
        check_index(index, self.nb_extrema(), member)?;
        self.my_ext_cc.extrema.item(index)
    }
}

// The results, the parallel flag and the trimmed distances of an Extrema_ExtCC.
fn results_of(ext: &ExtCC<'_>) -> CurveCurveResults {
    CurveCurveResults {
        extrema: snapshot(ext),
        is_parallel: ext.is_parallel(),
        trimmed: ext.trimmed_square_distances(),
    }
}

// The extrema of an Extrema_ExtCC, with the exceptions of its accessors.
fn snapshot(ext: &ExtCC<'_>) -> ExtremaResults<(POnCurv, POnCurv)> {
    let Ok(nb) = ext.nb_ext() else {
        return ExtremaResults::not_done("Extrema_ExtCC");
    };
    let mut results = ExtremaResults::done("Extrema_ExtCC", nb);
    for n in 1..=nb {
        results.push(ext.points(n), ext.square_distance(n));
    }
    results
}
