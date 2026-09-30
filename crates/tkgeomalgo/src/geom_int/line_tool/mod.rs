// Created on: 1995-02-08
// Created by: Jacques GOUSSARD
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomInt/GeomInt_LineTool.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomInt_LineTool`: the vertices and the parameter range of an intersection line of
//! any kind, and the decomposition of a walking line at the seams of periodic surfaces
//! (`decomposition`, with its boundary helpers in `boundary` and the projector in
//! `project`).
mod boundary;
mod decomposition;
mod line_ends;
mod project;
mod split;

use crate::int_patch::{
    IntPatchALine, IntPatchGLine, IntPatchIType, IntPatchLine, IntPatchPoint, IntPatchPointLine,
    IntPatchRLine, IntPatchWLine,
};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use std::f64::consts::PI;

/// `GeomInt_LineTool`: static members only (docs/geomalgo-design.md section 20).
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkgeomalgo::geom_int::GeomIntLineTool;
/// use rustycad_tkgeomalgo::int_patch::{IntPatchLine, IntPatchRLine};
/// let line: Handle<dyn IntPatchLine> = Handle::new(IntPatchRLine::new_undecided(false)).cast();
/// assert_eq!(GeomIntLineTool::nb_vertex(&line)?, 0);
/// assert!(GeomIntLineTool::first_parameter(&line)? < -1.0e99);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct GeomIntLineTool;

/// The native `occ::down_cast<T>(L)` that the code dereferences: Standard_NullObject for
/// a line of another class, which crashes natively.
pub(crate) fn line_as<T: RuntimeTyped + ?Sized>(
    l: &Handle<dyn IntPatchLine>,
    member: &str,
) -> Result<Handle<T>, Failure> {
    let cast: Handle<T> = l.cast::<T>();
    if cast.is_null() {
        return Err(Failure::with_message(
            "Standard_NullObject",
            &format!("{member}() - the line is not a {}", T::TYPE_NAME),
        ));
    }
    Ok(cast)
}

impl GeomIntLineTool {
    /// `NbVertex(L)`: the number of vertices of the line;
    /// `GeomIntLineTool::nb_vertex(&line)?`.
    pub fn nb_vertex(l: &Handle<dyn IntPatchLine>) -> Result<i32, Failure> {
        const MEMBER: &str = "GeomInt_LineTool::NbVertex";
        Ok(match l.arc_type() {
            IntPatchIType::Analytic => line_as::<IntPatchALine>(l, MEMBER)?.nb_vertex(),
            IntPatchIType::Restriction => line_as::<IntPatchRLine>(l, MEMBER)?.nb_vertex(),
            IntPatchIType::Walking => line_as::<IntPatchWLine>(l, MEMBER)?.nb_vertex(),
            _ => line_as::<IntPatchGLine>(l, MEMBER)?.nb_vertex(),
        })
    }

    /// `Vertex(L, I)`: a copy of the vertex `i` of the line (Standard_OutOfRange outside
    /// `1..=NbVertex`); `GeomIntLineTool::vertex(&line, 1)?`.
    pub fn vertex(l: &Handle<dyn IntPatchLine>, i: i32) -> Result<IntPatchPoint, Failure> {
        const MEMBER: &str = "GeomInt_LineTool::Vertex";
        match l.arc_type() {
            IntPatchIType::Analytic => line_as::<IntPatchALine>(l, MEMBER)?.vertex(i),
            IntPatchIType::Restriction => line_as::<IntPatchRLine>(l, MEMBER)?.vertex(i),
            IntPatchIType::Walking => line_as::<IntPatchWLine>(l, MEMBER)?.vertex(i),
            _ => line_as::<IntPatchGLine>(l, MEMBER)?.vertex(i),
        }
    }

    /// `FirstParameter(L)`: the parameter of the first point, else the start of the
    /// line (just inside it when it is excluded, 1 for a walking line, minus infinity
    /// for an open conic or a restriction, 0 for a closed conic);
    /// `GeomIntLineTool::first_parameter(&line)?`.
    pub fn first_parameter(l: &Handle<dyn IntPatchLine>) -> Result<f64, Failure> {
        end_parameter(l, End::First)
    }

    /// `LastParameter(L)`: the parameter of the last point, else the end of the line
    /// (just inside it when it is excluded, the number of points of a walking line,
    /// infinity for an open conic or a restriction, 2 Pi for a closed conic);
    /// `GeomIntLineTool::last_parameter(&line)?`.
    pub fn last_parameter(l: &Handle<dyn IntPatchLine>) -> Result<f64, Failure> {
        end_parameter(l, End::Last)
    }
}

/// The end of a line that `FirstParameter` or `LastParameter` reads.
#[derive(Clone, Copy, PartialEq, Eq)]
enum End {
    First,
    Last,
}

/// The parameter of the vertex at the end of the line, when it has one.
fn end_point(l: &Handle<dyn IntPatchLine>, end: End, member: &str) -> Result<Option<f64>, Failure> {
    let point = match l.arc_type() {
        IntPatchIType::Analytic => {
            let a = line_as::<IntPatchALine>(l, member)?;
            let has: bool = if end == End::First {
                a.has_first_point()
            } else {
                a.has_last_point()
            };
            has.then(|| {
                if end == End::First {
                    a.first_point()
                } else {
                    a.last_point()
                }
            })
        }
        IntPatchIType::Restriction => {
            let r = line_as::<IntPatchRLine>(l, member)?;
            let has: bool = if end == End::First {
                r.has_first_point()
            } else {
                r.has_last_point()
            };
            has.then(|| {
                if end == End::First {
                    r.first_point()
                } else {
                    r.last_point()
                }
            })
        }
        IntPatchIType::Walking => {
            let w = line_as::<IntPatchWLine>(l, member)?;
            let has: bool = if end == End::First {
                w.has_first_point()
            } else {
                w.has_last_point()
            };
            has.then(|| {
                if end == End::First {
                    w.first_point()
                } else {
                    w.last_point()
                }
            })
        }
        _ => {
            let g = line_as::<IntPatchGLine>(l, member)?;
            let has: bool = if end == End::First {
                g.has_first_point()
            } else {
                g.has_last_point()
            };
            has.then(|| {
                if end == End::First {
                    g.first_point()
                } else {
                    g.last_point()
                }
            })
        }
    };
    point.map(|p| p.map(|v| v.parameter_on_line())).transpose()
}

/// `FirstParameter(L)` and `LastParameter(L)`.
fn end_parameter(l: &Handle<dyn IntPatchLine>, end: End) -> Result<f64, Failure> {
    let member: &str = if end == End::First {
        "GeomInt_LineTool::FirstParameter"
    } else {
        "GeomInt_LineTool::LastParameter"
    };
    if let Some(prm) = end_point(l, end, member)? {
        return Ok(prm);
    }
    let sign: f64 = if end == End::First { -1.0 } else { 1.0 };
    let typl: IntPatchIType = l.arc_type();
    Ok(match typl {
        IntPatchIType::Analytic => {
            let a = line_as::<IntPatchALine>(l, member)?;
            let (prm, included) = if end == End::First {
                a.first_parameter()?
            } else {
                a.last_parameter()?
            };
            // An excluded end is moved just inside the line.
            if included {
                prm
            } else {
                prm - sign * epsilon(prm)
            }
        }
        // It depends on the kind of the 2d line.
        IntPatchIType::Restriction => sign * precision::infinite(),
        IntPatchIType::Walking if end == End::First => 1.0,
        IntPatchIType::Walking => f64::from(line_as::<IntPatchWLine>(l, member)?.nb_pnts()?),
        IntPatchIType::Lin | IntPatchIType::Parabola | IntPatchIType::Hyperbola => {
            sign * precision::infinite()
        }
        IntPatchIType::Circle | IntPatchIType::Ellipse if end == End::Last => 2.0 * PI,
        _ => 0.0,
    })
}
