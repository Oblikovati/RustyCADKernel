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

#![allow(clippy::excessive_precision)]
const POINT: &[&[f64]] = &[
    include!("kronrod_point_0.rs"),
    include!("kronrod_point_1.rs"),
    include!("kronrod_point_2.rs"),
    include!("kronrod_point_3.rs"),
    include!("kronrod_point_4.rs"),
    include!("kronrod_point_5.rs"),
    include!("kronrod_point_6.rs"),
    include!("kronrod_point_7.rs"),
];
pub(super) fn point(index: usize) -> f64 {
    POINT[index / 256][index % 256]
}
const WEIGHT: &[&[f64]] = &[
    include!("kronrod_weight_0.rs"),
    include!("kronrod_weight_1.rs"),
    include!("kronrod_weight_2.rs"),
    include!("kronrod_weight_3.rs"),
    include!("kronrod_weight_4.rs"),
    include!("kronrod_weight_5.rs"),
    include!("kronrod_weight_6.rs"),
    include!("kronrod_weight_7.rs"),
];
pub(super) fn weight(index: usize) -> f64 {
    WEIGHT[index / 256][index % 256]
}
