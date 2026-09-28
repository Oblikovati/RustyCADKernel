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

use super::{Failure, Gauss, Vector};
impl Gauss {
    pub(in crate::math) fn solve_lu(&self, rhs: &mut Vector<'_>) -> Result<(), Failure> {
        self.forward_substitute(rhs)?;
        let lower = rhs.lower() - 1;
        for row in (1..=self.lu.row_number()).rev() {
            let mut sum = *rhs.value(row + lower)?;
            for col in row + 1..=self.lu.row_number() {
                sum -= self.lu[(row, col)] * rhs.value(col + lower)?;
            }
            *rhs.change_value(row + lower)? = sum / self.lu[(row, row)];
        }
        Ok(())
    }
    fn forward_substitute(&self, rhs: &mut Vector<'_>) -> Result<(), Failure> {
        let mut first = 0;
        let lower = rhs.lower() - 1;
        for row in 1..=self.lu.row_number() {
            let pivot = self.pivots[(row - 1) as usize];
            let mut sum = *rhs.value(pivot + lower)?;
            *rhs.change_value(pivot + lower)? = *rhs.value(row + lower)?;
            if first != 0 {
                for col in first..row {
                    sum -= self.lu[(row, col)] * rhs.value(col + lower)?;
                }
            } else if sum != 0.0 {
                first = row;
            }
            *rhs.change_value(row + lower)? = sum;
        }
        Ok(())
    }
}
