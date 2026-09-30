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
// Rust translation of the vertex members shared by IntPatch_WLine, IntPatch_RLine,
// IntPatch_GLine and IntPatch_ALine (.hxx/.lxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The vertices of a line and its first and last vertex indices (`svtx`, `fipt`,
//! `lapt`, `indf`, `indl`), which the four vertex-holding lines declare each with the
//! same inline members.
use super::failures::{domain_error, sequence_exchange, sequence_remove, sequence_value_mut};
use super::point::IntPatchPoint;
use crate::int_start::failures::sequence_value;
use rustycad_tkernel::standard::Failure;

/// The vertex sequence of a line with its first and last point indices.
#[derive(Clone, Default)]
pub(crate) struct VertexList {
    pub(crate) svtx: Vec<IntPatchPoint>,
    pub(crate) fipt: bool,
    pub(crate) lapt: bool,
    // Uninitialized natively in IntPatch_WLine (0 in the other lines); only read when
    // the matching flag is set, which also sets the index.
    pub(crate) indf: i32,
    pub(crate) indl: i32,
}

impl VertexList {
    /// `NbVertex()`.
    pub(crate) fn nb_vertex(&self) -> i32 {
        self.svtx.len() as i32
    }

    /// `svtx(Index)`: Standard_OutOfRange outside `1..=NbVertex()`.
    pub(crate) fn value(&self, index: i32, member: &str) -> Result<&IntPatchPoint, Failure> {
        sequence_value(&self.svtx, index, member)
    }

    /// `svtx.ChangeValue(Index)`.
    pub(crate) fn value_mut(
        &mut self,
        index: i32,
        member: &str,
    ) -> Result<&mut IntPatchPoint, Failure> {
        sequence_value_mut(&mut self.svtx, index, member)
    }

    /// `svtx.Remove(Index)`.
    pub(crate) fn remove(&mut self, index: i32, member: &str) -> Result<(), Failure> {
        sequence_remove(&mut self.svtx, index, member).map(|_| ())
    }

    /// `svtx.Exchange(I, J)`.
    pub(crate) fn exchange(
        &mut self,
        first: i32,
        second: i32,
        member: &str,
    ) -> Result<(), Failure> {
        sequence_exchange(&mut self.svtx, first, second, member)
    }

    /// `AddVertex(Pnt, theIsPrepend)` of the point lines.
    pub(crate) fn add(&mut self, pnt: &IntPatchPoint, prepend: bool) {
        if prepend {
            self.svtx.insert(0, pnt.clone());
        } else {
            self.svtx.push(pnt.clone());
        }
    }

    /// `Replace(Index, Pnt)`: `svtx(Index) = Pnt`.
    pub(crate) fn replace(
        &mut self,
        index: i32,
        pnt: &IntPatchPoint,
        member: &str,
    ) -> Result<(), Failure> {
        *self.value_mut(index, member)? = pnt.clone();
        Ok(())
    }

    /// `SetFirstPoint(IndFirst)`.
    pub(crate) fn set_first_point(&mut self, index: i32) {
        (self.fipt, self.indf) = (true, index);
    }

    /// `SetLastPoint(IndLast)`.
    pub(crate) fn set_last_point(&mut self, index: i32) {
        (self.lapt, self.indl) = (true, index);
    }

    /// `FirstPoint()`: Standard_DomainError without a first point, then the vertex of
    /// index `indf`.
    pub(crate) fn first_point(&self, class: &str) -> Result<&IntPatchPoint, Failure> {
        let member: String = format!("{class}::FirstPoint");
        if !self.fipt {
            return Err(domain_error(&member, "the line has no first point"));
        }
        self.value(self.indf, &member)
    }

    /// `LastPoint()`: Standard_DomainError without a last point, then the vertex of index
    /// `indl`.
    pub(crate) fn last_point(&self, class: &str) -> Result<&IntPatchPoint, Failure> {
        let member: String = format!("{class}::LastPoint");
        if !self.lapt {
            return Err(domain_error(&member, "the line has no last point"));
        }
        self.value(self.indl, &member)
    }

    /// `RemoveVertex(theIndex)` of the point lines: the explicit range check, then the
    /// removal.
    pub(crate) fn remove_vertex(&mut self, index: i32, class: &str) -> Result<(), Failure> {
        if index < 1 || index > self.nb_vertex() {
            return Err(Failure::out_of_range(&format!(
                "{class}::RemoveVertex() - Cannot delete not existing vertex {index}; expected 1..={}",
                self.nb_vertex()
            )));
        }
        self.remove(index, class)
    }

    /// The index shifts of the removal of vertex `j` in the restriction filters:
    /// `if (lapt && indl >= j) indl--;` and the same for `indf`.
    pub(crate) fn shift_after_removal(&mut self, j: i32) {
        if self.lapt && self.indl >= j {
            self.indl -= 1;
        }
        if self.fipt && self.indf >= j {
            self.indf -= 1;
        }
    }

