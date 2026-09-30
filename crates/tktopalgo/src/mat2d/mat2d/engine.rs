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
// Rust translation of ModelingAlgorithms/TKTopAlgo/MAT2d/MAT2d_Mat2d.cxx (CreateMat and CreateMatOpen: the initialisation and the main loop) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The main loop of the MAT algorithm, shared by the closed and the open circuits
//! (docs/topalgo-design.md section 3):
//! 0. while there is more than one edge, the bisectors between neighbour edges are
//!    created, 1. intersected (the bisectors that the intersections cancel are
//!    removed), 2. the edges whose two bisectors meet first are removed, which gives
//!    the bisectors of the new neighbours; 3. the roots of the trees.
use super::Mat2dMat2d;
use crate::mat::{MatBisector, MatEdge, MatListOfEdge};
use crate::mat2d::Mat2dTool2d;
use rustycad_tkernel::standard::{Failure, Handle};

/// Which of the two native members runs: `CreateMat` (a closed circuit, the edges
/// form a ring) or `CreateMatOpen` (the first and last edges end the circuit).
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum MatContour {
    Closed,
    Open,
}

/// The local variables of the native members that live across the iterations.
pub(super) struct Run {
    pub contour: MatContour,
    pub noofbisectors: i32,
    pub beginbisector: i32,
    pub noofbisectorstoremove: i32,
    // Written by IntersectBisector only when it finds a point: it keeps its previous
    // value otherwise (natively uninitialised before the first point, 0 here).
    pub intersectionpoint: i32,
    pub nb_iter_bis: i32,
    pub edge_numbers: [i32; 12],
    pub to_nullify: bool,
    pub tolerance_of_confusion: f64,
    pub interrupt: bool,
    pub patch: LoopPatch,
}

/// The patch of `CreateMat` against an infinite loop on bad geometry: ten
/// iterations in a row that create one bisector between the same edges stop it.
#[derive(Default)]
pub(super) struct LoopPatch {
    nb_of_narea1: i32,
    pref_narea: i32,
    nb_elts: [i32; 2],
    count_elts: [i32; 2],
    pub is_break: bool,
}

impl Run {
    /// `theedgelist->First()`, then `Next()` for an open circuit (whose first edge
    /// has no bisector before it).
    pub(super) fn scan_start(&self, list: &MatListOfEdge) -> Result<(), Failure> {
        list.first();
        if self.contour == MatContour::Open {
            list.next()?;
        }
        Ok(())
    }
    /// The number of edges a scan visits: every edge of a ring, all but the ends of an
    /// open circuit.
    pub(super) fn scan_count(&self, number: i32) -> i32 {
        match self.contour {
            MatContour::Closed => number,
            MatContour::Open => number - 2,
        }
    }
}

impl Mat2dMat2d {
    /// `CreateMat(aTool)`: the bisecting locus of a closed circuit;
    /// `mat.create_mat(&mut tool)?`.
    pub fn create_mat(&mut self, atool: &mut Mat2dTool2d) -> Result<(), Failure> {
        self.run_mat(atool, MatContour::Closed)
    }
    /// `CreateMatOpen(aTool)`: the bisecting locus of an open circuit (an open result:
    /// the first and last items are not neighbours); `mat.create_mat_open(&mut tool)?`.
    pub fn create_mat_open(&mut self, atool: &mut Mat2dTool2d) -> Result<(), Failure> {
        self.run_mat(atool, MatContour::Open)
    }

    fn run_mat(&mut self, atool: &mut Mat2dTool2d, contour: MatContour) -> Result<(), Failure> {
        let noofedges: i32 = atool.number_of_items()?;
        let mut run = Run {
            contour,
            noofbisectors: if contour == MatContour::Open {
                noofedges - 1
            } else {
                noofedges
            },
            beginbisector: 0,
            noofbisectorstoremove: 0,
            intersectionpoint: 0,
            nb_iter_bis: 0,
            edge_numbers: [-1; 12],
            to_nullify: false,
            tolerance_of_confusion: atool.tolerance_of_confusion(),
            interrupt: false,
            patch: LoopPatch::default(),
        };
        self.bisectoronetoremove.clear();
        self.bisectortwotoremove.clear();
        self.typeofbisectortoremove.clear();
        self.bisectormap.clear();
        self.is_done = true;
        self.init_edges(atool, &run, noofedges)?;
        self.main_loop(atool, &mut run, noofedges)?;
        self.make_roots(atool, &mut run)
    }

    // The edge list (a ring of one edge per item), the bisectors between neighbour
    // edges and the bisectors of each edge.
    fn init_edges(
        &mut self,
        atool: &mut Mat2dTool2d,
        run: &Run,
        noofedges: i32,
    ) -> Result<(), Failure> {
        let list: Handle<MatListOfEdge> = Handle::new(MatListOfEdge::new());
        self.theedgelist = list.clone();
        self.removed_edges_list = Handle::new(MatListOfEdge::new());
        for i in 0..noofedges {
            let edge: Handle<MatEdge> = Handle::new(MatEdge::new());
            edge.set_edge_number(i + 1);
            edge.set_distance(-1.);
            list.back_add(&edge);
        }
        list.close_loop()?;
        list.first();
        let count: i32 = if run.contour == MatContour::Open {
            list.number() - 1
        } else {
            list.number()
        };
        for i in 0..count {
            self.init_bisector(atool, &list, i)?;
        }
        self.assign_bisectors(&list, run)
    }

