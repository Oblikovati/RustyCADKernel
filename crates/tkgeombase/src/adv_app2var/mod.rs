// Copyright (c) 1996-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/AdvApp2Var at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The AdvApp2Var package: approximation of a function of two parameters by
//! polynomial patches and B-spline surfaces.
//!
//! The package has two layers. The C++ classes drive the cutting of the domain
//! into patches and iso-curves; the numerical work is f2c translations of
//! Fortran routines in [`math_base`], [`approx_f2var`] and [`sys_base`], over the
//! common-block tables of [`data`].
//!
//! The kernels keep the native routine names, argument order and loop structure,
//! so every floating-point operation happens in the native order. Scalars that the
//! native code only reads are passed by value, scalars it writes are `&mut`, and an
//! array argument is the slice that starts at the native pointer; inside, the f2c
//! pointer shift is kept by the views of the `f2c` module. The calls to the trace
//! routines (`mgenmsg_`, `mgsomsg_`, `maermsg_`) are left out of the kernels: the
//! debug level [`sys_base::mnfndeb_`] is 0 and the routines do nothing.


pub mod data;

pub mod sys_base;