    /// The index shifts of the removal of vertex `j` in IntPatch_GLine, which test
    /// `indl > j` (strictly) where the other lines test `indl >= j`.
    pub(crate) fn shift_after_removal_strict(&mut self, j: i32) {
        if self.lapt && self.indl > j {
            self.indl -= 1;
        }
        if self.fipt && self.indf > j {
            self.indf -= 1;
        }
    }

    /// The index follow-up of `svtx.Exchange(i - 1, i)` in the sorts of IntPatch_GLine
    /// and IntPatch_ALine.
    pub(crate) fn follow_exchange(&mut self, i: i32) {
        if self.fipt {
            self.indf = swapped_index(self.indf, i);
        }
        if self.lapt {
            self.indl = swapped_index(self.indl, i);
        }
    }

    /// The parameter on the line of vertex `index`.
    pub(crate) fn param(&self, index: i32, member: &str) -> Result<f64, Failure> {
        Ok(self.value(index, member)?.parameter_on_line())
    }

    /// The bubble sort on the parameter on the line, repeated until no exchange
    /// (`do { SortIsOK = true; for (i = 2; ...) ... } while (!SortIsOK)`); `follow`
    /// keeps `indf`/`indl` on their vertices.
    pub(crate) fn bubble_sort(
        &mut self,
        nbvtx: i32,
        follow: bool,
        member: &str,
    ) -> Result<(), Failure> {
        loop {
            let mut sort_is_ok: bool = true;
            for i in 2..=nbvtx {
                if self.param(i - 1, member)? > self.param(i, member)? {
                    sort_is_ok = false;
                    self.exchange(i - 1, i, member)?;
                    if follow {
                        self.follow_exchange(i);
                    }
                }
            }
            if sort_is_ok {
                return Ok(());
            }
        }
    }
}

fn swapped_index(index: i32, i: i32) -> i32 {
    if index == i {
        i - 1
    } else if index == i - 1 {
        i
    } else {
        index
    }
}

/// A vertex scan whose loops bound their indices by a count that the scan updates as it
/// removes vertices (the native local `nbvtx`, which can be stale).
pub(crate) trait VertexScan {
    /// The current `nbvtx`.
    fn scan_len(&self) -> i32;
}

/// The vertices of a line with the `nbvtx` count of its scan.
pub(crate) struct CountedVertices<'a> {
    pub(crate) vertices: &'a mut VertexList,
    pub(crate) nbvtx: i32,
}

impl VertexScan for CountedVertices<'_> {
    fn scan_len(&self) -> i32 {
        self.nbvtx
    }
}

/// Runs `pair(i, j)` over the pairs `i != j` of `1..=nbvtx` until it deletes a vertex,
/// and repeats while it does (`do { APointDeleted = false; for (i ...) { if (first(i))
/// for (j ...) } } while (APointDeleted)`).
pub(crate) fn repeat_until_stable<C: VertexScan>(
    ctx: &mut C,
    first: &mut dyn FnMut(&mut C, i32) -> Result<bool, Failure>,
    pair: &mut dyn FnMut(&mut C, i32, i32) -> Result<bool, Failure>,
) -> Result<(), Failure> {
    loop {
        let mut deleted: bool = false;
        let mut i: i32 = 1;
        while i <= ctx.scan_len() && !deleted {
            if first(ctx, i)? {
                let mut j: i32 = 1;
                while j <= ctx.scan_len() && !deleted {
                    deleted = i != j && pair(ctx, i, j)?;
                    j += 1;
                }
            }
            i += 1;
        }
        if !deleted {
            return Ok(());
        }
    }
}

impl VertexList {
    /// Of two neighbour vertices of the same parameter on the line, the one on no domain
    /// is removed (the second one first), until none is (`nbvtx` follows the removals).
    pub(crate) fn remove_free_twins(
        &mut self,
        nbvtx: &mut i32,
        member: &str,
    ) -> Result<(), Failure> {
        let free = |v: &IntPatchPoint| !v.is_on_dom_s1() && !v.is_on_dom_s2();
        loop {
            let mut sort_is_ok: bool = true;
            let mut i: i32 = 2;
            while i <= *nbvtx && sort_is_ok {
                let (pim1, pii) = (self.value(i - 1, member)?, self.value(i, member)?);
                let removed: Option<i32> = if pim1.parameter_on_line() != pii.parameter_on_line() {
                    None
                } else if free(pii) {
                    Some(i)
                } else if free(pim1) {
                    Some(i - 1)
                } else {
                    None
                };
                if let Some(index) = removed {
                    sort_is_ok = false;
                    self.remove(index, member)?;
                    *nbvtx -= 1;
                }
                i += 1;
            }
            if sort_is_ok {
                return Ok(());
            }
        }
    }
}
