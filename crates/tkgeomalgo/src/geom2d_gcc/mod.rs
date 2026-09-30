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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/Geom2dGcc (Geom2dGcc.hxx/.cxx,
// Geom2dGcc_IsParallel.hxx, Geom2dGcc_Type1.hxx, Geom2dGcc_Type2.hxx,
// Geom2dGcc_Type3.hxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package Geom2dGcc: 2d constraint solvers on qualified curves. They build
//! the lines and circles tangent to, passing through or centred on arbitrary 2d
//! curves (`Geom2dAdaptor_Curve`), points and loci.
//!
//! Each solver calls the analytic GccAna solver when its arguments are lines and
//! circles, the geometric `*Geo` class (offset curves intersected with Geom2dInt)
//! for a curve and a fixed radius or centre locus, and the iterative `*Iter` class
//! (a tkmath root search from given parameters) otherwise. The `*Iter` and `*Geo`
//! classes are concrete at the pinned commit (docs/geomalgo-design.md, section 9).
//! Every native constructor overload is a named constructor returning `Result`, as
//! in GccAna, and output references become returned tuples.
mod circ2d_2tan_on;
mod circ2d_2tan_on_geo;
mod circ2d_2tan_on_iter;
mod circ2d_2tan_rad;
mod circ2d_2tan_rad_geo;
mod circ2d_3tan;
mod circ2d_3tan_iter;
mod circ2d_tan_cen;
mod circ2d_tan_cen_geo;
mod circle_solutions;
mod curve_tool;
mod domains;
mod function_tan_cir_cu;
mod function_tan_cu_cu;
mod function_tan_cu_cu_cu;
mod function_tan_cu_cu_on_cu;
mod function_tan_cu_pnt;
mod function_tan_obl;
mod lin2d_2tan;
mod lin2d_2tan_iter;
mod lin2d_tan_obl;
mod lin2d_tan_obl_iter;
mod qualified_curve;

pub use circ2d_2tan_on::Circ2d2TanOn;
pub use circ2d_2tan_on_geo::Circ2d2TanOnGeo;
pub use circ2d_2tan_on_iter::Circ2d2TanOnIter;
pub use circ2d_2tan_rad::Circ2d2TanRad;
pub use circ2d_2tan_rad_geo::Circ2d2TanRadGeo;
pub use circ2d_3tan::Circ2d3Tan;
pub use circ2d_3tan_iter::Circ2d3TanIter;
pub use circ2d_tan_cen::Circ2dTanCen;
pub use circ2d_tan_cen_geo::Circ2dTanCenGeo;
pub use curve_tool::CurveTool;
pub use function_tan_cir_cu::FunctionTanCirCu;
pub use function_tan_cu_cu::FunctionTanCuCu;
pub use function_tan_cu_cu_cu::FunctionTanCuCuCu;
pub use function_tan_cu_cu_on_cu::FunctionTanCuCuOnCu;
pub use function_tan_cu_pnt::FunctionTanCuPnt;
pub use function_tan_obl::FunctionTanObl;
pub use lin2d_2tan::Lin2d2Tan;
pub use lin2d_2tan_iter::Lin2d2TanIter;
pub use lin2d_tan_obl::Lin2dTanObl;
pub use lin2d_tan_obl_iter::Lin2dTanOblIter;
pub use qualified_curve::{QCurve, QualifiedCurve};

use crate::gcc_ent::Position;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dLine, Geom2dPoint};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::gp::{Circle2, Line2, Point2};

/// `Geom2dGcc_Type1`: the kinds of the three arguments of
/// [`FunctionTanCuCuCu`] (curve, circle, line), in the native order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type1 {
    /// `Geom2dGcc_CuCuCu`: three curves.
    CuCuCu,
    /// `Geom2dGcc_CiCuCu`: a circle and two curves.
    CiCuCu,
    /// `Geom2dGcc_CiCiCu`: two circles and a curve.
    CiCiCu,
    /// `Geom2dGcc_CiLiCu`: a circle, a line and a curve.
    CiLiCu,
    /// `Geom2dGcc_LiLiCu`: two lines and a curve.
    LiLiCu,
    /// `Geom2dGcc_LiCuCu`: a line and two curves.
    LiCuCu,
}

/// `Geom2dGcc_Type2`: the kinds of the two tangency arguments and of the centre
/// locus of [`FunctionTanCuCuOnCu`], in the native order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type2 {
    /// `Geom2dGcc_CuCuOnCu`.
    CuCuOnCu,
    /// `Geom2dGcc_CiCuOnCu`.
    CiCuOnCu,
    /// `Geom2dGcc_LiCuOnCu`.
    LiCuOnCu,
    /// `Geom2dGcc_CuPtOnCu`.
    CuPtOnCu,
    /// `Geom2dGcc_CuCuOnLi`.
    CuCuOnLi,
    /// `Geom2dGcc_CiCuOnLi`.
    CiCuOnLi,
    /// `Geom2dGcc_LiCuOnLi`.
    LiCuOnLi,
    /// `Geom2dGcc_CuPtOnLi`.
    CuPtOnLi,
    /// `Geom2dGcc_CuCuOnCi`.
    CuCuOnCi,
    /// `Geom2dGcc_CiCuOnCi`.
    CiCuOnCi,
    /// `Geom2dGcc_LiCuOnCi`.
    LiCuOnCi,
    /// `Geom2dGcc_CuPtOnCi`.
    CuPtOnCi,
}

