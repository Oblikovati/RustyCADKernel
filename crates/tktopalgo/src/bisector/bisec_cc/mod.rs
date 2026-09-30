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
// Rust translation of ModelingAlgorithms/TKTopAlgo/Bisector/Bisector_BisecCC.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Bisector_BisecCC`: the bisector of two curves.
mod curve;
mod perform;
mod proj;
mod state;
mod value;
mod value_by_int;
mod values;

use super::intervals::{cout_real, dump_intervals};
use super::{BisectorCurve, PointOnBis, PolyBis};
use crate::bisector::bisec_pc::lowered_continuity;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dGeometry};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Point2;
use state::{CcState, OnCurves};

/// `Bisector_BisecCC`: the locus of the points equidistant from two curves that
/// meet only at their ends, parameterized by the first curve and sampled by a
/// polygon of points ([`PolyBis`]); it is extended by segments at the common ends
/// of concave curves (docs/topalgo-design.md section 3).
///
/// `FirstParameter` and `LastParameter` read the first and last interval; they
/// raise Standard_NoSuchObject natively when there is none (a default bisector, or
/// one left empty early), where the port answers NaN.
/// ```
/// use rustycad_tktopalgo::bisector::BisecCC;
/// let bisector = BisecCC::new();
/// assert!(bisector.is_empty());
/// ```
pub struct BisecCC {
    parent: StandardTransient,
    state: SharedState<CcState>,
}
impl_transient!(
    BisecCC,
    "Bisector_BisecCC",
    dyn BisectorCurve,
    parent,
    interfaces: [dyn Geom2dGeometry, dyn Geom2dCurve, dyn BisectorCurve]
);

impl Default for BisecCC {
    fn default() -> Self {
        Self::new()
    }
}

/// The values of the bisector point: the point, its parameters on the two curves
/// and its distance to them (`ValueAndDist` and `ValueByInt`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BisecCCPoint {
    /// The point of the bisector.
    pub point: Point2,
    /// The parameter on the first curve.
    pub u1: f64,
    /// The parameter on the second curve.
    pub u2: f64,
    /// The distance (squared except on the extensions).
    pub distance: f64,
}

