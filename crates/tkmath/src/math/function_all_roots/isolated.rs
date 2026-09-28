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

use super::{
    Failure, FunctionAllRoots, FunctionRoots, FunctionSampling, FunctionWithDerivative, Scan,
};
impl FunctionAllRoots {
    pub(super) fn isolate(
        &mut self,
        f: &mut impl FunctionWithDerivative,
        s: &impl FunctionSampling,
        eps: [f64; 3],
        scan: &Scan,
    ) -> Result<(), Failure> {
        if self.intervals.is_empty() {
            return self.append_roots(
                f,
                [s.parameter(1), s.parameter(s.number_of_points())],
                s.number_of_points(),
                eps,
            );
        }
        if !scan.starts_null {
            self.gap(f, s, eps, [s.parameter(1), self.intervals[0][0].parameter])?;
        }
        for index in 1..self.intervals.len() {
            self.gap(
                f,
                s,
                eps,
                [
                    self.intervals[index - 1][1].parameter,
                    self.intervals[index][0].parameter,
                ],
            )?;
        }
        if !scan.ends_null {
            self.gap(
                f,
                s,
                eps,
                [
                    self.intervals.last().unwrap()[1].parameter,
                    s.parameter(s.number_of_points()),
                ],
            )?;
        }
        Ok(())
    }
    fn gap(
        &mut self,
        f: &mut impl FunctionWithDerivative,
        s: &impl FunctionSampling,
        eps: [f64; 3],
        bounds: [f64; 2],
    ) -> Result<(), Failure> {
        let count: i32 =
            (((bounds[1] - bounds[0]) / (s.parameter(s.number_of_points()) - s.parameter(1))).abs()
                * f64::from(s.number_of_points()))
            .trunc() as i32;
        self.append_roots(f, bounds, count.max(3), eps)
    }
    fn append_roots(
        &mut self,
        f: &mut impl FunctionWithDerivative,
        bounds: [f64; 2],
        count: i32,
        eps: [f64; 3],
    ) -> Result<(), Failure> {
        let roots: FunctionRoots = Self::roots(f, bounds, count, eps, 0.0, false)?;
        for index in 1..=roots.number_of_solutions()? {
            self.points.push(Self::root_state(&roots, index)?);
        }
        Ok(())
    }
}