/// `Geom2dGcc_Type3`: the kinds of the two arguments of [`FunctionTanCuCu`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type3 {
    /// `Geom2dGcc_CuCu`: two curves.
    CuCu,
    /// `Geom2dGcc_CiCu`: a circle and a curve.
    CiCu,
}

/// `Geom2dGcc::Unqualified(Obj)`: a curve whose solutions may lie on any side;
/// `geom2d_gcc::unqualified(&adaptor)?.is_unqualified()`.
pub fn unqualified(obj: &Geom2dAdaptorCurve) -> Result<QualifiedCurve, Failure> {
    QualifiedCurve::new(obj, Position::Unqualified)
}

/// `Geom2dGcc::Enclosing(Obj)`: the solutions enclose the curve;
/// `geom2d_gcc::enclosing(&adaptor)?.is_enclosing()`.
pub fn enclosing(obj: &Geom2dAdaptorCurve) -> Result<QualifiedCurve, Failure> {
    QualifiedCurve::new(obj, Position::Enclosing)
}

/// `Geom2dGcc::Enclosed(Obj)`: the solutions are enclosed by the curve;
/// `geom2d_gcc::enclosed(&adaptor)?.is_enclosed()`.
pub fn enclosed(obj: &Geom2dAdaptorCurve) -> Result<QualifiedCurve, Failure> {
    QualifiedCurve::new(obj, Position::Enclosed)
}

/// `Geom2dGcc::Outside(Obj)`: the solutions and the curve are outside each other;
/// `geom2d_gcc::outside(&adaptor)?.is_outside()`.
pub fn outside(obj: &Geom2dAdaptorCurve) -> Result<QualifiedCurve, Failure> {
    QualifiedCurve::new(obj, Position::Outside)
}

/// `Geom2dGcc_IsParallel` (a Standard_DomainError natively), raised when the
/// intersection with a line parallel to the solution is read;
/// `is_parallel("Geom2dGcc_Lin2dTanOblIter::Intersection2")`.
pub fn is_parallel(member: &str) -> Failure {
    Failure::with_type(
        "Geom2dGcc_IsParallel",
        &format!("{member}: the solution is parallel to the line; expected a secant line"),
        "",
    )
}

// `Geom2dAdaptor_Curve C = Q.Qualified()`: a value copy sharing the curve.
pub(crate) fn copy_adaptor(adaptor: &Geom2dAdaptorCurve) -> Result<Geom2dAdaptorCurve, Failure> {
    adaptor.shallow_adaptor()
}

// The Standard_NullObject that stands for a native null handle dereference.
pub(crate) fn null_object(member: &str) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!("{member}: null object; expected a loaded curve or point"),
        "",
    )
}

// `occ::down_cast<Geom2d_Circle>(C.Curve())->Circ2d()`, for an adaptor of type Circle.
pub(crate) fn circle_of(adaptor: &Geom2dAdaptorCurve) -> Result<Circle2, Failure> {
    let handle: Handle<Geom2dCircle> = adaptor.curve().cast();
    let circle = handle
        .get()
        .ok_or_else(|| null_object("Geom2d_Circle::Circ2d"))?;
    circle.circ2d()
}

// `occ::down_cast<Geom2d_Line>(C.Curve())->Lin2d()`, for an adaptor of type Line.
pub(crate) fn line_of(adaptor: &Geom2dAdaptorCurve) -> Result<Line2, Failure> {
    let handle: Handle<Geom2dLine> = adaptor.curve().cast();
    let line = handle
        .get()
        .ok_or_else(|| null_object("Geom2d_Line::Lin2d"))?;
    Ok(line.lin2d())
}

// `Point->Pnt2d()` of a `occ::handle<Geom2d_Point>` argument.
pub(crate) fn point_of(point: &Handle<dyn Geom2dPoint>) -> Result<Point2, Failure> {
    let geometry = point
        .get()
        .ok_or_else(|| null_object("Geom2d_Point::Pnt2d"))?;
    Ok(geometry.pnt2d())
}

// The first failure of a curve evaluation inside a tkmath callback, which reports
// only `false`: the solver that ran the function returns it afterwards, where the
// native exception would have left it (docs/geomalgo-design.md, section 9).
#[derive(Default)]
pub(crate) struct EvalFailure {
    failure: Option<Failure>,
}

impl EvalFailure {
    // The value of `result`, or `None` after keeping its (first) failure.
    pub(crate) fn keep<T>(&mut self, result: Result<T, Failure>) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(failure) => {
                self.failure.get_or_insert(failure);
                None
            }
        }
    }
    // `Err` with the kept failure, if any.
    pub(crate) fn check(&self) -> Result<(), Failure> {
        match &self.failure {
            Some(failure) => Err(failure.clone()),
            None => Ok(()),
        }
    }
}
