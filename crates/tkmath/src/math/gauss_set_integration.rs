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

//! math_GaussSetIntegration: Gauss quadrature of a vector-valued function of one variable.
use super::{FunctionSet, IntegerVector, Vector, gauss_points, gauss_points_max, gauss_weights};
use rustycad_tkernel::standard::Failure;
use std::fmt;

/// Integral of every equation of a one-variable function set; `GaussSetIntegration::new(&mut f,&lower,&upper,&order)?`.
pub struct GaussSetIntegration {
    value: Vector<'static>,
    done: bool,
}
impl GaussSetIntegration {
    /// Integrates between the first entries of `lower` and `upper` with the Gauss order in
    /// the first entry of `order`.
    ///
    /// More than one variable, or an order above `gauss_points_max()`, raises the native
    /// `Standard_NotImplemented`. A callback failure leaves the result not done.
    /// `GaussSetIntegration::new(&mut f,&lower,&upper,&order)?`.
    pub fn new(
        function: &mut impl FunctionSet,
        lower: &Vector<'_>,
        upper: &Vector<'_>,
        order: &IntegerVector<'_>,
    ) -> Result<Self, Failure> {
        let mut result = Self {
            value: Vector::with_bounds(1, function.equations())?,
            done: false,
        };
        // The native `||` reads the order only for single-variable functions.
        let degree = match function.variables() {
            1 => *order.value(order.lower())?,
            _ => 0,
        };
        if function.variables() != 1 || degree > gauss_points_max() {
            return Err(Failure::with_type(
                "Standard_NotImplemented",
                &format!(
                    "GaussSetIntegration: {} variables and order {degree}; expected 1 variable and order <= {}",
                    function.variables(),
                    gauss_points_max()
                ),
                "",
            ));
        }
        let bounds = [*lower.value(lower.lower())?, *upper.value(upper.lower())?];
        result.done = result.integrate(function, bounds, degree)?;
        Ok(result)
    }
    fn integrate(
        &mut self,
        function: &mut impl FunctionSet,
        [first, last]: [f64; 2],
        degree: i32,
    ) -> Result<bool, Failure> {
        let equations = function.equations();
        let (mut plus, mut minus) = (
            Vector::with_bounds(1, equations)?,
            Vector::with_bounds(1, equations)?,
        );
        let mut parameter = Vector::with_bounds(1, 1)?;
        let (mut points, mut weights) = (
            Vector::with_bounds(1, degree)?,
            Vector::with_bounds(1, degree)?,
        );
        gauss_points(degree, &mut points);
        gauss_weights(degree, &mut weights);
        let middle = 0.5 * (first + last);
        let radius = 0.5 * (last - first);
        let (half, odd_half) = (degree / 2, (degree + 1) / 2);
        if odd_half > half {
            // The odd rule's centre node; the native comment keeps `+ Xr * GaussP(ind1)` out.
            parameter[1] = middle;
            if !function.value(&parameter, &mut self.value) {
                return Ok(false);
            }
            self.value.multiply(weights[odd_half]);
        } else {
            self.value.init(0.0);
        }
        for i in 1..=half {
            parameter[1] = middle + radius * points[i];
            if !function.value(&parameter, &mut plus) {
                return Ok(false);
            }
            parameter[1] = middle - radius * points[i];
            if !function.value(&parameter, &mut minus) {
                return Ok(false);
            }
            plus.add(&minus)?;
            plus.multiply(weights[i]);
            self.value.add(&plus)?;
        }
        self.value.multiply(radius);
        Ok(true)
    }
    /// Reports successful callbacks; `integral.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// Borrows the integral of each equation; `integral.value()?`.
    pub fn value(&self) -> Result<&Vector<'static>, Failure> {
        if !self.done {
            return Err(Failure::with_type("StdFail_NotDone", "Integration ", ""));
        }
        Ok(&self.value)
    }
    /// Writes the native status text with the value vector's dump; `integral.dump(&mut text)?`.
    pub fn dump(&self, stream: &mut impl fmt::Write) -> fmt::Result {
        write!(stream, "math_GaussSetIntegration ")?;
        if !self.done {
            return writeln!(stream, "Status = not Done ");
        }
        writeln!(stream, " Status = Done ")?;
        write!(stream, "Integration Value = ")?;
        self.value.dump(stream)?;
        writeln!(stream)
    }
}
