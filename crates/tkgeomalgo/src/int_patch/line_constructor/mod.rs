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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_LineConstructor.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_LineConstructor`: splits an intersection line at its vertices and keeps
//! the parts inside both domains (docs/geomalgo-design.md section 18). The branches of
//! `Perform` (analytic and conic, walking, restriction) and the file-static helpers of
//! the native source are split into the submodules.
mod add_line;
mod helpers;
mod perform_curves;
mod perform_rline;
mod perform_walking;
mod rline_coding;
mod wline_rline;

use super::{IntPatchIType, IntPatchLine};
use crate::int_start::TheTopolTool;
use crate::int_start::failures::sequence_value;
use perform_curves::Split;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use wline_rline::Domains;

/// `IntPatch_LineConstructor`: the intersection algorithms compute the lines of two
/// surfaces; this class splits one of them into the parts that lie inside both
/// domains, as new lines.
///
/// ```
/// use rustycad_tkgeomalgo::int_patch::IntPatchLineConstructor;
/// let constructor = IntPatchLineConstructor::new(2);
/// assert_eq!(constructor.nb_lines(), 0);
/// assert!(constructor.line(1).is_err());
/// ```
#[derive(Default)]
pub struct IntPatchLineConstructor {
    slin: Vec<Handle<dyn IntPatchLine>>,
}

impl IntPatchLineConstructor {
    /// `IntPatch_LineConstructor(mode)`: the mode is unused natively.
    /// `IntPatchLineConstructor::new(2)`.
    pub fn new(_mode: i32) -> Self {
        Self::default()
    }

    /// `Perform(SL, L, S1, D1, S2, D2, Tol)`: splits `l` at its vertices; the parts whose
    /// middle point is inside both domains become the lines of the constructor, and the
    /// vertices on both domains that are on no part become lines of one vertex. A walking
    /// part lying on a restriction of `sl` (the lines of the intersection) is coded as a
    /// restriction line or dropped. `tol` is the 3d tolerance of the arcs.
    ///
    /// ```ignore
    /// constructor.perform(&lines, &line, &s1, &*d1, &s2, &*d2, 1e-7)?;
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn perform(
        &mut self,
        sl: &[Handle<dyn IntPatchLine>],
        l: &Handle<dyn IntPatchLine>,
        s1: &Adaptor3dSurfaceHandle,
        d1: &dyn TheTopolTool,
        s2: &Adaptor3dSurfaceHandle,
        d2: &dyn TheTopolTool,
        tol: f64,
    ) -> Result<(), Failure> {
        let domains = Domains { s1, d1, s2, d2 };
        // The surface types are read natively before the dispatch (for AddLine, which
        // ignores them).
        s1.get_type()?;
        s2.get_type()?;
        self.slin.clear();
        let nbvtx: usize = usize::try_from(vertex_count(l)).unwrap_or_default();
        let mut split = Split {
            l,
            tab_index: vec![0; nbvtx + 2],
            numline: 0,
            slin: &mut self.slin,
        };
        match l.arc_type() {
            IntPatchIType::Analytic => perform_curves::perform_aline(&mut split, &domains),
            IntPatchIType::Walking => perform_walking::perform_wline(&mut split, sl, &domains, tol),
            IntPatchIType::Restriction => perform_rline::perform_rline(&mut split, &domains),
            _ => perform_curves::perform_gline(&mut split, &domains),
        }
    }

    /// `NbLines()`. `constructor.nb_lines()`.
    pub fn nb_lines(&self) -> i32 {
        self.slin.len() as i32
    }

    /// `Line(index)`: Standard_OutOfRange outside `1..=NbLines()`. `constructor.line(1)?`.
    pub fn line(&self, index: i32) -> Result<Handle<dyn IntPatchLine>, Failure> {
        sequence_value(&self.slin, index, "IntPatch_LineConstructor::Line").cloned()
    }
}

/// The number of vertices of a line of any kind.
fn vertex_count(l: &Handle<dyn IntPatchLine>) -> i32 {
    use super::{IntPatchALine, IntPatchGLine, IntPatchPointLine, IntPatchRLine, IntPatchWLine};
    match l.arc_type() {
        IntPatchIType::Analytic => l.cast::<IntPatchALine>().nb_vertex(),
        IntPatchIType::Walking => l.cast::<IntPatchWLine>().nb_vertex(),
        IntPatchIType::Restriction => l.cast::<IntPatchRLine>().nb_vertex(),
        _ => l.cast::<IntPatchGLine>().nb_vertex(),
    }
}
