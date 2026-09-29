// Created on: 1995-03-15
// Created by: Laurent PAINNOT
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
// Rust translation of ModelingData/TKBRep/BRep/BRep_GCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRep_GCurve`: the representations with a parameter range (docs/topology-design.md section 9).
use super::curve_representation::{
    BRepCurveRepresentation, CurveRepresentationBase, dump_curve_representation,
};
use super::dump::{dump_base, dump_class_name};
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{Failure, SharedState};
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::toploc::Location;
use std::io;

/// The protected data of `BRep_GCurve`: the curve representation data and the
/// private `myFirst` and `myLast`.
///
/// ```
/// use rustycad_tkbrep::brep::GCurveBase;
/// use rustycad_tkmath::toploc::Location;
/// let base = GCurveBase::new(&Location::default(), 0.0, 2.0);
/// assert_eq!(base.range(), (0.0, 2.0));
/// ```
#[derive(Default)]
pub struct GCurveBase {
    representation: CurveRepresentationBase,
    range: SharedState<(f64, f64)>,
}

impl GCurveBase {
    /// The protected `BRep_GCurve(L, First, Last)`; `GCurveBase::new(&location, 0.0, 1.0)`.
    pub fn new(location: &Location, first: f64, last: f64) -> Self {
        Self {
            representation: CurveRepresentationBase::new(location),
            range: SharedState::new((first, last)),
        }
    }

    /// The embedded `BRep_CurveRepresentation` data; `base.representation()`.
    pub fn representation(&self) -> &CurveRepresentationBase {
        &self.representation
    }

    /// `Range(First, Last)`; `base.range()`.
    pub fn range(&self) -> (f64, f64) {
        *self.range.read()
    }

    /// Stores the range without calling `Update`; `base.store_range(0.0, 1.0)`.
    pub fn store_range(&self, first: f64, last: f64) {
        *self.range.write() = (first, last);
    }

    /// Stores the first parameter only; `base.store_first(0.0)`.
    pub fn store_first(&self, first: f64) {
        self.range.write().0 = first;
    }

    /// Stores the last parameter only; `base.store_last(1.0)`.
    pub fn store_last(&self, last: f64) {
        self.range.write().1 = last;
    }
}

/// `BRep_GCurve`: a representation with a parameter range; `D0` is pure virtual and
/// `Update` recomputes the derived data after a range change.
///
/// ```
/// use rustycad_tkbrep::brep::{BRepCurve3D, BRepGCurve};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::toploc::Location;
/// let curve = BRepCurve3D::new(&Handle::default(), &Location::default());
/// curve.set_range(1.0, 2.0).unwrap();
/// assert_eq!((curve.first(), curve.last()), (1.0, 2.0));
/// ```
pub trait BRepGCurve: BRepCurveRepresentation {
    /// The embedded `BRep_GCurve` data.
    fn gcurve_base(&self) -> &GCurveBase;

    /// `SetRange(First, Last)`: stores the range, then calls `Update`;
    /// `curve.set_range(0.0, 1.0)?`.
    fn set_range(&self, first: f64, last: f64) -> Result<(), Failure> {
        self.gcurve_base().store_range(first, last);
        self.update()
    }
    /// `Range(First, Last)`; `let (first, last) = curve.range();`.
    fn range(&self) -> (f64, f64) {
        self.gcurve_base().range()
    }
    /// `First()`; `curve.first()`.
    fn first(&self) -> f64 {
        self.gcurve_base().range().0
    }
    /// `Last()`; `curve.last()`.
    fn last(&self) -> f64 {
        self.gcurve_base().range().1
    }
    /// `First(F)`: stores the first parameter, then calls `Update`; `curve.set_first(0.0)?`.
    fn set_first(&self, first: f64) -> Result<(), Failure> {
        self.gcurve_base().store_first(first);
        self.update()
    }
    /// `Last(L)`: stores the last parameter, then calls `Update`; `curve.set_last(1.0)?`.
    fn set_last(&self, last: f64) -> Result<(), Failure> {
        self.gcurve_base().store_last(last);
        self.update()
    }
    /// `D0(U, P)`: the point at parameter `u`; `curve.d0(0.5)?`.
    fn d0(&self, u: f64) -> Result<Point3, Failure>;
    /// `Update()`: recomputes the derived data after a range change; the base does
    /// nothing; `curve.update()?`.
    fn update(&self) -> Result<(), Failure> {
        Ok(())
    }
}
impl_transient_interface!(dyn BRepGCurve, "BRep_GCurve", dyn BRepCurveRepresentation);

/// The body of `BRep_GCurve::DumpJson`: the class name, the base class and the range;
/// `dump_gcurve(base, &mut stream, -1)?`.
pub fn dump_gcurve(base: &GCurveBase, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
    dump_class_name(stream, "BRep_GCurve")?;
    dump_base(stream, depth, |nested, nested_depth| {
        dump_curve_representation(base.representation(), nested, nested_depth)
    })?;
    let (first, last) = base.range();
    dump::dump_real(stream, "First", first)?;
    dump::dump_real(stream, "Last", last)
}
