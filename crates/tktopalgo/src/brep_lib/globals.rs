// Created on: 1993-12-15
// Created by: Remi LEQUETTE
// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of BRepLib::Precision and BRepLib::Plane (BRepLib.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The two process-wide settings of `BRepLib` (docs/topalgo-design.md section 7): the
//! default precision of the makers and the plane of the 2d edges.
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::GeomPlane;
use rustycad_tkmath::gp::{self, Axis3};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex, PoisonError};

// `static double thePrecision = Precision::Confusion();` as IEEE bits, so that a set
// from any thread is seen by the next read, as the native (unsynchronized) static.
static THE_PRECISION: AtomicU64 = AtomicU64::new(precision::confusion().to_bits());

// `static occ::handle<Geom_Plane> thePlane;`: null until the first read.
static THE_PLANE: LazyLock<Mutex<Handle<GeomPlane>>> =
    LazyLock::new(|| Mutex::new(Handle::default()));

/// `BRepLib::Precision(P)`: sets the default precision of the makers;
/// `brep_lib::set_precision(1e-6)`.
pub fn set_precision(value: f64) {
    THE_PRECISION.store(value.to_bits(), Ordering::SeqCst);
}

/// `BRepLib::Precision()`: the default precision (Precision::Confusion() until set);
/// `assert_eq!(rustycad_tktopalgo::brep_lib::precision(), 1e-7)`.
pub fn precision() -> f64 {
    f64::from_bits(THE_PRECISION.load(Ordering::SeqCst))
}

/// `BRepLib::Plane(P)`: sets the plane of the 2d makers (a null handle resets it to
/// the default at the next read); `brep_lib::set_plane(&plane)`.
pub fn set_plane(plane: &Handle<GeomPlane>) {
    *THE_PLANE.lock().unwrap_or_else(PoisonError::into_inner) = plane.clone();
}

/// `BRepLib::Plane()`: the current plane, created as `Geom_Plane(gp::XOY())` when null;
/// the returned handle shares the stored plane;
/// `assert!(rustycad_tktopalgo::brep_lib::plane().is_some())`.
pub fn plane() -> Handle<GeomPlane> {
    let mut the_plane = THE_PLANE.lock().unwrap_or_else(PoisonError::into_inner);
    if the_plane.is_null() {
        *the_plane = Handle::new(GeomPlane::new(Axis3::from_axis2(gp::xoy())));
    }
    the_plane.clone()
}
