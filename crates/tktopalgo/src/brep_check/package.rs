// Created on: 1995-12-08
// Created by: Jacques GOUSSARD
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
// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepCheck/BRepCheck.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepCheck`: the package functions (status lists, the printer, the precisions).
use super::status::{BRepCheckStatus, ListOfStatus};
use super::wire::BRepCheckWire;
use rustycad_tkbrep::topods::{Edge, Face, Wire};
use rustycad_tkernel::standard::real::{epsilon, real_epsilon};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use std::io::{self, Write};

/// `BRepCheck`: the package tools to check the validity of a BRep.
///
/// ```
/// use rustycad_tktopalgo::brep_check::{BRepCheck, BRepCheckStatus, ListOfStatus};
/// let mut list = ListOfStatus::default();
/// BRepCheck::add(&mut list, BRepCheckStatus::NoError);
/// BRepCheck::add(&mut list, BRepCheckStatus::FreeEdge);
/// assert_eq!(list.iter().copied().collect::<Vec<_>>(), [BRepCheckStatus::FreeEdge]);
/// ```
pub struct BRepCheck;

impl BRepCheck {
    /// `Add(List, Stat)`: appends `status` unless the list holds it already; a defect
    /// removes the `NoError` entries first; `BRepCheck::add(&mut list, status)`.
    pub fn add(list: &mut ListOfStatus, status: BRepCheckStatus) {
        let mut cursor = list.cursor();
        while let Some(&value) = cursor.value(list) {
            if value == BRepCheckStatus::NoError && status != BRepCheckStatus::NoError {
                // Removing the current item moves the cursor to the next one.
                list.remove(&mut cursor)
                    .expect("the cursor is on an item of the list");
                continue;
            }
            if value == status {
                return;
            }
            cursor.next(list);
        }
        list.append(status);
    }

    /// `Print(Stat, OS)`: the enumerator name and a line end;
    /// `BRepCheck::print(status, &mut out)?`.
    pub fn print(status: BRepCheckStatus, out: &mut dyn Write) -> io::Result<()> {
        writeln!(out, "{}", status.name())
    }

    /// `SelfIntersection(W, F, E1, E2)`: whether `wire` intersects itself on `face`
    /// (`BRepCheck_Wire::SelfIntersect`), with the first intersecting edges written to
    /// `e1` and `e2` (see [`BRepCheckWire::self_intersect`]);
    /// `BRepCheck::self_intersection(&w, &f, &mut e1, &mut e2)?`.
    pub fn self_intersection(
        wire: &Wire,
        face: &Face,
        e1: &mut Edge,
        e2: &mut Edge,
    ) -> Result<bool, Failure> {
        let checker = BRepCheckWire::new(wire)?;
        let status = checker.self_intersect(face, e1, e2, false)?;
        Ok(status == BRepCheckStatus::SelfIntersectingWire)
    }

    /// `PrecCurve(aAC3D)`: the resolution on the 3d curve, `RealEpsilon()` except for an
    /// ellipse, whose largest `Epsilon` of its centre coordinates and radii it is;
    /// `BRepCheck::prec_curve(&adaptor)?`.
    pub fn prec_curve(curve: &dyn Adaptor3dCurve) -> Result<f64, Failure> {
        if curve.get_type()? != CurveType::Ellipse {
            return Ok(real_epsilon());
        }
        let ellipse = curve.ellipse()?;
        let centre = ellipse.location();
        Ok(largest_epsilon(&[
            centre.x(),
            centre.y(),
            centre.z(),
            ellipse.major_radius(),
            ellipse.minor_radius(),
        ]))
    }

    /// `PrecSurface(aAHSurf)`: the resolution on the surface, `RealEpsilon()` except for
    /// a cone, whose largest `Epsilon` of its apex-frame origin and reference radius it
    /// is; Standard_NullObject for a null handle (a crash natively);
    /// `BRepCheck::prec_surface(&surface)?`.
    pub fn prec_surface(surface: &Handle<dyn Adaptor3dSurface>) -> Result<f64, Failure> {
        let surface = surface.get().ok_or_else(|| {
            super::errors::null_object("BRepCheck::PrecSurface: null surface adaptor")
        })?;
        if surface.get_type()? != SurfaceType::Cone {
            return Ok(real_epsilon());
        }
        let cone = surface.cone()?;
        let origin = cone.location();
        Ok(largest_epsilon(&[
            origin.x(),
            origin.y(),
            origin.z(),
            cone.ref_radius(),
        ]))
    }
}

/// The largest `Epsilon(|x|)` of `values`, starting from -1 as natively.
fn largest_epsilon(values: &[f64]) -> f64 {
    let mut largest: f64 = -1.0;
    for &value in values {
        let magnitude: f64 = if value < 0.0 { -value } else { value };
        let candidate: f64 = epsilon(magnitude);
        if candidate > largest {
            largest = candidate;
        }
    }
    largest
}
