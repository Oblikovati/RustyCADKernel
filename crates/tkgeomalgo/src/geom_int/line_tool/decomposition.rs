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
// Rust translation of GeomInt_LineTool::DecompositionOfWLine (the grouping of the points)
// of ModelingAlgorithms/TKGeomAlgo/GeomInt/GeomInt_LineTool.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DecompositionOfWLine`: the points of a walking line grouped into runs on and off
//! the seams of the periodic surfaces; the runs off the seams get end points on the
//! seams (`line_ends`) and are cut by the parts of the line constructor (`split`).
use super::GeomIntLineTool;
use super::boundary::{adjust_periodic, is_point_on_boundary};
use super::project::ProjectPointOnSurf;
use crate::geom_int::line_constructor::GeomIntLineConstructor;
use crate::int_patch::{IntPatchLine, IntPatchPointLine, IntPatchWLine};
use crate::int_surf::IntSurfPntOn2S;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;

/// The runs of points: `anArrayOfLines` (the point indices of each run, 1-based) and
/// `anArrayOfLineType` (whether the run is on a seam).
pub(super) struct Runs {
    pub(super) lines: Vec<Vec<i32>>,
    pub(super) on_boundary: Vec<bool>,
}

impl Runs {
    /// The number of runs (`nblines`).
    pub(super) fn count(&self) -> i32 {
        self.lines.len() as i32
    }
    /// The points of the run `i` (1-based).
    pub(super) fn line(&self, i: i32) -> &[i32] {
        &self.lines[(i - 1) as usize]
    }
    /// Whether the run `i` is on a seam.
    pub(super) fn is_boundary(&self, i: i32) -> bool {
        self.on_boundary[(i - 1) as usize]
    }
}

/// The inputs of the decomposition shared by its steps.
pub(super) struct Decomposition<'a> {
    pub(super) wline: &'a IntPatchWLine,
    pub(super) surfaces: [&'a GeomAdaptorSurface; 2],
    pub(super) tol_sum: f64,
    pub(super) projectors: [ProjectPointOnSurf; 2],
}

impl Decomposition<'_> {
    /// Whether the point is on a seam (the adjusted periodic parameter within the
    /// resolution of a bound of the surface) of the first or the second surface.
    fn is_on_boundary(&self, point: &IntSurfPntOn2S) -> Result<bool, Failure> {
        let tol: f64 = precision::confusion();
        for (i, surface) in self.surfaces.iter().enumerate() {
            let (umin, umax, vmin, vmax) = surface.surface().bounds();
            let (u, v) = point.parameters_on_surface(i == 0);
            for j in 0..2 {
                let periodic: bool = if j == 0 {
                    surface.is_u_periodic()?
                } else {
                    surface.is_v_periodic()?
                };
                if !periodic {
                    continue;
                }
                let resolution: f64 = if j == 0 {
                    surface.u_resolution(tol)?
                } else {
                    surface.v_resolution(tol)?
                };
                let period: f64 = if j == 0 {
                    surface.u_period()?
                } else {
                    surface.v_period()?
                };
                let (lower, upper, parameter) = if j == 0 {
                    (umin, umax, u)
                } else {
                    (vmin, vmax, v)
                };
                let (adjusted, _) = adjust_periodic(parameter, lower, upper, period);
                if is_point_on_boundary(adjusted, lower, upper, resolution).is_some() {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// The runs of points on and off the seams.
    fn runs(&self, nb_pnts: i32) -> Result<Runs, Failure> {
        let mut runs = Runs {
            lines: Vec::new(),
            on_boundary: Vec::new(),
        };
        let mut list: Vec<i32> = Vec::new();
        let mut prev_on_boundary: bool = false;
        for pit in 1..=nb_pnts {
            let current: bool = self.is_on_boundary(&self.wline.point(pit)?)?;
            if current != prev_on_boundary {
                if !list.is_empty() {
                    runs.lines.push(std::mem::take(&mut list));
                    runs.on_boundary.push(prev_on_boundary);
                }
                prev_on_boundary = current;
            }
            list.push(pit);
        }
        if !list.is_empty() {
            runs.lines.push(list);
            runs.on_boundary.push(prev_on_boundary);
        }
        Ok(runs)
    }
}

impl GeomIntLineTool {
    /// `DecompositionOfWLine(theWLine, theSurface1, theSurface2, aTolSum,
    /// theLConstructor, theNewLines)`: when the walking line crosses the seams of the
    /// periodic surfaces, its runs of points off the seams, extended to the seams and
    /// cut by the parts of the line constructor, are appended to `new_lines` as walking
    /// lines; false when there is nothing to decompose;
    /// `GeomIntLineTool::decomposition_of_wline(&wl, &s1, &s2, 2e-7, &lc, &mut lines)?`.
    pub fn decomposition_of_wline(
        wline: &Handle<IntPatchWLine>,
        surface1: &Handle<GeomAdaptorSurface>,
        surface2: &Handle<GeomAdaptorSurface>,
        tol_sum: f64,
        lconstructor: &GeomIntLineConstructor,
        new_lines: &mut Vec<Handle<dyn IntPatchLine>>,
    ) -> Result<bool, Failure> {
        let nb_parts: i32 = lconstructor.nb_parts()?;
        let nb_pnts: i32 = wline.nb_pnts()?;
        if nb_pnts == 0 || nb_parts == 0 {
            return Ok(false);
        }
        let mut dec = Decomposition {
            wline,
            surfaces: [surface1, surface2],
            tol_sum,
            projectors: [ProjectPointOnSurf::default(), ProjectPointOnSurf::default()],
        };
        for k in 0..2 {
            let surface = dec.surfaces[k].surface();
            dec.projectors[k].init(&surface, surface.bounds())?;
        }
        let runs: Runs = dec.runs(nb_pnts)?;
        if runs.count() <= 1 {
            return Ok(false);
        }
        let (ends, seq) = dec.line_ends(&runs)?;
        dec.split(&runs, (&ends, &seq), lconstructor, new_lines)?;
        dec.split_short_parts(&runs, lconstructor, new_lines)?;
        Ok(true)
    }
}
