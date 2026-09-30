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
// Rust translation of MatchKnots and the static helpers inqadd and mklin of
// ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_ConstrainedFilling.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The knots of the blending functions: the domains of the tangent fields inserted
//! as knots, and the mixing functions of `Law::MixBnd` and `Law::MixTgt`.
use super::{GeomFillConstrainedFilling, required};
use crate::law::{self, LawFunction, LawLinear};
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::{at, one_based};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::Point3;

/// `inqadd`: the knots to add for the domains `d1` and `d2` of the two tangent
/// fields of a direction (none when both are 1, one when they meet within `tolk`),
/// all of multiplicity `deg - 2`.
fn inqadd(d1: f64, d2: f64, deg: i32, tolk: f64) -> Vec<(f64, i32)> {
    let m: i32 = deg - 2;
    if d1 != 1. && d2 != 1. {
        if (d1 + d2 - 1.).abs() < tolk {
            return vec![(0.5 * (d1 + 1. - d2), m)];
        }
        return vec![(d1.min(1. - d2), m), (d1.max(1. - d2), m)];
    }
    if d1 != 1. {
        return vec![(d1, m)];
    }
    if d2 != 1. {
        return vec![(d2, m)];
    }
    Vec::new()
}

/// `mklin`: the law itself when it is linear, else the linear law between its
/// values at its bounds.
fn mklin(func: &Handle<dyn LawFunction>) -> Result<Handle<LawLinear>, Failure> {
    let linear: Handle<LawLinear> = func.cast();
    if linear.is_some() {
        return Ok(linear);
    }
    let func: &dyn LawFunction = func
        .get()
        .ok_or_else(|| errors::null_object("mklin: null blending law"))?;
    let fu = LawLinear::new();
    let (d, f) = func.bounds()?;
    fu.set(d, func.value(d)?, f, func.value(f)?);
    Ok(Handle::new(fu))
}

