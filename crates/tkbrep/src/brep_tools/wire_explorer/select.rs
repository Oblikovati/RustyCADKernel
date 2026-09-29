// Created on: 1993-01-21
// Created by: Remi LEQUETTE
// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of the static SelectDouble, SelectDegenerated and GetNextParamOnPC
// (ModelingData/TKBRep/BRepTools/BRepTools_WireExplorer.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The auxiliary functions of `BRepTools_WireExplorer::Next`.
use crate::brep::BRepTool;
use crate::topods::{self, Edge, Shape};
use crate::toptools::MapOfShape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkgeombase::scan_step::next_scan_parameter;
use rustycad_tkmath::gp::Point2;

// The native owner named when a step search stalls.
const OWNER: &str = "BRepTools_WireExplorer GetNextParamOnPC";

/// `SelectDouble`: takes out of `list` the first edge met twice in the wire that is not
/// `edge`, which it replaces; false when there is none.
pub(super) fn select_double(
    doubles: &MapOfShape,
    list: &mut Vec<Shape>,
    edge: &mut Edge,
) -> Result<bool, Failure> {
    let found = list
        .iter()
        .position(|candidate| doubles.contains(candidate) && !edge.is_same(candidate));
    let Some(position) = found else {
        return Ok(false);
    };
    *edge = topods::edge(&list.remove(position))?;
    Ok(true)
}

/// `SelectDegenerated`: takes out of `list` the first degenerated edge that is not
/// `edge`. As natively, `edge` is overwritten by every other edge examined, so on a
/// false return it holds the last edge different from the original one.
pub(super) fn select_degenerated(list: &mut Vec<Shape>, edge: &mut Edge) -> Result<bool, Failure> {
    let mut position = 0;
    while position < list.len() {
        if !list[position].is_same(edge) {
            *edge = topods::edge(&list[position])?;
            if BRepTool::degenerated(edge)? {
                list.remove(position);
                return Ok(true);
            }
        }
        position += 1;
    }
    Ok(false)
}

/// The UV tolerances and the direction of `GetNextParamOnPC`.
pub(super) struct StepSearch {
    pub(super) tol_u: f64,
    pub(super) tol_v: f64,
    pub(super) reverse: bool,
}

/// `GetNextParamOnPC`: walking from the first parameter (`reverse`) or the last one in
/// steps of a thousandth of the range, the first parameter whose p-curve point leaves
/// the UV tolerance box around `reference`; the other end of the range when none does.
/// A step that cannot advance (a range of a few ulps) raises Standard_DomainError where
/// the native walk never ends (`rustycad_tkgeombase::scan_step`).
pub(super) fn next_param_on_pcurve(
    pcurve: &dyn Geom2dCurve,
    reference: Point2,
    (first, last): (f64, f64),
    search: &StepSearch,
) -> Result<f64, Failure> {
    let step = (last - first).abs() / 1000.0;
    let is_near = |point: Point2| {
        (reference.x() - point.x()).abs() < search.tol_u
            && (reference.y() - point.y()).abs() < search.tol_v
    };
    if search.reverse {
        let mut start = first;
        while start < last {
            start = next_scan_parameter(OWNER, start, step, [first, last])?;
            if !is_near(pcurve.d0(start)?) {
                return Ok(if start > last { last } else { start });
            }
        }
        return Ok(last);
    }
    let mut start = last;
    while start > first {
        start = next_scan_parameter(OWNER, start, -step, [first, last])?;
        if !is_near(pcurve.d0(start)?) {
            return Ok(if start < first { first } else { start });
        }
    }
    Ok(first)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkg2d::geom2d::Geom2dLine;
    use rustycad_tkmath::gp::Direction2;

    // The X axis, and a search whose tolerance box holds every point of it near 0.
    fn x_axis_search(reverse: bool) -> (Geom2dLine, StepSearch) {
        let line = Geom2dLine::from_point(Point2::new(0., 0.), Direction2::new(1., 0.).unwrap());
        let search = StepSearch {
            tol_u: 10.,
            tol_v: 10.,
            reverse,
        };
        (line, search)
    }

    #[test]
    fn step_search_leaves_the_tolerance_box_as_natively() {
        let (line, mut search) = x_axis_search(true);
        search.tol_u = 0.5;
        let found = next_param_on_pcurve(&line, Point2::new(0., 0.), (0., 1.), &search);
        // The 500th accumulated thousandth, rounded as natively, is the first sample out.
        assert_eq!(found.unwrap(), 0.5000000000000003);
    }

    #[test]
    fn step_search_over_a_one_ulp_range_returns() {
        // Regression: a thousandth of one ulp rounds back, so the native walk
        // `while (T < L) { T += Step; ... }` never ends; both directions must return.
        let range: (f64, f64) = (1., 1. + f64::EPSILON);
        for reverse in [true, false] {
            let (line, search) = x_axis_search(reverse);
            let failure = next_param_on_pcurve(&line, Point2::new(1., 0.), range, &search);
            let message = failure.unwrap_err();
            assert!(
                message.message().contains("never ends"),
                "{}",
                message.message()
            );
        }
    }
}
