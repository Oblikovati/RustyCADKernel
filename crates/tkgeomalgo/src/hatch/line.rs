// Copyright (c) 1992-1999 Matra Datavision
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
// Rust translation of Hatch_Line.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Hatch_Line: a hatching line and its sorted intersections.
use super::{LineForm, Parameter};
use rustycad_tkmath::gp::Line2;

/// Hatch_Line: a line stored in the hatcher: its geometry, its form and the
/// intersections found on it, sorted by increasing parameter.
/// ```
/// use rustycad_tkgeomalgo::hatch::{Line, LineForm};
/// use rustycad_tkmath::gp::Line2;
/// let mut line = Line::new(Line2::default(), LineForm::YLine);
/// line.add_intersection(2.0, true, 1, 0.0, 1.0e-7);
/// ```
#[derive(Clone, Default)]
pub struct Line {
    pub(super) lin: Line2,
    pub(super) form: LineForm,
    // NCollection_Sequence<Hatch_Parameter>; the hatcher reads it with the native
    // one-based bound checks (`Hatcher::parameter`).
    pub(super) inters: Vec<Parameter>,
}

impl Line {
    /// Hatch_Line(L, T): the line `lin` of form `form` without intersections;
    /// `Line::new(lin, LineForm::AnyLine)`. The default constructor is `Default`
    /// (the default gp_Lin2d, Hatch_ANYLINE).
    pub fn new(lin: Line2, form: LineForm) -> Self {
        Self {
            lin,
            form,
            inters: Vec::new(),
        }
    }

    /// AddIntersection: insert an intersection in the sorted list. An existing
    /// intersection closer than `tolerance` is removed and the new one dropped
    /// (akm OCC109: two intersections too close cancel each other);
    /// `line.add_intersection(par1, start, index, par2, tolerance)`.
    pub fn add_intersection(
        &mut self,
        par1: f64,
        start: bool,
        index: i32,
        par2: f64,
        tolerance: f64,
    ) {
        let parameter = Parameter::new(par1, start, index, par2);
        // The native loop stops at the first intersection that is too close or
        // lies after `par1`, in that order of tests.
        let stop = self
            .inters
            .iter()
            .position(|existing| (par1 - existing.par1).abs() < tolerance || par1 < existing.par1);
        let Some(position) = stop else {
            self.inters.push(parameter);
            return;
        };
        if (par1 - self.inters[position].par1).abs() < tolerance {
            self.inters.remove(position);
            return;
        }
        self.inters.insert(position, parameter);
    }
}
