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
// Rust translation of the Extrema_ExtCC2d/Extrema_ExtPC2d members kept by
// Geom2dAPI_ExtremaCurveCurve and Geom2dAPI_ProjectPointOnCurve at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The results of an Extrema algorithm as the Geom2dAPI classes read them: each
//! extremum and square distance with the exception the Extrema accessor raised.
use rustycad_tkernel::standard::Failure;

/// The done flag, the count and, per extremum, the item and the square distance
/// returned by the Extrema accessors (or their exception).
pub(crate) struct ExtremaResults<T: Clone> {
    name: &'static str,
    pub(crate) done: bool,
    pub(crate) nb: i32,
    items: Vec<Result<T, Failure>>,
    sq: Vec<Result<f64, Failure>>,
}

impl<T: Clone> ExtremaResults<T> {
    /// The results of an Extrema algorithm that failed.
    pub(crate) fn not_done(name: &'static str) -> Self {
        Self {
            name,
            done: false,
            nb: 0,
            items: Vec::new(),
            sq: Vec::new(),
        }
    }
    /// The results of a done algorithm with `nb` extrema, to fill with `push`.
    pub(crate) fn done(name: &'static str, nb: i32) -> Self {
        Self {
            name,
            done: true,
            nb,
            items: Vec::new(),
            sq: Vec::new(),
        }
    }
    /// The next extremum as its accessors returned it.
    pub(crate) fn push(&mut self, item: Result<T, Failure>, square_distance: Result<f64, Failure>) {
        self.items.push(item);
        self.sq.push(square_distance);
    }
    /// `NbExt()`: StdFail_NotDone when the algorithm failed.
    pub(crate) fn nb_ext(&self) -> Result<i32, Failure> {
        if !self.done {
            return Err(super::not_done(&format!("{}::NbExt", self.name)));
        }
        Ok(self.nb)
    }
    /// The extremum `n` (1-based) as its accessor returned it.
    pub(crate) fn item(&self, n: i32) -> Result<T, Failure> {
        self.stored(&self.items, n)?.clone()
    }
    /// `SquareDistance(n)` as it was returned.
    pub(crate) fn square_distance(&self, n: i32) -> Result<f64, Failure> {
        self.stored(&self.sq, n)?.clone()
    }
    /// The index of the smallest square distance (the first of equal ones).
    pub(crate) fn nearest(&self) -> Result<i32, Failure> {
        let mut dist2_min: f64 = self.square_distance(1)?;
        let mut index: i32 = 1;
        for i in 2..=self.nb {
            let dist2: f64 = self.square_distance(i)?;
            if dist2 < dist2_min {
                dist2_min = dist2;
                index = i;
            }
        }
        Ok(index)
    }

    fn stored<'s, V>(
        &self,
        values: &'s [Result<V, Failure>],
        n: i32,
    ) -> Result<&'s Result<V, Failure>, Failure> {
        let rank: usize = usize::try_from(n - 1).unwrap_or(usize::MAX);
        values.get(rank).ok_or_else(|| {
            Failure::out_of_range(&format!(
                "{}() - index {n} outside [1, {}]",
                self.name,
                values.len()
            ))
        })
    }
}
