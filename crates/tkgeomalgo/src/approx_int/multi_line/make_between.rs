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
// Rust translation of ApproxInt_MultiLine::MakeMLBetween of
// ModelingAlgorithms/TKGeomAlgo/ApproxInt/ApproxInt_MultiLine.gxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MakeMLBetween`: a sub-line resampled at a constant 3d arc length, with the points
//! between the line points computed by the SvSurfaces.
use super::ApproxIntMultiLine;
use crate::approx_int::line::ApproxIntLine;
use crate::approx_int::sv_surfaces::{ApproxIntSvComputed, SvSurfacesRef};
use crate::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::Point2;

/// The parameters and the cumulated 3d arc length of the points `Low..=High` (the
/// arrays `U1`, `V1`, `U2`, `V2` and `AC`, 0-based from `Low`).
struct Abscissae {
    low: i32,
    uv: Vec<[f64; 4]>,
    ac: Vec<f64>,
}

impl Abscissae {
    fn ac(&self, i: i32) -> f64 {
        self.ac[(i - self.low) as usize]
    }
    fn uv(&self, i: i32) -> [f64; 4] {
        self.uv[(i - self.low) as usize]
    }
}

/// The state of the resampling loop (`Indice`, `HasBeenInserted`, the result line).
struct Resampling<'a, L: ApproxIntLine> {
    line: &'a L,
    abs: Abscissae,
    high: i32,
    indice: i32,
    has_been_inserted: bool,
    result: Handle<IntSurfLineOn2S>,
}

impl<L: ApproxIntLine> Resampling<'_, L> {
    fn add_line_point(&self, i: i32) -> Result<(), Failure> {
        self.result.add(&self.line.point(i)?);
        Ok(())
    }

    /// The `while (AC(Indice + 1) <= s)` walk to the interval of `s` and the insertion of
    /// its first point; true when the walk reached `High` (the native `break`).
    fn advance(&mut self, s: f64) -> Result<bool, Failure> {
        while self.abs.ac(self.indice + 1) <= s {
            if !self.has_been_inserted {
                self.add_line_point(self.indice)?;
            }
            self.has_been_inserted = false;
            self.indice += 1;
            if self.indice == self.high {
                break;
            }
        }
        if self.indice == self.high {
            return Ok(true);
        }
        if !self.has_been_inserted && self.abs.ac(self.indice) <= s {
            self.add_line_point(self.indice)?;
            self.has_been_inserted = true;
        }
        Ok(false)
    }

    /// The point at `s` between the points `Indice` and `Indice + 1`, by `Compute` from
    /// the interpolated parameters; the new `s` (moved back by `ds - dsmin` when the
    /// point is too close to an existing one) or None to stop.
    fn insert_at(
        &mut self,
        sv: &SvSurfacesRef,
        s: f64,
        (ds, dsmin): (f64, f64),
    ) -> Result<Option<f64>, Failure> {
        let (i, abs) = (self.indice, &self.abs);
        let a: f64 = s - abs.ac(i);
        let b: f64 = abs.ac(i + 1) - s;
        let nab: f64 = 1.0 / (a + b);
        if a > dsmin && b > dsmin {
            let (p, q) = (abs.uv(i), abs.uv(i + 1));
            let mut uv: [f64; 4] = [0.0; 4];
            for k in 0..4 {
                uv[k] = (p[k] * b + q[k] * a) * nab;
            }
            let mut out = ApproxIntSvComputed::default();
            if sv.borrow_mut().compute(&mut uv, &mut out)? {
                let mut start = IntSurfPntOn2S::new();
                start.set_value_all(out.point, uv[0], uv[1], uv[2], uv[3]);
                self.result.add(&start);
            }
            return Ok(Some(s));
        }
        // The point is not far enough from the existing ones.
        if b < 0.0 {
            return Ok(if self.advance(s)? { None } else { Some(s) });
        }
        Ok(Some(s + (dsmin - ds)))
    }
}

impl<L: ApproxIntLine> ApproxIntMultiLine<L> {
    /// `MakeMLBetween(Low, High, NbPointsToInsert)`: the sub-line of the points
    /// `low..=high` resampled at a constant 3d arc length with at least
    /// `nb_points_to_insert` new points computed by the SvSurfaces (with their solver
    /// switched on). The result keeps the SvSurfaces only for more than 10 source
    /// intervals; without SvSurfaces, or when too few points were made or the result
    /// turns too sharply in a parameter plane, it is an empty line of indices 1..1;
    /// `line.make_ml_between(1, 20, 10)?`.
    pub fn make_ml_between(
        &self,
        low: i32,
        high: i32,
        nb_points_to_insert: i32,
    ) -> Result<Self, Failure> {
        let Some(sv) = self.ptr_on_sv_surfaces.clone() else {
            return Ok(self.empty_line());
        };
        let save_use_solver: bool = sv.borrow().get_use_solver();
        if !save_use_solver {
            sv.borrow_mut().set_use_solver(true);
        }
        let nb_to_insert: i32 = nb_points_to_insert.max(high - low);
        let mut nb_pnts: i32 = nb_to_insert + high - low + 1;
        let nb_pnts_min: i32 = 2 * (high - low);
        if nb_pnts < nb_pnts_min {
            nb_pnts = nb_pnts_min;
        }
        let temp: Handle<L> = self.resample(&sv, (low, high), nb_pnts)?;
        let code_erreur: i32 = turn_error(&*temp)?;
        sv.borrow_mut().set_use_solver(save_use_solver);
        let count: i32 = temp.nb_pnts()?;
        if count < nb_to_insert + high - low + 1 || code_erreur != 0 {
            return Ok(self.empty_line());
        }
        let keep: Option<SvSurfacesRef> = if high - low > 10 { Some(sv) } else { None };
        Ok(Self::with_sv_surfaces(temp, keep, self.layout(1, count)))
    }

