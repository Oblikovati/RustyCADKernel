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
// Rust translation of ModelingAlgorithms/TKTopAlgo/Bisector/Bisector_PolyBis.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Bisector_PolyBis`: the polygon of points that samples a [`super::BisecCC`].
use super::PointOnBis;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkmath::gp::{self, Point2, Transform2};

// `thePoints[30]`: the native array, of which slots 1..29 hold points.
const CAPACITY: usize = 30;

/// `Bisector_PolyBis`: up to 29 points in a fixed array indexed from 1, as the
/// native class; slot 0 and the slots after the last point keep default points, so
/// `First` and `Last` of an empty polygon read a default point as natively.
/// ```
/// use rustycad_tktopalgo::bisector::{PointOnBis, PolyBis};
/// use rustycad_tkmath::gp::Point2;
/// let mut poly = PolyBis::new();
/// poly.append(PointOnBis::new(0.0, 0.0, 0.0, 1.0, Point2::new(0.0, 0.0)))?;
/// poly.append(PointOnBis::new(1.0, 1.0, 1.0, 1.0, Point2::new(1.0, 0.0)))?;
/// assert_eq!(poly.interval(0.5)?, 1);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PolyBis {
    the_points: [PointOnBis; CAPACITY],
    nb_points: i32,
}

impl Default for PolyBis {
    fn default() -> Self {
        Self::new()
    }
}

impl PolyBis {
    /// `Bisector_PolyBis()`: an empty polygon; `PolyBis::new()`.
    pub fn new() -> Self {
        Self {
            the_points: [PointOnBis::default(); CAPACITY],
            nb_points: 0,
        }
    }
    /// `Append(Point)`. Natively a 30th point overruns the array; the port raises
    /// Standard_OutOfRange with the count instead; `poly.append(p)?`.
    pub fn append(&mut self, point: PointOnBis) -> Result<(), Failure> {
        let slot: usize = Self::slot(self.nb_points + 1)?;
        self.nb_points += 1;
        self.the_points[slot] = point;
        Ok(())
    }
    /// `Length()`; `poly.length()`.
    pub fn length(&self) -> i32 {
        self.nb_points
    }
    /// `IsEmpty()`; `poly.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.nb_points == 0
    }
    /// `Value(Index)`: the slot `index` of the array, unchecked natively; the port
    /// raises Standard_OutOfRange outside `0..30`; `poly.value(1)?`.
    pub fn value(&self, index: i32) -> Result<PointOnBis, Failure> {
        Ok(self.the_points[Self::slot(index)?])
    }
    /// `First()`: slot 1; `poly.first()`.
    pub fn first(&self) -> PointOnBis {
        self.the_points[1]
    }
    /// `Last()`: slot `Length()` (slot 0 when empty); `poly.last()`.
    pub fn last(&self) -> PointOnBis {
        self.the_points[self.nb_points as usize]
    }
    /// `Interval(U)`: the index of the segment whose parameters on the bisector
    /// bracket `u`; `poly.interval(0.5)?`.
    pub fn interval(&self, u: f64) -> Result<i32, Failure> {
        if self.last().param_on_bis() - u < gp::resolution() {
            return Ok(self.nb_points - 1);
        }
        let du: f64 = (self.last().param_on_bis() - self.first().param_on_bis())
            / f64::from(self.nb_points - 1);
        if du <= gp::resolution() {
            return Ok(1);
        }
        // `int(...)` truncates toward zero.
        let int_u: i32 = ((u - self.first().param_on_bis()).abs() / du) as i32 + 1;
        if self.value(int_u)?.param_on_bis() >= u {
            return self.search_down(int_u, u);
        }
        self.search_up(int_u, u)
    }
    // The downward search of Interval: the last slot at or below `u`.
    fn search_down(&self, int_u: i32, u: f64) -> Result<i32, Failure> {
        for i in (1..=int_u).rev() {
            if self.value(i)?.param_on_bis() <= u {
                return Ok(i);
            }
        }
        Ok(int_u)
    }
    // The upward search of Interval: the slot before the first at or above `u`.
    fn search_up(&self, int_u: i32, u: f64) -> Result<i32, Failure> {
        for i in int_u..self.nb_points {
            if self.value(i)?.param_on_bis() >= u {
                return Ok(i - 1);
            }
        }
        Ok(int_u)
    }
    /// `Transform(T)`: transforms the point of every vertex; `poly.transform(t)`.
    pub fn transform(&mut self, transform: Transform2) {
        for i in 1..=self.nb_points as usize {
            let mut p: Point2 = self.the_points[i].point();
            p.transform(transform);
            self.the_points[i].set_point(p);
        }
    }
    // The array slot of `index`, or Standard_OutOfRange with the index.
    fn slot(index: i32) -> Result<usize, Failure> {
        match usize::try_from(index) {
            Ok(slot) if slot < CAPACITY => Ok(slot),
            _ => Err(errors::out_of_range(&format!(
                "Bisector_PolyBis: index {index}; expected 0..{}",
                CAPACITY - 1
            ))),
        }
    }
}