impl BisecCC {
    /// `Bisector_BisecCC()`: an empty bisector; `BisecCC::new()`.
    pub fn new() -> Self {
        Self::from_state(CcState {
            is_empty: true,
            ..CcState::default()
        })
    }
    fn from_state(state: CcState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// `Bisector_BisecCC(Cu1, Cu2, Side1, Side2, Origin, DistMax)`: [`BisecCC::perform`]
    /// on a new object; `BisecCC::with_curves(&c1, &c2, 1.0, 1.0, origin, 500.0)?`.
    pub fn with_curves(
        cu1: &Handle<dyn Geom2dCurve>,
        cu2: &Handle<dyn Geom2dCurve>,
        side1: f64,
        side2: f64,
        origin: Point2,
        dist_max: f64,
    ) -> Result<Self, Failure> {
        let mut state = CcState::default();
        state.perform((cu1, cu2), (side1, side2), origin, dist_max)?;
        Ok(Self::from_state(state))
    }
    /// `Perform(Cu1, Cu2, Side1, Side2, Origin, DistMax)`: the bisector of copies of
    /// the curves on the sides `Side1`, `Side2` (1 on the left, -1 on the right of
    /// each), from `Origin`. The intervals and the polygon are appended to the
    /// existing ones, as natively; `bisector.perform(&c1, &c2, 1.0, 1.0, origin, 500.0)?`.
    pub fn perform(
        &self,
        cu1: &Handle<dyn Geom2dCurve>,
        cu2: &Handle<dyn Geom2dCurve>,
        side1: f64,
        side2: f64,
        origin: Point2,
        dist_max: f64,
    ) -> Result<(), Failure> {
        // The fields keep what a failing native Perform left in them.
        let mut state: CcState = self.state.snapshot();
        let result = state.perform((cu1, cu2), (side1, side2), origin, dist_max);
        *self.state.write() = state;
        result
    }
    /// `IsEmpty()`; `bisector.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.state.read().is_empty
    }
    /// `ValueAndDist(U, U1, U2, Distance)`: the point of parameter `u` by the
    /// iterative method; a path that does not write a parameter leaves it 0 (a
    /// native uninitialised variable); `bisector.value_and_dist(0.5)?.point`.
    pub fn value_and_dist(&self, u: f64) -> Result<BisecCCPoint, Failure> {
        let mut on_curves = OnCurves::default();
        let point: Point2 = self.state.read().value_and_dist(u, &mut on_curves)?;
        Ok(point_with(point, on_curves))
    }
    /// `ValueByInt(U, U1, U2, Distance)`: the point of parameter `u` by root finding;
    /// `bisector.value_by_int(0.5)?.distance`.
    pub fn value_by_int(&self, u: f64) -> Result<BisecCCPoint, Failure> {
        let mut on_curves = OnCurves::default();
        let point: Point2 = self.state.read().value_by_int(u, &mut on_curves)?;
        Ok(point_with(point, on_curves))
    }
    /// `LinkBisCurve(U)`: the parameter on the first curve; `bisector.link_bis_curve(0.5)`.
    pub fn link_bis_curve(&self, u: f64) -> f64 {
        self.state.read().link_bis_curve(u)
    }
    /// `LinkCurveBis(U)`: the reverse of [`BisecCC::link_bis_curve`];
    /// `bisector.link_curve_bis(0.5)`.
    pub fn link_curve_bis(&self, u: f64) -> f64 {
        self.state.read().link_curve_bis(u)
    }
    /// `Curve(IndCurve)`: the (copied) curve 1 or 2; Standard_OutOfRange otherwise;
    /// `bisector.curve(1)?`.
    pub fn curve(&self, index: i32) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        self.state.read().curve(index)
    }
    /// `Polygon()`: the polygon that samples the bisector; `bisector.polygon()`.
    pub fn polygon(&self) -> PolyBis {
        self.state.read().my_polygon
    }
    /// `IntervalContinuity()`: the continuity of the first curve lowered by one;
    /// `bisector.interval_continuity()`.
    pub fn interval_continuity(&self) -> Shape {
        lowered_continuity(self.state.read().curve1.continuity())
    }
    /// `ChangeGuide()`: the same bisector parameterized by the second curve, on the
    /// part of the polygon only (the curves are shared, not copied; the result is
    /// marked empty as a default bisector is); `bisector.change_guide()`.
    pub fn change_guide(&self) -> Result<Handle<BisecCC>, Failure> {
        let state = self.state.read();
        let mut guide = CcState {
            curve1: state.curve2.clone(),
            curve2: state.curve1.clone(),
            sign1: state.sign2,
            sign2: state.sign1,
            is_convex1: state.is_convex2,
            is_convex2: state.is_convex1,
            is_empty: true,
            ..CcState::default()
        };
        guide.my_polygon = swapped_polygon(&state.my_polygon, state.sign1 == state.sign2)?;
        guide
            .start_intervals
            .append(guide.my_polygon.first().param_on_bis());
        guide
            .end_intervals
            .append(guide.my_polygon.last().param_on_bis());
        Ok(Handle::new(Self::from_state(guide)))
    }
    /// `Dump(Deep, Offset)`: the sides and the intervals as `std::cout` gets them;
    /// `bisector.dump(0, 0, &mut text)?`.
    pub fn dump(&self, _deep: i32, offset: i32, out: &mut String) -> Result<(), Failure> {
        let state = self.state.read();
        let indent: String = " ".repeat(usize::try_from(offset).unwrap_or(0));
        out.push_str(&format!("{indent}Bisector_BisecCC :\n{indent}"));
        out.push_str(&format!(
            "Sign1  :{}\nSign2  :{}\n",
            cout_real(state.sign1),
            cout_real(state.sign2)
        ));
        dump_intervals(
            &state.start_intervals,
            &state.end_intervals,
            state.current_interval,
            out,
        )
    }
}

// The public form of the in-out outputs.
fn point_with(point: Point2, on_curves: OnCurves) -> BisecCCPoint {
    BisecCCPoint {
        point,
        u1: on_curves.u1,
        u2: on_curves.u2,
        distance: on_curves.dist,
    }
}

// The polygon of ChangeGuide: the parameters on the curves exchanged, the new
// parameter on the bisector is the one on the old second curve, in the reverse
// order when the sides are equal.
fn swapped_polygon(polygon: &PolyBis, reverse: bool) -> Result<PolyBis, Failure> {
    let mut poly = PolyBis::new();
    let count: i32 = polygon.length();
    for k in 1..=count {
        let index: i32 = if reverse { count + 1 - k } else { k };
        let p: PointOnBis = polygon.value(index)?;
        poly.append(PointOnBis::new(
            p.param_on_c2(),
            p.param_on_c1(),
            p.param_on_c2(),
            p.distance(),
            p.point(),
        ))?;
    }
    Ok(poly)
}

impl BisectorCurve for BisecCC {
    fn parameter(&self, p: Point2) -> Result<f64, Failure> {
        let first: f64 = self.first_parameter();
        if p.is_equal(self.value(first)?, precision::confusion()) {
            return Ok(first);
        }
        let last: f64 = self.last_parameter();
        if p.is_equal(self.value(last)?, precision::confusion()) {
            return Ok(last);
        }
        let curve1: Handle<dyn Geom2dCurve> = self.state.read().curve1.clone();
        Ok(proj::proj_on_curve(p, &curve1)?.unwrap_or(0.))
    }
    fn is_extend_at_start(&self) -> bool {
        self.state.read().extension_start
    }
    fn is_extend_at_end(&self) -> bool {
        self.state.read().extension_end
    }
    fn nb_intervals(&self) -> i32 {
        self.state.read().start_intervals.len()
    }
    fn interval_first(&self, index: i32) -> Result<f64, Failure> {
        self.state.read().start_intervals.value(index)
    }
    fn interval_last(&self, index: i32) -> Result<f64, Failure> {
        self.state.read().end_intervals.value(index)
    }
}