    // The bisector `i` between the current edge and the next one (the list moves on).
    fn init_bisector(
        &mut self,
        atool: &mut Mat2dTool2d,
        list: &MatListOfEdge,
        i: i32,
    ) -> Result<(), Failure> {
        let bisector: Handle<MatBisector> = Handle::new(MatBisector::new());
        self.bisectormap.insert(i, bisector.clone());
        bisector.set_index_number(i);
        let first: Handle<MatEdge> = list.current()?;
        bisector.set_first_edge(first.clone());
        bisector
            .set_first_vector(atool.tangent_before(first.edge_number(), self.my_is_open_result)?);
        list.next()?;
        let second: Handle<MatEdge> = list.current()?;
        bisector.set_second_edge(second.clone());
        let (issue_point, dist) = atool.first_point(second.edge_number())?;
        bisector.set_issue_point(issue_point);
        bisector.set_dist_issue_point(dist);
        bisector
            .set_second_vector(atool.tangent_after(second.edge_number(), self.my_is_open_result)?);
        Ok(())
    }

    // Each edge takes the bisectors before and after it; the ends of an open circuit
    // take their only bisector twice.
    fn assign_bisectors(&self, list: &MatListOfEdge, run: &Run) -> Result<(), Failure> {
        list.first();
        let number: i32 = list.number();
        if run.contour == MatContour::Closed {
            for i in 0..number {
                let edge: Handle<MatEdge> = list.current()?;
                edge.set_first_bisector(&super::bound(
                    &self.bisectormap,
                    (i - 1 + run.noofbisectors) % run.noofbisectors,
                    "bisector",
                )?);
                edge.set_second_bisector(&super::bound(&self.bisectormap, i, "bisector")?);
                list.next()?;
            }
            return Ok(());
        }
        let first: Handle<MatBisector> = super::bound(&self.bisectormap, 0, "bisector")?;
        list.current()?.set_first_bisector(&first);
        list.current()?.set_second_bisector(&first);
        list.next()?;
        for i in 1..number - 1 {
            let edge: Handle<MatEdge> = list.current()?;
            edge.set_first_bisector(&super::bound(&self.bisectormap, i - 1, "bisector")?);
            edge.set_second_bisector(&super::bound(&self.bisectormap, i, "bisector")?);
            list.next()?;
        }
        let last: Handle<MatBisector> = super::bound(&self.bisectormap, number - 2, "bisector")?;
        list.current()?.set_first_bisector(&last);
        list.current()?.set_second_bisector(&last);
        Ok(())
    }

    // The main loop (step 2), until one edge remains, no edge is removed, or too many
    // iterations were made (the computation then fails).
    fn main_loop(
        &mut self,
        atool: &mut Mat2dTool2d,
        run: &mut Run,
        noofedges: i32,
    ) -> Result<(), Failure> {
        let list: Handle<MatListOfEdge> = self.theedgelist.clone();
        let number_max_of_ite: i32 = noofedges * noofedges;
        let mut number_of_ite: i32 = 0;
        while list.number() > 1 {
            let a_nb_bis: i32 = run.noofbisectors - run.beginbisector;
            for i in run.beginbisector..run.noofbisectors {
                atool.create_bisector(&super::bound(&self.bisectormap, i, "bisector")?)?;
                self.thenumberofbisectors += 1;
            }
            if run.contour == MatContour::Closed {
                self.update_patch(run, a_nb_bis)?;
            }
            if list.number() < 3 {
                break;
            }
            self.remove_loop(atool, run)?;
            let mut areas = self.analyse_areas(atool, run, noofedges)?;
            if run.contour == MatContour::Closed && areas.narea == 1 && run.patch.is_break {
                areas.narea = 0;
            }
            if areas.narea == 0 {
                run.interrupt = true;
                break;
            }
            self.eliminate_areas(atool, run, &areas)?;
            if number_of_ite > number_max_of_ite {
                self.is_done = false;
                break;
            }
            number_of_ite += 1;
        }
        Ok(())
    }

    // The patch of CreateMat that prevents an infinite loop on bad geometry.
    fn update_patch(&self, run: &mut Run, a_nb_bis: i32) -> Result<(), Failure> {
        let max_narea1: i32 = 10;
        let patch: &mut LoopPatch = &mut run.patch;
        if a_nb_bis == 1 {
            if patch.pref_narea == 1 {
                patch.nb_of_narea1 += 1;
                let bisector: Handle<MatBisector> =
                    super::bound(&self.bisectormap, run.beginbisector, "bisector")?;
                let edges = [bisector.first_edge(), bisector.second_edge()];
                for (k, edge) in edges.iter().enumerate() {
                    let number: i32 = crate::mat2d::mat_edge_number(edge)?;
                    if patch.nb_elts[k] == number {
                        patch.count_elts[k] += 1;
                    } else {
                        patch.count_elts[k] = 0;
                        patch.nb_elts[k] = number;
                    }
                }
                if patch.nb_of_narea1 >= max_narea1
                    && (patch.count_elts[0] >= max_narea1 || patch.count_elts[1] >= max_narea1)
                {
                    patch.is_break = true;
                }
            } else {
                patch.nb_of_narea1 = 0;
                patch.count_elts = [0, 0];
            }
        }
        patch.pref_narea = a_nb_bis;
        Ok(())
    }
}
