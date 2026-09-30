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
// Rust translation of ModelingAlgorithms/TKTopAlgo/MAT2d/MAT2d_Mat2d.hxx/.cxx (the fields, the accessors and the destructor) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MAT2d_Mat2d`: the algorithm of the bisecting locus of a circuit: the trees of
//! bisectors from the bisectors of neighbour items up.
mod areas;
mod engine;
mod removal;
mod roots;

use crate::mat::{MatBisector, MatEdge, MatListOfBisector, MatListOfEdge};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::errors;
use std::collections::HashMap;

/// `MAT2d_Mat2d`: the bisecting locus of the items of a [`super::Mat2dTool2d`]
/// ([`Mat2dMat2d::create_mat`] for a closed circuit, [`Mat2dMat2d::create_mat_open`]
/// for an open one, docs/topalgo-design.md section 3), read through its roots. The
/// bisectors and the edges reference each other; the `Drop` breaks those cycles, as
/// the native destructor does.
/// ```
/// use rustycad_tktopalgo::mat2d::Mat2dMat2d;
/// let mat = Mat2dMat2d::new(false);
/// assert!(!mat.is_done() && !mat.semi_infinite());
/// assert_eq!(mat.number_of_bisectors(), 0);
/// ```
pub struct Mat2dMat2d {
    my_is_open_result: bool,
    thenumberofbisectors: i32,
    semi_infinite: bool,
    theedgelist: Handle<MatListOfEdge>,
    removed_edges_list: Handle<MatListOfEdge>,
    typeofbisectortoremove: HashMap<i32, i32>,
    bisectoronetoremove: HashMap<i32, Handle<MatBisector>>,
    bisectortwotoremove: HashMap<i32, Handle<MatBisector>>,
    bisectormap: HashMap<i32, Handle<MatBisector>>,
    roots: Handle<MatListOfBisector>,
    is_done: bool,
}

// The value of a map of the algorithm; a missing key raises Standard_NoSuchObject.
pub(super) fn bound<V: Clone>(map: &HashMap<i32, V>, key: i32, what: &str) -> Result<V, Failure> {
    map.get(&key).cloned().ok_or_else(|| {
        errors::no_such_object(&format!(
            "MAT2d_Mat2d: no {what} {key}; expected a bound one"
        ))
    })
}

// The list of the roots; a null one is a native null dereference.
fn roots_ref(roots: &Handle<MatListOfBisector>) -> Result<&MatListOfBisector, Failure> {
    roots.get().ok_or_else(|| {
        errors::null_object("MAT2d_Mat2d: no roots; expected CreateMat or CreateMatOpen")
    })
}

impl Mat2dMat2d {
    /// `MAT2d_Mat2d(IsOpenResult)` (the native default is false); `Mat2dMat2d::new(false)`.
    pub fn new(is_open_result: bool) -> Self {
        Self {
            my_is_open_result: is_open_result,
            thenumberofbisectors: 0,
            semi_infinite: false,
            theedgelist: Handle::default(),
            removed_edges_list: Handle::default(),
            typeofbisectortoremove: HashMap::new(),
            bisectoronetoremove: HashMap::new(),
            bisectortwotoremove: HashMap::new(),
            bisectormap: HashMap::new(),
            roots: Handle::default(),
            is_done: false,
        }
    }
    /// `IsDone()`: false when the algorithm needed too many iterations;
    /// `mat.is_done()`.
    pub fn is_done(&self) -> bool {
        self.is_done
    }
    /// `Init()`: the first root becomes current; `mat.init()?`.
    pub fn init(&self) -> Result<(), Failure> {
        roots_ref(&self.roots)?.first();
        Ok(())
    }
    /// `More()`: true while there is a current root; `mat.more()?`.
    pub fn more(&self) -> Result<bool, Failure> {
        Ok(roots_ref(&self.roots)?.more())
    }
    /// `Next()`: the next root becomes current; `mat.next()?`.
    pub fn next(&self) -> Result<(), Failure> {
        roots_ref(&self.roots)?.next()
    }
    /// `Bisector()`: the current root; `mat.bisector()?`.
    pub fn bisector(&self) -> Result<Handle<MatBisector>, Failure> {
        roots_ref(&self.roots)?.current()
    }
    /// `SemiInfinite()`: true when the roots go to infinity (the locus outside a
    /// contour, or an interrupted computation); `mat.semi_infinite()`.
    pub fn semi_infinite(&self) -> bool {
        self.semi_infinite
    }
    /// `NumberOfBisectors()`: the number of geometric bisectors kept;
    /// `mat.number_of_bisectors()`.
    pub fn number_of_bisectors(&self) -> i32 {
        self.thenumberofbisectors
    }
}

impl Drop for Mat2dMat2d {
    // The destructor unlinks the bisectors from their edges and the edges from their
    // bisectors, so the reference cycles between them are broken.
    fn drop(&mut self) {
        for bisector in self.bisectormap.values() {
            bisector.set_first_edge(Handle::default());
            bisector.set_second_edge(Handle::default());
        }
        for list in [&self.theedgelist, &self.removed_edges_list] {
            let Some(list) = list.get() else {
                continue;
            };
            list.first();
            for _ in 0..list.number() {
                if let Ok(edge) = list.current() {
                    unlink_edge(&edge);
                }
                if list.next().is_err() {
                    break;
                }
            }
        }
    }
}

fn unlink_edge(edge: &Handle<MatEdge>) {
    if let Some(edge) = edge.get() {
        edge.set_first_bisector(&Handle::default());
        edge.set_second_bisector(&Handle::default());
    }
}