/// The knots and multiplicities of one direction after the insertion, and the pole
/// rows of its two boundaries and tangent fields.
struct Direction {
    knots: Array1<'static, f64>,
    mults: Array1<'static, i32>,
    rows: [Option<Array1<'static, Point3>>; 4],
}

impl GeomFillConstrainedFilling {
    /// `MatchKnots()`: inserts the knots of the domains (`SetDomain`) into both
    /// directions, then computes the blending functions `ab` (Law::MixBnd of the
    /// linear laws, and their complements) and the tangent mixing functions `pq`
    /// (Law::MixTgt, cut at the domain knots).
    pub(super) fn match_knots(&mut self) -> Result<(), Failure> {
        self.nm = self.mults.clone();
        self.nk = self.knots.clone();
        let mut ind: [i32; 4] = [required(&self.nk[1], "nk[1]")?.length(), 1, 1, 0];
        ind[3] = required(&self.nk[0], "nk[0]")?.length();
        self.ncpol = self.curvpol.clone();
        self.ntpol = self.tgtepol.clone();
        let tolk: f64 = 1. / f64::from(10.max(2 * required(&self.knots[1], "knots[1]")?.length()));
        self.insert_domain_knots(1, [0, 2], tolk, &mut ind)?;
        let tolk: f64 =
            1. / 10f64.max(2. * f64::from(required(&self.knots[0], "knots[0]")?.length()));
        self.insert_domain_knots(0, [1, 3], tolk, &mut ind)?;
        self.mixing_functions(ind)
    }
    /// The insertion of the domain knots of the sides `sides` into direction `d`
    /// (the knots of `degree[d]`), and the indices of the domain knots in `ind`.
    /// Quirk kept: the domains are compared with the multiplicities `nm`, not the
    /// knots.
    fn insert_domain_knots(
        &mut self,
        d: usize,
        sides: [usize; 2],
        tolk: f64,
        ind: &mut [i32; 4],
    ) -> Result<(), Failure> {
        let add = inqadd(self.dom[sides[0]], self.dom[sides[1]], self.degree[d], tolk);
        if add.is_empty() {
            return Ok(());
        }
        let mut addk: Array1<'static, f64> = one_based(add.len() as i32)?;
        let mut addm: Array1<'static, i32> = one_based(add.len() as i32)?;
        for (k, &(knot, mult)) in (1..).zip(&add) {
            (addk[k], addm[k]) = (knot, mult);
        }
        let nbnk: i32 = self.insert_into_direction(d, (&addk, &addm), tolk)?;
        let nm: &Array1<'static, i32> = required(&self.nm[d], "nm")?;
        let [first, second] = sides;
        if self.dom[first] != 1.
            && let Some(i) = find_index(2..=nbnk, |i| {
                Ok((self.dom[first] - f64::from(at(nm, i)?)).abs() < tolk)
            })?
        {
            ind[first] = i;
        }
        if self.dom[second] != 1.
            && let Some(i) = find_index(1..nbnk, |i| {
                Ok((1. - self.dom[second] - f64::from(at(nm, i)?)).abs() < tolk)
            })?
        {
            ind[second] = i;
        }
        Ok(())
    }
    /// `PrepareInsertKnots` and the `InsertKnots` of the two boundaries and the
    /// tangent fields of direction `d`; returns the new knot count (the count
    /// `PrepareInsertKnots` left when it refuses, as natively).
    fn insert_into_direction(
        &mut self,
        d: usize,
        (addk, addm): (&Array1<'static, f64>, &Array1<'static, i32>),
        tolk: f64,
    ) -> Result<i32, Failure> {
        let insertion = KnotInsertion {
            degree: self.degree[d],
            knots: required(&self.knots[d], "knots")?.clone(),
            mults: required(&self.mults[d], "mults")?.clone(),
            added: (addk, addm),
            tolk,
        };
        let (mergeable, nbnp, nbnk) = insertion.prepare()?;
        if !mergeable {
            return Ok(nbnk);
        }
        let mut direction = Direction {
            knots: one_based(nbnk)?,
            mults: one_based(nbnk)?,
            rows: Default::default(),
        };
        let rows = [
            (d, &self.curvpol),
            (d + 2, &self.curvpol),
            (d, &self.tgtepol),
            (d + 2, &self.tgtepol),
        ];
        for (k, (side, source)) in rows.into_iter().enumerate() {
            if let Some(poles) = &source[side] {
                direction.rows[k] = Some(insertion.insert(poles, nbnp, &mut direction)?);
            }
        }
        self.store_direction(d, direction);
        Ok(nbnk)
    }
    /// The new knots, multiplicities and pole rows of direction `d` (the knots of
    /// the boundaries `d` and `d + 2`).
    fn store_direction(&mut self, d: usize, direction: Direction) {
        let [c0, c2, t0, t2] = direction.rows;
        self.nm[d] = Some(direction.mults);
        self.nk[d] = Some(direction.knots);
        (self.ncpol[d], self.ncpol[d + 2]) = (c0, c2);
        if t0.is_some() {
            self.ntpol[d] = t0;
        }
        if t2.is_some() {
            self.ntpol[d + 2] = t2;
        }
    }
    /// The blending functions `ab` and the tangent mixing functions `pq`.
    fn mixing_functions(&mut self, ind: [i32; 4]) -> Result<(), Failure> {
        let (k0, m0) = (
            required(&self.nk[0], "nk[0]")?,
            required(&self.nm[0], "nm[0]")?,
        );
        let (k1, m1) = (
            required(&self.nk[1], "nk[1]")?,
            required(&self.nm[1], "nm[1]")?,
        );
        let patch = self.patch()?;
        let ab0 = law::mix_bnd(self.degree[1], k1, m1, &mklin(&patch.func(0)?)?)?;
        let ab1 = law::mix_bnd(self.degree[0], k0, m0, &mklin(&patch.func(1)?)?)?;
        let (ab2, ab3) = (complement(&ab0)?, complement(&ab1)?);
        let pq0 = law::mix_tgt(self.degree[1], k1, m1, true, ind[0])?;
        let pq2 = law::mix_tgt(self.degree[1], k1, m1, false, ind[2])?;
        let pq1 = law::mix_tgt(self.degree[0], k0, m0, false, ind[1])?;
        let pq3 = law::mix_tgt(self.degree[0], k0, m0, true, ind[3])?;
        self.ab = [ab0, ab1, ab2, ab3];
        self.pq = [pq0, pq1, pq2, pq3];
        Ok(())
    }
}

/// `1 - ab(j)` for every value of `ab`, 1-based.
fn complement(ab: &Handle<HArray1<f64>>) -> Result<Handle<HArray1<f64>>, Failure> {
    let values = ab
        .get()
        .ok_or_else(|| errors::null_object("MatchKnots: null blending function"))?
        .array1();
    let mut out: Array1<'static, f64> = one_based(values.length())?;
    for j in 1..=values.length() {
        out[j] = 1. - at(&values, j)?;
    }
    Ok(Handle::new(HArray1::from_owned(out)))
}

/// The knot insertion of one direction: its degree, knots and multiplicities, the
/// knots to add and the tolerance.
struct KnotInsertion<'a> {
    degree: i32,
    knots: Array1<'static, f64>,
    mults: Array1<'static, i32>,
    added: (&'a Array1<'static, f64>, &'a Array1<'static, i32>),
    tolk: f64,
}

impl KnotInsertion<'_> {
    /// `BSplCLib::PrepareInsertKnots`: whether the knots merge, and the new pole
    /// and knot counts.
    fn prepare(&self) -> Result<(bool, i32, i32), Failure> {
        let (mut nbnp, mut nbnk) = (0, 0);
        let (addk, addm) = self.added;
        let mergeable = bsplclib::prepare_insert_knots(
            self.degree,
            false,
            &self.knots,
            &self.mults,
            addk,
            Some(addm),
            &mut nbnp,
            &mut nbnk,
            self.tolk,
            false,
        )?;
        Ok((mergeable, nbnp, nbnk))
    }
    /// `BSplCLib::InsertKnots` of one pole row (no weights) into `nbnp` new poles;
    /// the new knots and multiplicities go to `direction`.
    fn insert(
        &self,
        poles: &Array1<'static, Point3>,
        nbnp: i32,
        direction: &mut Direction,
    ) -> Result<Array1<'static, Point3>, Failure> {
        let mut new_poles: Array1<'static, Point3> = one_based(nbnp)?;
        let (addk, addm) = self.added;
        bsplclib::insert_knots(
            self.degree,
            false,
            poles,
            None,
            &self.knots,
            &self.mults,
            addk,
            Some(addm),
            &mut new_poles,
            None,
            &mut direction.knots,
            &mut direction.mults,
            self.tolk,
            false,
        )?;
        Ok(new_poles)
    }
}

/// The first index of `range` for which `test` holds.
fn find_index(
    range: impl Iterator<Item = i32>,
    test: impl Fn(i32) -> Result<bool, Failure>,
) -> Result<Option<i32>, Failure> {
    for i in range {
        if test(i)? {
            return Ok(Some(i));
        }
    }
    Ok(None)
}
