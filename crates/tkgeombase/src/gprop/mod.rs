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
// Rust translation of ModelingData/TKGeomBase/GProp at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! GProp: global properties (mass, centre of mass, inertia) of point sets,
//! elementary curves, surfaces and solids, and their composition.
//!
//! [`GProps`] is the native base class; the derived frameworks ([`PGProps`],
//! [`CelGProps`], [`SelGProps`], [`VelGProps`]) own one and reach its members
//! through `Deref`, so any of them is passed where a `&GProps` is expected.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Matrix3, Point3, Xyz};

mod cel_gprops;
mod elementary;
mod gprops;
mod pequation;
mod pgprops;
mod principal_props;
mod sel_gprops;
mod vel_gprops;

pub use cel_gprops::CelGProps;
pub use gprops::GProps;
pub use pequation::{PEquation, PEquationType};
pub use pgprops::PGProps;
pub use principal_props::PrincipalProps;
pub use sel_gprops::SelGProps;
pub use vel_gprops::VelGProps;

/// `GProp_ValueType`: the kinds of global property values;
/// `ValueType::Mass as i32 == 0`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ValueType {
    /// The mass.
    Mass,
    /// X of the centre of mass.
    CenterMassX,
    /// Y of the centre of mass.
    CenterMassY,
    /// Z of the centre of mass.
    CenterMassZ,
    /// Moment about X.
    InertiaXX,
    /// Moment about Y.
    InertiaYY,
    /// Moment about Z.
    InertiaZZ,
    /// Product XY.
    InertiaXY,
    /// Product XZ.
    InertiaXZ,
    /// Product YZ.
    InertiaYZ,
    /// Unknown.
    Unknown,
}

/// `GProp_UndefinedAxis` (a Standard_DomainError natively): an undefined axis of
/// symmetry was referenced; no GProp path raises it;
/// `undefined_axis("PrincipalProps: no axis of symmetry")`.
pub fn undefined_axis(message: &str) -> Failure {
    Failure::with_type("GProp_UndefinedAxis", message, "")
}

/// `GProp::HOperator(G, Q, Mass, Operator)`: the Huygens operator, the inertia of
/// the mass `mass` at `g` relative to `q`;
/// `h_operator(Point3::new(1.0, 0.0, 0.0), Point3::default(), 2.0)`.
pub fn h_operator(g: Point3, q: Point3, mass: f64) -> Matrix3 {
    let qg: Xyz = g.xyz() - q.xyz();
    let ixx: f64 = qg.y() * qg.y() + qg.z() * qg.z();
    let iyy: f64 = qg.x() * qg.x() + qg.z() * qg.z();
    let izz: f64 = qg.y() * qg.y() + qg.x() * qg.x();
    let ixy: f64 = -qg.x() * qg.y();
    let iyz: f64 = -qg.y() * qg.z();
    let ixz: f64 = -qg.x() * qg.z();
    let mut operator = Matrix3::from_columns(
        Xyz::new(ixx, ixy, ixz),
        Xyz::new(ixy, iyy, iyz),
        Xyz::new(ixz, iyz, izz),
    );
    operator *= mass;
    operator
}
