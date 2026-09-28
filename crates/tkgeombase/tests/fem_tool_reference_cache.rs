// Copyright (c) 1998-1999 Matra Datavision
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

//! The process-wide reference integrals of the linear criteria (the native
//! function-local statics): one test in its own binary, since the cache is shared.
use rustycad_tkgeombase::fem_tool::{LinearFlexion, LinearTension};
use rustycad_tkmath::geom_abs::Shape;

#[test]
fn a_cached_order_skips_the_work_degree_check() {
    // A new order refuses WorkDegree 15 before integrating.
    assert_eq!(
        LinearFlexion::new(15, Shape::C1)
            .err()
            .map(|e| e.exception_type()),
        Some("Standard_ConstructionError")
    );
    assert!(LinearFlexion::new(14, Shape::C1).is_ok());
    // The same order is cached: WorkDegree 15 reads past the 120 integrals.
    assert_eq!(
        LinearFlexion::new(15, Shape::C1)
            .err()
            .map(|e| e.exception_type()),
        Some("Standard_OutOfRange")
    );
    // Each criterion class has its own statics.
    assert_eq!(
        LinearTension::new(15, Shape::C1)
            .err()
            .map(|e| e.exception_type()),
        Some("Standard_ConstructionError")
    );
}
