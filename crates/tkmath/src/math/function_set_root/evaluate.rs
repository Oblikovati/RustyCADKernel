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

use super::{FunctionSetRoot, FunctionSetWithDerivatives};
impl FunctionSetRoot {
    pub(super) fn evaluate(&mut self, function: &mut impl FunctionSetWithDerivatives) -> bool {
        if !function.values(&self.sol, &mut self.residual, &mut self.jacobian) {
            return false;
        }
        for index in 1..=self.residual.length() {
            if self.residual[index] <= -1e100 || self.residual[index] >= 1e100 {
                return false;
            }
        }
        self.run.objective = 0.5 * self.residual.norm2_unchecked();
        self.gradient
            .set_transpose_matrix_vector(&self.jacobian, &self.residual)
            .unwrap();
        for index in 1..=self.gradient.length() {
            if self.gradient[index].abs() >= 1e100 {
                return false;
            }
        }
        self.run.gradient_norm = self.gradient.norm2_unchecked();
        true
    }
    pub(super) fn evaluate_or_finish(
        &mut self,
        function: &mut impl FunctionSetWithDerivatives,
    ) -> bool {
        if !self.run.new_solution {
            return true;
        }
        if self.evaluate(function) {
            return true;
        }
        self.finish(function, false, false);
        false
    }
}
