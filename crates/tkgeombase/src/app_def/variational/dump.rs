// Copyright (c) 1996-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/AppDef/AppDef_Variational.cxx (Dump)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AppDef_Variational::Dump`.
use super::Variational;
use crate::native_text::stream_scientific;
use std::fmt;

/// The integer `o.setf(std::ios::scientific)` returns for a stream with the
/// default flags (`skipws | dec`), which the native Dump prints by mistake.
const DEFAULT_STREAM_FLAGS: i32 = 0x1000 | 0x2;

// `std::setprecision(3) << std::setw(9) << value` in scientific mode.
fn sci9(value: f64) -> String {
    format!("{:>9}", stream_scientific(value, 3))
}

impl Variational {
    /// `Dump(o)`: the problem sizes, the tolerance and weights (in scientific
    /// notation, preceded by the flags `o.setf` returns, printed by the native
    /// chained output: 4098 for a stream with the default flags) and the errors and
    /// criteria when done; `smoothing.dump(&mut text)?`.
    pub fn dump(&self, o: &mut impl fmt::Write) -> fmt::Result {
        let t = &self.tables;
        writeln!(o, " \nVariational Smoothing ")?;
        writeln!(
            o,
            " Number of multipoints                   {}",
            self.nb_points
        )?;
        writeln!(o, " Number of 2d par multipoint {}", self.nb_p2d)?;
        writeln!(o, " Number of 3d per multipoint {}", self.nb_p3d)?;
        writeln!(o, " Number of PassagePoint      {}", t.nb_pass_points)?;
        writeln!(o, " Number of TangencyPoints    {}", t.nb_tang_points)?;
        writeln!(o, " Number of CurvaturePoints   {}", t.nb_curv_points)?;
        write!(
            o,
            " \nTolerance {DEFAULT_STREAM_FLAGS}{}",
            sci9(self.options.tolerance)
        )?;
        let size: &str = if self.options.with_min_max {
            "Max"
        } else {
            "size"
        };
        writeln!(o, "  as {size} Error.")?;
        let [p1, p2, p3] = self.percent.map(|p| stream_scientific(p, 3));
        writeln!(o, "CriteriumWeights : {p1} , {p2} , {p3}")?;
        if !self.is_done {
            let text: &str = if self.is_over_constr {
                " The problem is overconstraint"
            } else {
                " Error in approximation"
            };
            return writeln!(o, "{text}");
        }
        self.dump_results(o)
    }

    fn dump_results(&self, o: &mut impl fmt::Write) -> fmt::Result {
        let r = &self.results;
        writeln!(o, " MaxError             {}", sci9(r.max_error))?;
        writeln!(o, " Index of  MaxError   {}", r.max_error_index)?;
        writeln!(o, " Average Error        {}", sci9(r.average_error))?;
        writeln!(o, " Quadratic Error      {}", sci9(r.criterium[0]))?;
        writeln!(o, " Tension Criterium    {}", sci9(r.criterium[1]))?;
        writeln!(o, " Flexion  Criterium   {}", sci9(r.criterium[2]))?;
        writeln!(o, " Jerk  Criterium      {}", sci9(r.criterium[3]))?;
        writeln!(
            o,
            " NbSegments           {}",
            self.knots.array1().length() - 1
        )
    }
}
