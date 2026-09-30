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
// Rust translation of the "Split wlines" and "cda002/I3" blocks of
// GeomInt_LineTool::DecompositionOfWLine (ModelingAlgorithms/TKGeomAlgo/GeomInt/
// GeomInt_LineTool.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The new walking lines: the runs off the seams, with their end points, cut by the
//! parts of the line constructor.
use super::decomposition::{Decomposition, Runs};
use crate::geom_int::line_constructor::GeomIntLineConstructor;
use crate::int_patch::{IntPatchLine, IntPatchPointLine, IntPatchWLine};
use crate::int_surf::IntSurfLineOn2S;
use crate::intf::native_int_cast;
use rustycad_tkernel::standard::{Failure, Handle};

/// The run `i` with its end points in the sequence of end points.
struct Run<'a> {
    list: &'a [i32],
    ends: &'a [i32],
    has_first_point: bool,
    has_last_point: bool,
    /// Whether the next run ends the new line (not an empty seam run).
    end_of_line: bool,
}

impl Decomposition<'_> {
    /// The line being assembled becomes a walking line (of the creation way of the
    /// source line) when it has more than one point; a fresh one is started.
    fn flush(
        &self,
        line: &mut Handle<IntSurfLineOn2S>,
        new_lines: &mut Vec<Handle<dyn IntPatchLine>>,
    ) {
        if line.nb_points() > 1 {
            let wline = IntPatchWLine::new_undecided(line, false);
            wline.set_creating_way_info(self.wline.get_creating_way());
            new_lines.push(Handle::new(wline).cast());
        }
        *line = Handle::new(IntSurfLineOn2S::new(None));
    }

    /// Appends the points of the run whose index passes `keep`.
    fn add_run_points(
        &self,
        line: &Handle<IntSurfLineOn2S>,
        run: &Run<'_>,
        keep: impl Fn(i32) -> bool,
    ) -> Result<(), Failure> {
        for &index in run.list {
            if keep(index) {
                line.add(&self.wline.point(index)?);
            }
        }
        Ok(())
    }

    /// The run `i` of the runs off the seams with at least two points, else None.
    fn run<'a>(runs: &'a Runs, ends: &'a [Vec<i32>], i: i32) -> Option<Run<'a>> {
        if runs.is_boundary(i) || runs.line(i).len() < 2 {
            return None;
        }
        let fl: &[i32] = &ends[(i - 1) as usize];
        let mut has_first_point: bool = fl.len() == 2;
        let mut has_last_point: bool = fl.len() == 2;
        if !has_first_point && !fl.is_empty() {
            has_first_point = i != 1;
        }
        if !has_last_point && !fl.is_empty() {
            has_last_point = i != runs.count();
        }
        let next: i32 = i + 1;
        let end_of_line: bool =
            !(next <= runs.count() && runs.is_boundary(next) && runs.line(next).is_empty());
        Some(Run {
            list: runs.line(i),
            ends: fl,
            has_first_point,
            has_last_point,
            end_of_line,
        })
    }

    /// The runs of `runs` cut by each part of the line constructor into new lines.
    pub(super) fn split(
        &self,
        runs: &Runs,
        (ends, seq): (&[Vec<i32>], &Handle<IntSurfLineOn2S>),
        lconstructor: &GeomIntLineConstructor,
        new_lines: &mut Vec<Handle<dyn IntPatchLine>>,
    ) -> Result<(), Failure> {
        for j in 1..=lconstructor.nb_parts()? {
            let (fprm, lprm) = lconstructor.part(j)?;
            let part: (i32, i32) = (native_int_cast(fprm), native_int_cast(lprm));
            let mut line = Handle::new(IntSurfLineOn2S::new(None));
            for i in 1..=runs.count() {
                let Some(run) = Self::run(runs, ends, i) else {
                    continue;
                };
                self.split_run(&run, part, (&mut line, seq), new_lines)?;
            }
            if line.nb_points() > 1 {
                self.flush(&mut line, new_lines);
            }
        }
        Ok(())
    }

    /// One run cut by the part `ifprm..=ilprm`.
    fn split_run(
        &self,
        run: &Run<'_>,
        (ifprm, ilprm): (i32, i32),
        (line, seq): (&mut Handle<IntSurfLineOn2S>, &Handle<IntSurfLineOn2S>),
        new_lines: &mut Vec<Handle<dyn IntPatchLine>>,
    ) -> Result<(), Failure> {
        let (first, last) = (run.list[0], run.list[run.list.len() - 1]);
        let first_inside: bool = ifprm >= first && ifprm <= last;
        let last_inside: bool = ilprm >= first && ilprm <= last;
        if !first_inside && !last_inside {
            if ifprm < first && ilprm > last {
                // The whole run, and its end points when it has them.
                self.add_end(line, seq, run, run.has_first_point, 0)?;
                self.add_run_points(line, run, |_| true)?;
                self.add_end(
                    line,
                    seq,
                    run,
                    run.has_last_point,
                    run.ends.len().wrapping_sub(1),
                )?;
                if run.end_of_line {
                    self.flush(line, new_lines);
                }
            }
            return Ok(());
        }
        if first_inside && last_inside {
            return self.add_run_points(line, run, |k| k >= ifprm && k <= ilprm);
        }
        if first_inside {
            self.add_run_points(line, run, |k| k >= ifprm)?;
            self.add_end(
                line,
                seq,
                run,
                run.has_last_point,
                run.ends.len().wrapping_sub(1),
            )?;
            if run.end_of_line {
                self.flush(line, new_lines);
            }
        }
        if last_inside {
            self.add_end(line, seq, run, run.has_first_point, 0)?;
            self.add_run_points(line, run, |k| k <= ilprm)?;
        }
        Ok(())
    }

    /// Appends the end point of rank `rank` of the run (its first or last one) when
    /// `wanted`.
    fn add_end(
        &self,
        line: &Handle<IntSurfLineOn2S>,
        seq: &Handle<IntSurfLineOn2S>,
        run: &Run<'_>,
        wanted: bool,
        rank: usize,
    ) -> Result<(), Failure> {
        if wanted {
            line.add(&seq.value(run.ends[rank])?);
        }
        Ok(())
    }

    /// The parts of two points whose one end is a run of one point off the seams
    /// become lines of these two points ("cda002/I3").
    pub(super) fn split_short_parts(
        &self,
        runs: &Runs,
        lconstructor: &GeomIntLineConstructor,
        new_lines: &mut Vec<Handle<dyn IntPatchLine>>,
    ) -> Result<(), Failure> {
        for j in 1..=lconstructor.nb_parts()? {
            let (fprm, lprm) = lconstructor.part(j)?;
            let (ifprm, ilprm) = (native_int_cast(fprm), native_int_cast(lprm));
            if ilprm - ifprm != 1 {
                continue;
            }
            for i in 1..=runs.count() {
                let list: &[i32] = runs.line(i);
                if runs.is_boundary(i) || list.len() != 1 || (list[0] != ifprm && list[0] != ilprm)
                {
                    continue;
                }
                let line = Handle::new(IntSurfLineOn2S::new(None));
                line.add(&self.wline.point(ifprm)?);
                line.add(&self.wline.point(ilprm)?);
                let wline = IntPatchWLine::new_undecided(&line, false);
                wline.set_creating_way_info(self.wline.get_creating_way());
                new_lines.push(Handle::new(wline).cast());
            }
        }
        Ok(())
    }
}
