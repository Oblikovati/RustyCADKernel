// Copyright (c) 2026 OPEN CASCADE SAS
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

//! Complete legacy KDTree declarations; original assertions and deterministic fixtures.
#![allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_assignments,
    unused_parens,
    clippy::double_parens,
    clippy::explicit_auto_deref,
    clippy::needless_borrow,
    clippy::len_zero,
    clippy::range_plus_one,
    clippy::needless_parens_on_range_literals,
    clippy::unnecessary_cast,
    clippy::needless_range_loop,
    clippy::approx_constant,
    clippy::excessive_precision
)]
use rustycad_tkernel::ncollection::{Array1, KDTree};
#[path = "ncollection_kdtree/support.rs"]
mod support;
use support::*;
macro_rules! near {
    ($a:expr,$b:expr,$e:expr) => {{
        let actual: f64 = $a;
        let expected: f64 = $b;
        assert!(
            (actual - expected).abs() <= $e,
            "actual {actual}, expected {expected}, tolerance {}",
            $e
        );
    }};
}
include!("ncollection_kdtree/cases_00.rs");
include!("ncollection_kdtree/cases_01.rs");
include!("ncollection_kdtree/cases_02.rs");
include!("ncollection_kdtree/cases_03.rs");
include!("ncollection_kdtree/cases_04.rs");
include!("ncollection_kdtree/cases_05.rs");
include!("ncollection_kdtree/cases_06.rs");
include!("ncollection_kdtree/cases_07.rs");
include!("ncollection_kdtree/cases_08.rs");
include!("ncollection_kdtree/cases_09.rs");
include!("ncollection_kdtree/cases_10.rs");
include!("ncollection_kdtree/cases_11.rs");
include!("ncollection_kdtree/cases_12.rs");
include!("ncollection_kdtree/cases_13.rs");
include!("ncollection_kdtree/cases_14.rs");
include!("ncollection_kdtree/cases_15.rs");
include!("ncollection_kdtree/cases_16.rs");
include!("ncollection_kdtree/cases_17.rs");
include!("ncollection_kdtree/cases_18.rs");
