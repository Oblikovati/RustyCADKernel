// Created on: 2007-05-26
// Created by: Andrey BETENEV
// Copyright (c) 2007-2014 OPEN CASCADE SAS
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

pub(super) fn cell_coordinate(value: f64) -> i32 {
    let wrapped: f64 = if value > f64::from(i32::MAX - 1) {
        value % f64::from(i32::MAX)
    } else if value < f64::from(i32::MIN + 1) {
        value % f64::from(i32::MIN)
    } else {
        value
    };
    wrapped as i32
}
pub(super) struct CellIndices {
    min: Vec<i32>,
    max: Vec<i32>,
    current: Option<Vec<i32>>,
}
impl CellIndices {
    pub(super) fn new(min: Vec<i32>, max: Vec<i32>) -> Self {
        let reversed: bool = min.iter().zip(&max).any(|(lo, hi)| lo > hi);
        let current: Option<Vec<i32>> = (!reversed).then(|| min.clone());
        Self { min, max, current }
    }
    fn advance(&mut self) {
        let current: &mut Vec<i32> = self.current.as_mut().unwrap();
        for (axis, value) in current.iter_mut().enumerate() {
            if *value < self.max[axis] {
                *value += 1;
                return;
            }
            *value = self.min[axis];
        }
        self.current = None;
    }
}
impl Iterator for CellIndices {
    type Item = Vec<i32>;
    fn next(&mut self) -> Option<Self::Item> {
        let result: Vec<i32> = self.current.clone()?;
        self.advance();
        Some(result)
    }
}
