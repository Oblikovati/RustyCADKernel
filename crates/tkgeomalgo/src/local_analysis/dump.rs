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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/LocalAnalysis/LocalAnalysis.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LocalAnalysis::Dump`: the text reports of the continuity analyses. Reals are
//! written as the default `std::ostream` does (`%g`).
use super::{
    LocalAnalysisCurveContinuity, LocalAnalysisStatusErrorType, LocalAnalysisSurfaceContinuity,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::geom_abs::Shape;
use std::io;

/// A real in the default stream format.
fn g(value: f64) -> String {
    String::from_utf8_lossy(AsciiString::from_real(value).as_bytes()).into_owned()
}

/// A query of a done analysis as an I/O result (it cannot fail once done).
fn query<T>(value: Result<T, Failure>) -> io::Result<T> {
    value.map_err(|failure| io::Error::other(failure.message().to_owned()))
}

/// The status line: the first test that holds among the orders, from `levels`.
fn status_line(o: &mut impl io::Write, levels: &[(bool, &str)], fallback: &str) -> io::Result<()> {
    let label: &str = levels
        .iter()
        .find(|(holds, _)| *holds)
        .map_or(fallback, |(_, label)| label);
    writeln!(o, " Continuity Status : {label} ")
}

/// `LocalAnalysis::Dump(SurfaceContinuity)`: the failure reason, or the continuity
/// status and the measured values of the analysed order. A failed analysis without a
/// recorded reason (uninitialized natively) prints only the first line.
/// `dump_surface(&analysis, &mut std::io::stdout())?`.
pub fn dump_surface(
    surfconti: &LocalAnalysisSurfaceContinuity,
    o: &mut impl io::Write,
) -> io::Result<()> {
    if !surfconti.is_done() {
        writeln!(o, "Problem in the computation ")?;
        let reason: Option<&str> = match surfconti.status_error() {
            Some(LocalAnalysisStatusErrorType::NullFirstDerivative) => {
                Some("one of the first derivatives is null")
            }
            Some(LocalAnalysisStatusErrorType::NullSecondDerivative) => {
                Some("one of the  second derivatives is null")
            }
            Some(LocalAnalysisStatusErrorType::NormalNotDefined) => {
                Some("one (or both) normal is undefined")
            }
            Some(LocalAnalysisStatusErrorType::CurvatureNotDefined) => {
                Some("one of the mean curvatures is undefined")
            }
            _ => None,
        };
        if let Some(reason) = reason {
            writeln!(o, "{reason}")?;
        }
        return Ok(());
    }
    let Ok(order) = surfconti.continuity_status() else {
        return Ok(());
    };
    let s = surfconti;
    let c0: bool = query(s.is_c0())?;
    match order {
        Shape::C0 => {
            writeln!(
                o,
                " Continuity Status : {} ",
                if c0 { "C0" } else { "No C0" }
            )?;
            writeln!(o, " C0Value = {}", g(query(s.c0_value())?))
        }
        Shape::C1 => {
            status_line(o, &[(query(s.is_c1())?, "C1"), (c0, "C0")], "NoC0")?;
            write_values(
                o,
                &[
                    ("C0Value", query(s.c0_value())?),
                    ("C1UAngle", query(s.c1u_angle())?),
                    ("C1URatio", query(s.c1u_ratio())?),
                    ("C1VAngle", query(s.c1v_angle())?),
                    ("C1VRatio", query(s.c1v_ratio())?),
                ],
            )
        }
        Shape::C2 => {
            status_line(
                o,
                &[
                    (query(s.is_c2())?, "C2"),
                    (query(s.is_c1())?, "C1"),
                    (c0, "C0"),
                ],
                "NoC0",
            )?;
            write_values(
                o,
                &[
                    ("C0Value", query(s.c0_value())?),
                    ("C1UAngle", query(s.c1u_angle())?),
                    ("C1VAngle", query(s.c1v_angle())?),
                    ("C2UAngle", query(s.c2u_angle())?),
                    ("C2VAngle", query(s.c2v_angle())?),
                    ("C1URatio", query(s.c1u_ratio())?),
                    ("C1VRatio", query(s.c1v_ratio())?),
                    ("C2URatio", query(s.c2u_ratio())?),
                    ("C2VRatio", query(s.c2v_ratio())?),
                ],
            )
        }
        Shape::G1 => {
            status_line(o, &[(query(s.is_g1())?, "G1"), (c0, "G0")], "NoG0")?;
            write_values(
                o,
                &[
                    ("G0Value", query(s.c0_value())?),
                    ("G1Angle", query(s.g1_angle())?),
                ],
            )?;
            writeln!(o)
        }
        Shape::G2 => {
            status_line(
                o,
                &[
                    (query(s.is_g2())?, "G2"),
                    (query(s.is_g1())?, "G1"),
                    (c0, "G0"),
                ],
                "NoG0",
            )?;
            write_values(
                o,
                &[
                    ("G0Value", query(s.c0_value())?),
                    ("G1Value", query(s.g1_angle())?),
                    ("G2CurvatureGap", query(s.g2_curvature_gap())?),
                ],
            )
        }
        _ => Ok(()),
    }
}

fn write_values(o: &mut impl io::Write, values: &[(&str, f64)]) -> io::Result<()> {
    for (name, value) in values {
        writeln!(o, " {name} = {}", g(*value))?;
    }
    Ok(())
}

/// `LocalAnalysis::Dump(CurveContinuity)`: the failure reason, or the continuity
/// status and the measured values of the analysed order.
/// `dump_curve(&analysis, &mut std::io::stdout())?`.
pub fn dump_curve(
    curvconti: &LocalAnalysisCurveContinuity,
    o: &mut impl io::Write,
) -> io::Result<()> {
    if !curvconti.is_done() {
        writeln!(o, "Problem in the computation ")?;
        let reason: Option<&str> = match curvconti.status_error() {
            Some(LocalAnalysisStatusErrorType::NullFirstDerivative) => {
                Some("one (or both) first derivative is null")
            }
            Some(LocalAnalysisStatusErrorType::NullSecondDerivative) => {
                Some("one (or both) second derivative is null")
            }
            Some(LocalAnalysisStatusErrorType::TangentNotDefined) => {
                Some("one (or both) tangent is undefined ")
            }
            Some(LocalAnalysisStatusErrorType::NormalNotDefined) => {
                Some("one (or both) normal is undefined")
            }
            _ => None,
        };
        if let Some(reason) = reason {
            writeln!(o, "{reason}")?;
        }
        return Ok(());
    }
    let c = curvconti;
    let c0: bool = query(c.is_c0())?;
    match query(c.continuity_status())? {
        Shape::C0 => {
            writeln!(
                o,
                " Continuity Status : {} ",
                if c0 { "C0" } else { "No C0" }
            )?;
            writeln!(o, " C0Value = {}", g(query(c.c0_value())?))
        }
        Shape::C1 => {
            status_line(o, &[(query(c.is_c1())?, "C1"), (c0, "C0")], "NoC0")?;
            write_values(
                o,
                &[
                    ("C0Value", query(c.c0_value())?),
                    ("C1Angle", query(c.c1_angle())?),
                    ("C1Ratio", query(c.c1_ratio())?),
                ],
            )
        }
        Shape::C2 => {
            status_line(
                o,
                &[
                    (query(c.is_c2())?, "C2"),
                    (query(c.is_c1())?, "C1"),
                    (c0, "C0"),
                ],
                "NoC0",
            )?;
            write_values(
                o,
                &[
                    ("C0Value", query(c.c0_value())?),
                    ("C1Angle", query(c.c1_angle())?),
                    ("C2Angle", query(c.c2_angle())?),
                    ("C1Ratio", query(c.c1_ratio())?),
                    ("C2Ratio", query(c.c2_ratio())?),
                ],
            )
        }
        Shape::G1 => {
            status_line(o, &[(query(c.is_g1())?, "G1"), (c0, "G0")], "NoG0")?;
            write_values(
                o,
                &[
                    ("G0Value", query(c.c0_value())?),
                    ("G1Angle", query(c.g1_angle())?),
                ],
            )
        }
        Shape::G2 => {
            status_line(
                o,
                &[
                    (query(c.is_g2())?, "G2"),
                    (query(c.is_g1())?, "G1"),
                    (c0, "G0"),
                ],
                "NoG0",
            )?;
            write_values(
                o,
                &[
                    ("G0Value", query(c.c0_value())?),
                    ("G1Angle", query(c.g1_angle())?),
                    ("G2Angle", query(c.g2_angle())?),
                    (
                        "Relative curvature variation",
                        query(c.g2_curvature_variation())?,
                    ),
                ],
            )
        }
        _ => Ok(()),
    }
}
