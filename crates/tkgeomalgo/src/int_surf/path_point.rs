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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntSurf/IntSurf_PathPoint.hxx/.lxx/.cxx
// and IntSurf_PathPointTool.hxx/.lxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntSurf_PathPoint` (a start or stop point of a marching on a domain boundary) and
//! its walking tool.
use rustycad_tkernel::ncollection::HSequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Direction2, Point3, Vector3, Xy};

/// A point of a path on a restriction arc, with its principal parameters and the
/// auxiliary parameters of its multiplicity (`IntSurf_PathPoint`).
///
/// The parameters live in a shared `NCollection_HSequence<gp_XY>`: a copy shares it, as
/// the native copy of the handle does, so `AddUV` on a copy is seen by the original.
/// `let p = IntSurfPathPoint::with_value(Point3::new(0.0, 0.0, 0.0), 0.5, 0.5); p.multiplicity()? == 0`.
#[derive(Clone)]
pub struct IntSurfPathPoint {
    pt: Point3,
    ispass: bool,
    istgt: bool,
    vectg: Vector3,
    dirtg: Direction2,
    sequv: Handle<HSequence<Xy>>,
}

impl Default for IntSurfPathPoint {
    fn default() -> Self {
        Self::new()
    }
}

/// The error of a member that reads the parameters of a default-constructed point,
/// whose native null handle would be dereferenced.
fn null_parameters(member: &str) -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        &format!("IntSurf_PathPoint::{member}: the point has no parameters (default-constructed)"),
    )
}

/// The StdFail_UndefinedDerivative of the tangent accessors of a tangent point.
fn tangent_point(member: &str) -> Failure {
    Failure::with_message(
        "StdFail_UndefinedDerivative",
        &format!("IntSurf_PathPoint::{member}: the surfaces are tangent at the point"),
    )
}

fn parameter_list(u: f64, v: f64) -> Handle<HSequence<Xy>> {
    let sequv: Handle<HSequence<Xy>> = Handle::new(HSequence::default());
    if let Some(sequence) = sequv.get() {
        sequence.append(Xy::new(u, v));
    }
    sequv
}

impl IntSurfPathPoint {
    /// A passing, tangent point at the origin without parameters (`IntSurf_PathPoint()`).
    /// `IntSurfPathPoint::new().is_passing_pnt()`.
    pub fn new() -> Self {
        Self {
            pt: Point3::new(0.0, 0.0, 0.0),
            ispass: true,
            istgt: true,
            vectg: Vector3::new(0.0, 0.0, 0.0),
            dirtg: Direction2::default(),
            sequv: Handle::default(),
        }
    }

    /// A passing, tangent point with its principal parameters (`IntSurf_PathPoint(P, U, V)`).
    /// `IntSurfPathPoint::with_value(p, 0.5, 0.5)`.
    pub fn with_value(p: Point3, u: f64, v: f64) -> Self {
        let mut point = Self::new();
        point.pt = p;
        point.sequv = parameter_list(u, v);
        point
    }

    /// Replaces the point and starts a new parameter list (`SetValue`); the flags and
    /// tangents keep their values. `p.set_value(p3, 0.5, 0.5)`.
    pub fn set_value(&mut self, p: Point3, u: f64, v: f64) {
        self.pt = p;
        self.sequv = parameter_list(u, v);
    }

    /// Appends auxiliary parameters (`AddUV`); Standard_NullObject on a
    /// default-constructed point. `p.add_uv(1.0, 0.5)?`.
    pub fn add_uv(&self, u: f64, v: f64) -> Result<(), Failure> {
        let sequence = self.sequv.get().ok_or_else(|| null_parameters("AddUV"))?;
        sequence.append(Xy::new(u, v));
        Ok(())
    }

    /// Sets the tangents and marks the point not tangent (`SetDirections`).
    /// `p.set_directions(tangent, direction2d)`.
    pub fn set_directions(&mut self, v: Vector3, d: Direction2) {
        self.istgt = false;
        self.vectg = v;
        self.dirtg = d;
    }

    /// Sets the tangency flag (`SetTangency`); `p.set_tangency(true)`.
    pub fn set_tangency(&mut self, tang: bool) {
        self.istgt = tang;
    }

    /// Sets the passing flag (`SetPassing`); `p.set_passing(false)`.
    pub fn set_passing(&mut self, pass: bool) {
        self.ispass = pass;
    }

    /// The 3d point (`Value`); `p.value()`.
    pub const fn value(&self) -> Point3 {
        self.pt
    }

    /// The principal parameters (`Value2d`); Standard_NullObject on a
    /// default-constructed point. `p.value2d()?`.
    pub fn value2d(&self) -> Result<(f64, f64), Failure> {
        let sequence = self.sequv.get().ok_or_else(|| null_parameters("Value2d"))?;
        let uv: Option<Xy> = sequence.sequence(|list| list.first().copied());
        let uv: Xy = uv.ok_or_else(|| null_parameters("Value2d"))?;
        Ok((uv.x(), uv.y()))
    }

