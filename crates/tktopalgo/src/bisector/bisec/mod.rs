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
// Rust translation of ModelingAlgorithms/TKTopAlgo/Bisector/Bisector_Bisec.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Bisector_Bisec`: the bisector of two elements, chosen among the analytic and
//! the algorithmic ones and trimmed by the point of passage.
mod curves;
mod points;

use super::{BisecAna, BisectorCurve};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkmath::gp::{Direction2, Point2};

/// `Bisector_Bisec`: the bisector of two curves, a curve and a point, or two
/// points, as a trimmed curve whose basis is a [`BisectorCurve`] (the analytic
/// [`BisecAna`], or a [`super::BisecCC`] or [`super::BisecPC`]). It starts at the
/// point of passage `P` and lies in the sector of `-V1` and `-V2` (indirect when
/// `Sense` is true, direct otherwise); conics whose size is below the tolerance are
/// replaced by lines. A copy shares the trimmed curve, as the native value class
/// shares its handle.
/// ```
/// use rustycad_tktopalgo::bisector::Bisec;
/// use rustycad_tkg2d::geom2d::{Geom2dCartesianPoint, Geom2dCurve, Geom2dPoint};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Point2, Vector2};
/// let p1: Handle<dyn Geom2dPoint> = Handle::new(Geom2dCartesianPoint::new(0.0, 0.0)).cast();
/// let p2: Handle<dyn Geom2dPoint> = Handle::new(Geom2dCartesianPoint::new(2.0, 0.0)).cast();
/// let mut bisec = Bisec::new();
/// bisec.perform_points(&p1, &p2, Point2::new(1.0, 0.0), Vector2::new(0.0, 1.0),
///     Vector2::new(0.0, -1.0), 1.0, 0.0, true)?;
/// assert_eq!(bisec.value().first_parameter(), 0.0);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Default)]
pub struct Bisec {
    thebisector: Handle<Geom2dTrimmedCurve>,
}

impl Bisec {
    /// `Bisector_Bisec()`: no bisector yet; `Bisec::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Value()`: the trimmed bisector; `bisec.value()`.
    pub fn value(&self) -> Handle<Geom2dTrimmedCurve> {
        self.thebisector.clone()
    }
    /// `ChangeValue()`: the trimmed bisector, whose trim the caller may change;
    /// `bisec.change_value().set_trim(0.0, 1.0, true, true)?`.
    pub fn change_value(&mut self) -> Handle<Geom2dTrimmedCurve> {
        self.thebisector.clone()
    }
    // `thebisector = new Geom2d_TrimmedCurve(Bis, UFirst, ULast)`: a copy of `bis`.
    fn trim(
        &mut self,
        bis: &Handle<dyn BisectorCurve>,
        u_first: f64,
        u_last: f64,
    ) -> Result<(), Failure> {
        let basis: Handle<dyn Geom2dCurve> = bis.cast();
        self.thebisector = Handle::new(Geom2dTrimmedCurve::new(
            &basis, u_first, u_last, true, true,
        )?);
        Ok(())
    }
}

/// A half line of `BisecAna` from `apoint` along `n`, trimmed to `[0, last]`: the
/// straight bisector that replaces a degenerate or missing algorithmic one. It
/// returns the bisector and its start and end parameters.
pub(crate) fn half_line(
    apoint: Point2,
    n: Direction2,
    last: f64,
) -> Result<(Handle<dyn BisectorCurve>, f64, f64), Failure> {
    let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(apoint, n)).cast();
    let bis_l: Handle<Geom2dTrimmedCurve> =
        Handle::new(Geom2dTrimmedCurve::new(&line, 0., last, true, true)?);
    let bis_ana = BisecAna::new();
    bis_ana.init(&bis_l);
    let (u_first, u_last) = (
        bis_ana.parameter_of_start_point(),
        bis_ana.parameter_of_end_point(),
    );
    Ok((Handle::new(bis_ana).cast(), u_first, u_last))
}

/// A half line from `apoint` to infinity: [`half_line`] with the native
/// `Precision::Infinite()` end.
pub(crate) fn infinite_half_line(
    apoint: Point2,
    n: Direction2,
) -> Result<(Handle<dyn BisectorCurve>, f64, f64), Failure> {
    half_line(apoint, n, precision::infinite())
}
