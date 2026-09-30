// Copyright (c) 1997-1999 Matra Datavision
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
// Rust translation of the arguments of the Section members of
// ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_PolynomialConvertor.hxx and
// GeomFill_QuasiAngularConvertor.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The arguments of the circular-arc convertors (`Section` of
//! `GeomFill_PolynomialConvertor` and `GeomFill_QuasiAngularConvertor`), grouped as
//! structs because the native members take up to eighteen of them.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkmath::gp::{Point3, Vector3};

/// An arc of centre `center` from `first_pnt`, turning by `angle` around `dir`
/// (`FirstPnt`, `Center`, `Dir`, `Angle`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ArcSectionD0 {
    /// `FirstPnt`.
    pub first_pnt: Point3,
    /// `Center`.
    pub center: Point3,
    /// `Dir`.
    pub dir: Vector3,
    /// `Angle`.
    pub angle: f64,
}

/// The arc and the first derivatives of its data (`DFirstPnt`, `DCenter`, `DDir`,
/// `DAngle`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ArcSectionD1 {
    /// The arc itself.
    pub arc: ArcSectionD0,
    /// `DFirstPnt`.
    pub d_first_pnt: Vector3,
    /// `DCenter`.
    pub d_center: Vector3,
    /// `DDir`.
    pub d_dir: Vector3,
    /// `DAngle`.
    pub d_angle: f64,
}

/// The arc and the first two derivatives of its data (`D2FirstPnt`, `D2Center`,
/// `D2Dir`, `D2Angle`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ArcSectionD2 {
    /// The arc and its first derivatives.
    pub d1: ArcSectionD1,
    /// `D2FirstPnt`.
    pub d2_first_pnt: Vector3,
    /// `D2Center`.
    pub d2_center: Vector3,
    /// `D2Dir`.
    pub d2_dir: Vector3,
    /// `D2Angle`.
    pub d2_angle: f64,
}

/// The rational section and its first derivatives (`Poles`, `DPoles`, `Weigths`,
/// `DWeigths` outputs).
pub struct ArcPolesD1<'a, 'b> {
    /// `Poles`.
    pub poles: &'a mut Array1<'b, Point3>,
    /// `DPoles`.
    pub d_poles: &'a mut Array1<'b, Vector3>,
    /// `Weigths`.
    pub weights: &'a mut Array1<'b, f64>,
    /// `DWeigths`.
    pub d_weights: &'a mut Array1<'b, f64>,
}

/// The rational section and its first two derivatives (adds `D2Poles`,
/// `D2Weigths`).
pub struct ArcPolesD2<'a, 'b> {
    /// The value and first derivatives.
    pub d1: ArcPolesD1<'a, 'b>,
    /// `D2Poles`.
    pub d2_poles: &'a mut Array1<'b, Vector3>,
    /// `D2Weigths`.
    pub d2_weights: &'a mut Array1<'b, f64>,
}
