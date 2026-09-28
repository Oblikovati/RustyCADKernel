// Copyright (c) 2005-2014 OPEN CASCADE SAS
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

//! math_ValueAndWeight: a quadrature node paired with its weight, ordered by node.
use std::cmp::Ordering;

/// A (value, weight) pair; the default is `(0, 0)`. `ValueAndWeight::new(0.5, 1.0)`.
#[derive(Clone, Copy, Debug, Default)]
pub struct ValueAndWeight {
    value: f64,
    weight: f64,
}
impl ValueAndWeight {
    /// Pairs a value with its weight; `ValueAndWeight::new(-0.5, 2.0).weight() == 2.0`.
    pub fn new(value: f64, weight: f64) -> Self {
        Self { value, weight }
    }
    /// The stored value; `ValueAndWeight::new(1.0, 2.0).value() == 1.0`.
    pub fn value(&self) -> f64 {
        self.value
    }
    /// The stored weight; `ValueAndWeight::new(1.0, 2.0).weight() == 2.0`.
    pub fn weight(&self) -> f64 {
        self.weight
    }
    /// The free `operator<`: compares values only, so NaN is never less;
    /// `ValueAndWeight::new(1.0, 9.0).less(&ValueAndWeight::new(2.0, 0.0))`.
    pub fn less(&self, other: &Self) -> bool {
        self.value < other.value
    }
    /// Sorts ascending by value with the `operator<` strict weak order that the native
    /// `std::sort` call relies on; `ValueAndWeight::sort(&mut nodes)`.
    pub fn sort(nodes: &mut [Self]) {
        nodes.sort_unstable_by(|left, right| {
            if left.less(right) {
                Ordering::Less
            } else if right.less(left) {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });
    }
}
