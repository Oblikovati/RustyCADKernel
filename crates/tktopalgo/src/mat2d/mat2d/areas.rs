// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKTopAlgo/MAT2d/MAT2d_Mat2d.cxx (steps 2.5 and 2.6 of CreateMat and CreateMatOpen) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Step 2.5: the edges whose two bisectors meet before meeting their other
//! neighbours form areas to remove; step 2.6 removes them and creates the bisectors
//! of the new neighbours.
use super::Mat2dMat2d;
use super::engine::Run;
use crate::mat::{MatBisector, MatEdge, MatListOfEdge};
use crate::mat2d::Mat2dTool2d;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};

/// The continuous portions of the contour to remove: their first and last edge
/// indices and their sizes, `narea` of them (`compact` edges of the last one are
/// merged into the first when they wrap around).
pub(super) struct Areas {
    pub narea: i32,
    compact: i32,
    firstarea: Vec<i32>,
    lastarea: Vec<i32>,
    noofarea: Vec<i32>,
}

impl Areas {
    // A new area, or one more edge in the current one.
    fn add(&mut self, index: i32) {
        if self.narea >= 0 && index == self.lastarea[self.narea as usize] + 1 {
            self.lastarea[self.narea as usize] += 1;
            self.noofarea[self.narea as usize] += 1;
            return;
        }
        self.narea += 1;
        let k: usize = self.narea as usize;
        self.firstarea[k] = index;
        self.lastarea[k] = index;
        self.noofarea[k] = 1;
    }
}

impl Mat2dMat2d {
    // The side of the intersections on a bisector: `(parama, paramb)` is 1 on the side
    // where the bisector meets its neighbour first, -1 on the other.
    fn param_signs(
        &self,
        atool: &Mat2dTool2d,
        run: &Run,
        bisector: &Handle<MatBisector>,
    ) -> Result<(i32, i32), Failure> {
        let (first, second) = (bisector.first_parameter(), bisector.second_parameter());
        let infinite: f64 = precision::infinite();
        if first == infinite && second == infinite {
            return Ok((-1, -1));
        }
        if first == infinite {
            return Ok((-1, 1));
        }
        if second == infinite {
            return Ok((1, -1));
        }
        if atool.distance(bisector, first, second)? > run.tolerance_of_confusion {
            let before: bool = (first - second) * bisector.sense() > 0.;
            return Ok(if before { (-1, 1) } else { (1, -1) });
        }
        Ok((1, 1))
    }

    /// Step 2.5: the areas of edges to remove.
    pub(super) fn analyse_areas(
        &self,
        atool: &Mat2dTool2d,
        run: &Run,
        noofedges: i32,
    ) -> Result<Areas, Failure> {
        let list: Handle<MatListOfEdge> = self.theedgelist.clone();
        let size: usize = noofedges.max(0) as usize + 1;
        let mut areas = Areas {
            narea: -1,
            compact: 0,
            firstarea: vec![0; size],
            lastarea: vec![0; size],
            noofarea: vec![0; size],
        };
        run.scan_start(&list)?;
        let (_, mut paramb0) = self.param_signs(atool, run, &list.current()?.first_bisector())?;
        for _ in 0..run.scan_count(list.number()) {
            let (parama1, paramb1) =
                self.param_signs(atool, run, &list.current()?.second_bisector())?;
            if paramb0 > 0 && parama1 > 0 {
                areas.add(list.index());
            }
            paramb0 = paramb1;
            list.next()?;
        }
        if areas.narea > 0 {
            let last: usize = areas.narea as usize;
            if areas.lastarea[last] == list.number() && areas.firstarea[0] == 1 {
                areas.firstarea[0] = areas.firstarea[last];
                areas.noofarea[0] += areas.noofarea[last];
                areas.compact = areas.noofarea[last];
                areas.narea -= 1;
            }
        }
        areas.narea += 1;
        Ok(areas)
    }

