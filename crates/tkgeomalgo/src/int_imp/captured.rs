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

//! The `math_FunctionSetWithDerivatives` side of the IntImp functions.
//!
//! Their tool calls can raise natively, and the exception unwinds through the math
//! solver. Each function keeps the first failure of its callbacks
//! (`rustycad_tkgeombase::failure_capture::FailureCapture`), answers false to the
//! solver, and the IntImp algorithms raise it (`take_failure`) as soon as the solver
//! returns.

/// Implements `FunctionSet` and `FunctionSetWithDerivatives` for a function type
/// with the fallible inherent `value`, `derivatives` and `values`, a `failure`
/// capture and the native `nb_variables`/`nb_equations`.
macro_rules! captured_function_set {
    ([$($generics:tt)*] $function:ty) => {
        impl<$($generics)*> rustycad_tkmath::math::FunctionSet for $function {
            fn variables(&self) -> i32 {
                self.nb_variables()
            }
            fn equations(&self) -> i32 {
                self.nb_equations()
            }
            fn value(
                &mut self,
                x: &rustycad_tkmath::math::Vector<'_>,
                f: &mut rustycad_tkmath::math::Vector<'_>,
            ) -> bool {
                let result = Self::value(self, x, f);
                self.failure.keep(result).unwrap_or(false)
            }
        }
        impl<$($generics)*> rustycad_tkmath::math::FunctionSetWithDerivatives for $function {
            fn derivatives(
                &mut self,
                x: &rustycad_tkmath::math::Vector<'_>,
                d: &mut rustycad_tkmath::math::Matrix<'_>,
            ) -> bool {
                let result = Self::derivatives(self, x, d);
                self.failure.keep(result).unwrap_or(false)
            }
            fn values(
                &mut self,
                x: &rustycad_tkmath::math::Vector<'_>,
                f: &mut rustycad_tkmath::math::Vector<'_>,
                d: &mut rustycad_tkmath::math::Matrix<'_>,
            ) -> bool {
                let result = Self::values(self, x, f, d);
                self.failure.keep(result).unwrap_or(false)
            }
        }
    };
}
pub(crate) use captured_function_set;