    /// The empty line of the failures of `MakeMLBetween`: no point, no SvSurfaces,
    /// the indices 1 and 1.
    fn empty_line(&self) -> Self {
        let vide1 = Handle::new(IntSurfLineOn2S::new(None));
        Self::with_sv_surfaces(
            Handle::new(L::from_points(&vide1, false)),
            None,
            self.layout(1, 1),
        )
    }

    /// The resampled line of `MakeMLBetween`.
    fn resample(
        &self,
        sv: &SvSurfacesRef,
        (low, high): (i32, i32),
        nb_pnts: i32,
    ) -> Result<Handle<L>, Failure> {
        let line: &L = self.line("MakeMLBetween")?;
        let abs: Abscissae = abscissae(line, low, high)?;
        let smax: f64 = abs.ac(high);
        let ds: f64 = smax / f64::from(nb_pnts - 1);
        let dsmin: f64 = ds * 0.3;
        let mut walk = Resampling {
            line,
            abs,
            high,
            indice: low,
            has_been_inserted: false,
            result: Handle::new(IntSurfLineOn2S::new(None)),
        };
        let mut s: f64 = ds;
        while s < smax && walk.indice < high {
            if walk.advance(s)? {
                break;
            }
            match walk.insert_at(sv, s, (ds, dsmin))? {
                Some(next) => s = next + ds,
                None => break,
            }
        }
        walk.add_line_point(high)?;
        Ok(Handle::new(L::from_points(&walk.result, false)))
    }
}

/// The parameter arrays and the cumulated distances between the points (the native
/// arrays of bounds `Low..High`): `High < Low - 1` cannot be allocated, and the write
/// of `U1(Low)` into the empty arrays of `High == Low - 1` raises Standard_OutOfRange.
fn abscissae<L: ApproxIntLine>(line: &L, low: i32, high: i32) -> Result<Abscissae, Failure> {
    if high < low {
        let kind: &str = if high < low - 1 {
            "Standard_OutOfMemory"
        } else {
            "Standard_OutOfRange"
        };
        return Err(Failure::with_message(
            kind,
            &format!(
                "ApproxInt_MultiLine::MakeMLBetween: bounds {low}..{high}, expected Low <= High"
            ),
        ));
    }
    let first = line.point(low)?;
    let (u1, v1, u2, v2) = first.parameters();
    let mut abs = Abscissae {
        low,
        uv: vec![[u1, v1, u2, v2]],
        ac: vec![0.0],
    };
    let mut previous = first;
    for i in low + 1..=high {
        let point = line.point(i)?;
        let (u1, v1, u2, v2) = point.parameters();
        abs.uv.push([u1, v1, u2, v2]);
        let last: f64 = abs.ac[abs.ac.len() - 1];
        abs.ac
            .push(last + previous.value().distance(&point.value()));
        previous = point;
    }
    Ok(abs)
}

/// The a-posteriori check of `MakeMLBetween`: 1 (or 2) when three consecutive points
/// turn too sharply on the first (or second) surface, else 0; Standard_OutOfRange when
/// the line has less than two points.
fn turn_error<L: ApproxIntLine>(temp: &L) -> Result<i32, Failure> {
    let (u1, v1, u2, v2) = temp.point(1)?.parameters();
    let (mut p1a, mut p2a) = (Point2::new(u1, v1), Point2::new(u2, v2));
    let (u1, v1, u2, v2) = temp.point(2)?.parameters();
    let (mut p1b, mut p2b) = (Point2::new(u1, v1), Point2::new(u2, v2));
    for i in 3..=temp.nb_pnts()? {
        let (u1, v1, u2, v2) = temp.point(i)?.parameters();
        let (p1c, p2c) = (Point2::new(u1, v1), Point2::new(u2, v2));
        if sharp_turn(p1a, p1b, p1c) {
            return Ok(1);
        }
        if sharp_turn(p2a, p2b, p2c) {
            return Ok(2);
        }
        (p1a, p2a, p1b, p2b) = (p1b, p2b, p1c, p2c);
    }
    Ok(0)
}

/// Whether `c` is further than half the step `a -> b` from `b + (b - a)`.
fn sharp_turn(a: Point2, b: Point2, c: Point2) -> bool {
    let mut du: f64 = b.x() - a.x();
    let mut dv: f64 = b.y() - a.y();
    let duv2: f64 = 0.25 * (du * du + dv * dv);
    let u: f64 = b.x() + du;
    let v: f64 = b.y() + dv;
    du = c.x() - u;
    dv = c.y() - v;
    du * du + dv * dv > duv2
}