    /// Step 2.6: the edges of each area leave the contour; a new bisector joins the
    /// edges around the area, its sons the bisectors of the area.
    pub(super) fn eliminate_areas(
        &mut self,
        atool: &mut Mat2dTool2d,
        run: &mut Run,
        areas: &Areas,
    ) -> Result<(), Failure> {
        let list: Handle<MatListOfEdge> = self.theedgelist.clone();
        run.beginbisector = run.noofbisectors;
        let mut shift: i32 = 0;
        let all: bool = areas.narea == 1 && areas.noofarea[0] == list.number();
        for i in 0..areas.narea as usize {
            if i == 1 {
                shift -= areas.compact;
            }
            run.scan_start(&list)?;
            let edgetoremove: Handle<MatEdge> = list.brackets(areas.firstarea[i] - shift)?;
            let bisector: Handle<MatBisector> =
                self.open_area_bisector(atool, run, &list, &edgetoremove)?;
            shift += self.remove_area(
                atool,
                &list,
                &bisector,
                edgetoremove,
                (areas.noofarea[i], all),
            )?;
            self.close_area_bisector(atool, &list, &bisector)?;
            run.noofbisectors += 1;
        }
        Ok(())
    }

    // The first bisector of the area ends at the intersection; the new bisector starts
    // there, from the edge before the area.
    fn open_area_bisector(
        &mut self,
        atool: &mut Mat2dTool2d,
        run: &Run,
        list: &MatListOfEdge,
        edgetoremove: &Handle<MatEdge>,
    ) -> Result<Handle<MatBisector>, Failure> {
        let first_bisector: Handle<MatBisector> = edgetoremove.first_bisector();
        first_bisector.set_end_point(edgetoremove.intersection_point());
        first_bisector.set_first_parameter(first_bisector.second_parameter());
        atool.trim_bisector(&first_bisector)?;
        let bisector: Handle<MatBisector> = Handle::new(MatBisector::new());
        self.bisectormap.insert(run.noofbisectors, bisector.clone());
        bisector.set_index_number(run.noofbisectors);
        bisector.set_dist_issue_point(edgetoremove.distance());
        bisector.set_issue_point(edgetoremove.intersection_point());
        bisector.set_first_edge(list.previous_item()?);
        bisector.add_bisector(&first_bisector);
        Ok(bisector)
    }

    // The edges of the area leave the contour; the bisector after each becomes a son
    // of the new bisector (but the last one when the area is the whole contour). It
    // returns the number of edges removed.
    fn remove_area(
        &mut self,
        atool: &mut Mat2dTool2d,
        list: &MatListOfEdge,
        bisector: &Handle<MatBisector>,
        mut edgetoremove: Handle<MatEdge>,
        (noofarea, all): (i32, bool),
    ) -> Result<i32, Failure> {
        for j in 0..noofarea {
            self.removed_edges_list.back_add(&list.current()?);
            list.unlink()?;
            list.next()?;
            let second: Handle<MatBisector> = edgetoremove.second_bisector();
            if !all || j + 1 != noofarea {
                bisector.add_bisector(&second);
            }
            second.set_end_point(edgetoremove.intersection_point());
            second.set_second_parameter(second.first_parameter());
            atool.trim_bisector(&second)?;
            edgetoremove = list.current()?;
        }
        Ok(noofarea)
    }

    // The new bisector ends the area: it separates the edges before and after it, and
    // its vectors are the tangents at the ends of its first and last sons.
    fn close_area_bisector(
        &self,
        atool: &mut Mat2dTool2d,
        list: &MatListOfEdge,
        bisector: &Handle<MatBisector>,
    ) -> Result<(), Failure> {
        bisector.set_second_edge(list.current()?);
        list.previous_item()?.set_second_bisector(bisector);
        list.current()?.set_first_bisector(bisector);
        bisector.set_first_vector(atool.tangent(bisector.first_bisector()?.bisector_number())?);
        bisector.set_second_vector(atool.tangent(bisector.last_bisector()?.bisector_number())?);
        list.previous_item()?.set_distance(-1.);
        list.current()?.set_distance(-1.);
        list.previous_item()?
            .first_bisector()
            .set_second_parameter(precision::infinite());
        list.current()?
            .second_bisector()
            .set_first_parameter(precision::infinite());
        Ok(())
    }
}