    /// Whether the line passes through the point (`IsPassingPnt`); `p.is_passing_pnt()`.
    pub const fn is_passing_pnt(&self) -> bool {
        self.ispass
    }

    /// Whether the surfaces are tangent at the point (`IsTangent`); `p.is_tangent()`.
    pub const fn is_tangent(&self) -> bool {
        self.istgt
    }

    /// The 3d tangent (`Direction3d`); StdFail_UndefinedDerivative at a tangent point.
    /// `p.direction3d()?`.
    pub fn direction3d(&self) -> Result<Vector3, Failure> {
        if self.istgt {
            return Err(tangent_point("Direction3d"));
        }
        Ok(self.vectg)
    }

    /// The parametric tangent (`Direction2d`); StdFail_UndefinedDerivative at a tangent
    /// point. `p.direction2d()?`.
    pub fn direction2d(&self) -> Result<Direction2, Failure> {
        if self.istgt {
            return Err(tangent_point("Direction2d"));
        }
        Ok(self.dirtg)
    }

    /// The number of auxiliary parameter pairs (`Multiplicity`); Standard_NullObject on
    /// a default-constructed point. `p.multiplicity()?`.
    pub fn multiplicity(&self) -> Result<i32, Failure> {
        let sequence = self
            .sequv
            .get()
            .ok_or_else(|| null_parameters("Multiplicity"))?;
        Ok(sequence.sequence(|list| list.length()) - 1)
    }

    /// The auxiliary parameters of rank `index` (`Parameters`, the list entry
    /// `index + 1`); Standard_OutOfRange outside `0..=Multiplicity`. `p.parameters(1)?`.
    pub fn parameters(&self, index: i32) -> Result<(f64, f64), Failure> {
        let sequence = self
            .sequv
            .get()
            .ok_or_else(|| null_parameters("Parameters"))?;
        let length: i32 = sequence.sequence(|list| list.length());
        if index + 1 < 1 || index + 1 > length {
            return Err(Failure::with_message(
                "Standard_OutOfRange",
                &format!(
                    "IntSurf_PathPoint::Parameters: index {index} outside 0..={}",
                    length - 1
                ),
            ));
        }
        let uv: Xy = sequence.sequence(|list| list.value((index + 1) as usize).copied())?;
        Ok((uv.x(), uv.y()))
    }
}

/// The tool through which the walking algorithms read a path point
/// (`IntSurf_PathPointTool`); `IntSurfPathPointTool::value3d(&p)`.
pub struct IntSurfPathPointTool;

impl IntSurfPathPointTool {
    /// The 3d point (`Value3d`); `IntSurfPathPointTool::value3d(&p)`.
    pub const fn value3d(p_start: &IntSurfPathPoint) -> Point3 {
        p_start.value()
    }

    /// The principal parameters (`Value2d`); `IntSurfPathPointTool::value2d(&p)?`.
    pub fn value2d(p_start: &IntSurfPathPoint) -> Result<(f64, f64), Failure> {
        p_start.value2d()
    }

    /// Whether the line passes through the point (`IsPassingPnt`).
    /// `IntSurfPathPointTool::is_passing_pnt(&p)`.
    pub const fn is_passing_pnt(p_start: &IntSurfPathPoint) -> bool {
        p_start.is_passing_pnt()
    }

    /// Whether the surfaces are tangent at the point (`IsTangent`).
    /// `IntSurfPathPointTool::is_tangent(&p)`.
    pub const fn is_tangent(p_start: &IntSurfPathPoint) -> bool {
        p_start.is_tangent()
    }

    /// The 3d tangent (`Direction3d`); `IntSurfPathPointTool::direction3d(&p)?`.
    pub fn direction3d(p_start: &IntSurfPathPoint) -> Result<Vector3, Failure> {
        p_start.direction3d()
    }

    /// The parametric tangent (`Direction2d`); `IntSurfPathPointTool::direction2d(&p)?`.
    pub fn direction2d(p_start: &IntSurfPathPoint) -> Result<Direction2, Failure> {
        p_start.direction2d()
    }

    /// The multiplicity (`Multiplicity`); `IntSurfPathPointTool::multiplicity(&p)?`.
    pub fn multiplicity(p_start: &IntSurfPathPoint) -> Result<i32, Failure> {
        p_start.multiplicity()
    }

    /// The auxiliary parameters of rank `mult` (`Parameters`).
    /// `IntSurfPathPointTool::parameters(&p, 1)?`.
    pub fn parameters(p_start: &IntSurfPathPoint, mult: i32) -> Result<(f64, f64), Failure> {
        p_start.parameters(mult)
    }
}
