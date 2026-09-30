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
// Rust translation of PerformS0, PerformS1 and PerformSurface of
// ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_ConstrainedFilling.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The poles of the filled surface: the Coons construction `S0` of the boundaries,
//! the correction `S1` of the tangent fields, and their sum.
use super::{GeomFillConstrainedFilling, required};
use rustycad_tkernel::ncollection::{Array1, Array2, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::at;
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkg3d::geom::GeomBSplineSurface;
use rustycad_tkmath::gp::{Point3, Vector3, Xyz};

/// The value `j` of a mixing function (`ab[k]->Value(j)`, `pq[k]->Value(j)`).
fn mix(function: &Handle<HArray1<f64>>, j: i32) -> Result<f64, Failure> {
    let values = function
        .get()
        .ok_or_else(|| errors::null_object("GeomFill_ConstrainedFilling: null mixing function"))?
        .array1();
    at(&values, j)
}

/// `S->Value(i, j)` with the native bounds check.
fn grid_at(grid: &Array2<'static, Point3>, i: i32, j: i32) -> Result<Xyz, Failure> {
    grid.value(i, j).map(|p| p.xyz()).map_err(|error| {
        errors::out_of_range(&format!("NCollection_Array2::Value: ({i}, {j}); {error}"))
    })
}

/// `nt[i][k]`, a read past the temporary array natively.
fn nt_at(nt: &[Xyz], k: i32) -> Result<Xyz, Failure> {
    usize::try_from(k)
        .ok()
        .and_then(|k| nt.get(k))
        .copied()
        .ok_or_else(|| {
            errors::out_of_range(&format!(
                "GeomFill_ConstrainedFilling::PerformS1: tangent pole {k}; expected 0..{}",
                nt.len()
            ))
        })
}

/// `degree / (knots(b) - knots(a))`: the derivative factor at an end of a knot vector.
fn end_factor(degree: i32, knots: &Array1<'static, f64>, a: i32, b: i32) -> Result<f64, Failure> {
    Ok(f64::from(degree) / (at(knots, b)? - at(knots, a)?))
}

/// `(nt[k1] - nt[k0]) * coef`, a difference of two tangent poles.
fn difference(nt: &[Xyz], (k0, k1): (i32, i32), coef: f64) -> Result<Xyz, Failure> {
    Ok((nt_at(nt, k0)? * -1. + nt_at(nt, k1)?) * coef)
}

/// The row walked by the tangent correction of side `i`: the first pole `(i1, j1)`
/// and second pole `(i2, j2)`, whether the walk goes along the rows, and the factor
/// `z` (minus the derivative factor at that end).
struct SideWalk {
    first: (i32, i32),
    second: (i32, i32),
    along_rows: bool,
    z: f64,
}

impl GeomFillConstrainedFilling {
    /// `PerformS0()`: the Coons combination of the boundary poles, the blending
    /// functions `ab` and the corners.
    pub(super) fn perform_s0(&mut self) -> Result<(), Failure> {
        let (c0, c2) = (
            required(&self.ncpol[0], "ncpol[0]")?,
            required(&self.ncpol[2], "ncpol[2]")?,
        );
        let (c1, c3) = (
            required(&self.ncpol[1], "ncpol[1]")?,
            required(&self.ncpol[3], "ncpol[3]")?,
        );
        let (ni, nj) = (c0.length(), c1.length());
        let mut ss0: Array2<'static, Point3> =
            Array2::with_bounds(1, ni, 1, nj).map_err(errors::array_bounds)?;
        let patch = self.patch()?;
        let corner = [0, 1, 2, 3].map(|k| patch.corner(k).map(|p| p.xyz()));
        let [k0, k1, k2, k3] = corner;
        let (k0, k1, k2, k3) = (k0?, k1?, k2?, k3?);
        for i in 1..=ni {
            let (ab1, ab3) = (mix(&self.ab[1], i)?, mix(&self.ab[3], i)?);
            let (b0, b2) = (at(c0, i)?.xyz(), at(c2, i)?.xyz());
            for j in 1..=nj {
                let (ab0, ab2) = (mix(&self.ab[0], j)?, mix(&self.ab[2], j)?);
                let (b1, b3) = (at(c1, j)?.xyz(), at(c3, j)?.xyz());
                let mut polij: Xyz = b0 * ab0 + b1 * ab1;
                polij += b2 * ab2;
                polij += b3 * ab3;
                polij += k0 * (-ab3 * ab0);
                polij += k1 * (-ab0 * ab1);
                polij += k2 * (-ab1 * ab2);
                polij += k3 * (-ab2 * ab3);
                ss0[(i, j)] = Point3::from(polij);
            }
        }
        self.s0 = Some(ss0);
        Ok(())
    }
    /// The walk of the tangent correction along side `i` of the `ni` x `nj` grid.
    fn side_walk(&self, i: usize, (ni, nj): (i32, i32)) -> Result<SideWalk, Failure> {
        let (k0, k1) = (
            required(&self.nk[0], "nk[0]")?,
            required(&self.nk[1], "nk[1]")?,
        );
        let (l0, l1) = (k0.length(), k1.length());
        Ok(match i {
            0 => SideWalk {
                first: (1, 1),
                second: (1, 2),
                along_rows: true,
                z: end_factor(-self.degree[1], k1, 1, 2)?,
            },
            1 => SideWalk {
                first: (ni - 1, 1),
                second: (ni, 1),
                along_rows: false,
                z: end_factor(-self.degree[0], k0, l0 - 1, l0)?,
            },
            2 => SideWalk {
                first: (1, nj - 1),
                second: (1, nj),
                along_rows: true,
                z: end_factor(-self.degree[1], k1, l1 - 1, l1)?,
            },
            _ => SideWalk {
                first: (1, 1),
                second: (2, 1),
                along_rows: false,
                z: end_factor(-self.degree[0], k0, 1, 2)?,
            },
        })
    }
    /// `nt[i]`: the tangent poles of side `i` minus the cross derivative of `S0`
    /// there (`None` without tangent field).
    fn tangent_corrections(&self, i: usize) -> Result<Option<Vec<Xyz>>, Failure> {
        let Some(ntpol) = &self.ntpol[i] else {
            return Ok(None);
        };
        let s0: &Array2<'static, Point3> = required(&self.s0, "S0")?;
        let walk: SideWalk = self.side_walk(i, (s0.col_length(), s0.row_length()))?;
        let ((mut i1, mut j1), (mut i2, mut j2)) = (walk.first, walk.second);
        let mut nt: Vec<Xyz> = Vec::with_capacity(ntpol.len());
        for k in 0..ntpol.length() {
            let value: Xyz = (grid_at(s0, i1, j1)? * -1. + grid_at(s0, i2, j2)?) * walk.z;
            nt.push(value + at(ntpol, k + 1)?.xyz());
            if walk.along_rows {
                (i1, i2) = (i1 + 1, i2 + 1);
            } else {
                (j1, j2) = (j1 + 1, j2 + 1);
            }
        }
        Ok(Some(nt))
    }
    /// The corner twists `v[0..3]`: the mean of the two tangent differences at each
    /// corner whose two sides have tangent fields (the others keep their value).
    fn corner_twists(&mut self, nt: &[Option<Vec<Xyz>>; 4]) -> Result<(), Failure> {
        let (k0, k1) = (
            required(&self.nk[0], "nk[0]")?,
            required(&self.nk[1], "nk[1]")?,
        );
        let (ln0, ln1) = (k0.length(), k1.length());
        let lp0: i32 = required(&self.ncpol[0], "ncpol[0]")?.length();
        let lp1: i32 = required(&self.ncpol[1], "ncpol[1]")?.length();
        let first0: f64 = end_factor(self.degree[0], k0, 1, 2)?;
        let last0: f64 = end_factor(self.degree[0], k0, ln0 - 1, ln0)?;
        let first1: f64 = end_factor(self.degree[1], k1, 1, 2)?;
        let last1: f64 = end_factor(self.degree[1], k1, ln1 - 1, ln1)?;
        let corners = [
            (0, 3, (0, 1), (0, 1), (first0, first1)),
            (0, 1, (lp0 - 2, lp0 - 1), (0, 1), (last0, first1)),
            (2, 1, (lp0 - 2, lp0 - 1), (lp1 - 2, lp1 - 1), (last0, last1)),
            (2, 3, (0, 1), (lp1 - 2, lp1 - 1), (first0, last1)),
        ];
        for (k, (a, b, ka, kb, (coef0, coef1))) in corners.into_iter().enumerate() {
            let (Some(na), Some(nb)) = (&nt[a], &nt[b]) else {
                continue;
            };
            let vtemp: Xyz = (difference(na, ka, coef0)? + difference(nb, kb, coef1)?) * 0.5;
            self.v[k] = Vector3::from(vtemp);
        }
        Ok(())
    }
    /// `PerformS1()`: the correction of the tangent fields, mixed by `pq`, minus the
    /// corner twists.
    pub(super) fn perform_s1(&mut self) -> Result<(), Failure> {
        let nt: [Option<Vec<Xyz>>; 4] = [
            self.tangent_corrections(0)?,
            self.tangent_corrections(1)?,
            self.tangent_corrections(2)?,
            self.tangent_corrections(3)?,
        ];
        self.corner_twists(&nt)?;
        let s0: &Array2<'static, Point3> = required(&self.s0, "S0")?;
        let (ni, nj) = (s0.col_length(), s0.row_length());
        let mut ss1: Array2<'static, Point3> =
            Array2::with_bounds(1, ni, 1, nj).map_err(errors::array_bounds)?;
        for i in 1..=ni {
            for j in 1..=nj {
                ss1[(i, j)] = Point3::from(self.correction(&nt, i, j)?);
            }
        }
        self.s1 = Some(ss1);
        Ok(())
    }
    /// The correction pole `(i, j)` of `S1`.
    fn correction(&self, nt: &[Option<Vec<Xyz>>; 4], i: i32, j: i32) -> Result<Xyz, Failure> {
        let has = |k: usize| nt[k].is_some();
        let pq1: f64 = if has(1) { -mix(&self.pq[1], i)? } else { 0. };
        let pq3: f64 = if has(3) { mix(&self.pq[3], i)? } else { 0. };
        let pq0: f64 = if has(0) { mix(&self.pq[0], j)? } else { 0. };
        let pq2: f64 = if has(2) { -mix(&self.pq[2], j)? } else { 0. };
        let term = |k: usize, index: i32| -> Result<Xyz, Failure> {
            nt[k]
                .as_deref()
                .map_or(Ok(Xyz::default()), |row| nt_at(row, index - 1))
        };
        let (t0, t2, t1, t3) = (term(0, i)?, term(2, i)?, term(1, j)?, term(3, j)?);
        let mut tpolij = Xyz::new(0., 0., 0.);
        for (k, t, pq) in [(0, t0, pq0), (1, t1, pq1), (2, t2, pq2), (3, t3, pq3)] {
            if has(k) {
                tpolij += t * pq;
            }
        }
        let twists = [
            (3, 0, -pq3 * pq0),
            (0, 1, -pq0 * pq1),
            (1, 2, -pq1 * pq2),
            (2, 3, -pq2 * pq3),
        ];
        for (k, (a, b, factor)) in twists.into_iter().enumerate() {
            if has(a) && has(b) {
                tpolij += *self.v[k].xyz() * factor;
            }
        }
        Ok(tpolij)
    }
    /// `PerformSurface()`: the surface of the poles `S0 + S1` on the matched knots.
    pub(super) fn perform_surface(&mut self) -> Result<(), Failure> {
        let (t0, t1) = (required(&self.s0, "S0")?, required(&self.s1, "S1")?);
        let (ni, nj) = (t0.col_length(), t0.row_length());
        let mut temp: Array2<'static, Point3> =
            Array2::with_bounds(1, ni, 1, nj).map_err(errors::array_bounds)?;
        for i in 1..=ni {
            for j in 1..=nj {
                temp[(i, j)] = Point3::from(grid_at(t0, i, j)? + grid_at(t1, i, j)?);
            }
        }
        let u = (
            required(&self.nk[0], "nk[0]")?,
            required(&self.nm[0], "nm[0]")?,
        );
        let v = (
            required(&self.nk[1], "nk[1]")?,
            required(&self.nm[1], "nm[1]")?,
        );
        let surface = GeomBSplineSurface::new(
            &temp,
            u,
            v,
            (self.degree[0], self.degree[1]),
            (false, false),
        )?;
        self.surf = Handle::new(surface);
        Ok(())
    }
}
