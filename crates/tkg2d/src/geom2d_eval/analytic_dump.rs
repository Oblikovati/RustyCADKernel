// Copyright (c) 2025 OPEN CASCADE SAS
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
// Rust translation of the DumpJson members of ModelingData/TKG2d/Geom2dEval (spirals, involute, sine wave) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `DumpJson` shape shared by the four analytic eval curves: the class name,
//! the Geom2d_Curve base, `&myPosition`, then the numeric parameters in order.
use crate::geom2d::{dump_base, dump_class_name, dump_curve, dump_field};
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkmath::gp::Axis2d;
use std::io;

/// Write `className`, the curve base, `"Position"` and each `(name, value)` field;
/// `dump_analytic_curve(&mut s, -1, "Geom2dEval_SineWaveCurve", axis, &[("Amplitude", 2.0)])?`.
pub(crate) fn dump_analytic_curve(
    stream: &mut DumpStream,
    depth: i32,
    class_name: &str,
    position: Axis2d,
    fields: &[(&str, f64)],
) -> io::Result<()> {
    dump_class_name(stream, class_name)?;
    dump_base(stream, depth, dump_curve)?;
    dump_field(stream, depth, "Position", |nested, level| {
        position.dump_json(nested, level)
    })?;
    for &(name, value) in fields {
        dump::dump_real(stream, name, value)?;
    }
    Ok(())
}
