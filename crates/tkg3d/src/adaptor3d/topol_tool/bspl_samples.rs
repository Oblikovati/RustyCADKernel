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
// Rust translation of ModelingData/TKG3d/Adaptor3d/Adaptor3d_TopolTool.cxx (BSplSamplePnts) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BSplSamplePnts`: sample parameters of a B-spline surface, `Degree` per knot
//! span of the adaptor range (or a uniform grid when a minimum count asks for more),
//! thinned by a deflection test along every row and column.
use super::super::surface::Adaptor3dSurface;
use super::Adaptor3dTopolTool;
use crate::geom::GeomBSplineSurface;
use crate::geom::errors::{array_bounds, null_object};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Direction3, Line3, Point3, Vector3};

const MAX_PNTS: i32 = 1001;
/// `myMinPnts`: the absolute minimum of sample points (a restriction of IntPolyh).
const MIN_PNTS: i32 = 4;

/// The knots of one direction: `(Knot(i), FirstKnotIndex, LastKnotIndex, Degree)`.
struct KnotLine {
    knots: Vec<f64>,
    first: i32,
    degree: i32,
}
impl KnotLine {
    fn knot(&self, index: i32) -> f64 {
        self.knots[(index - self.first) as usize]
    }
    fn last(&self) -> i32 {
        self.first + self.knots.len() as i32 - 1
    }
    // The first and last knot indices of the spans that [inf, sup] touches.
    fn span_range(&self, inf: f64, sup: f64) -> (i32, i32) {
        let (mut i1, mut i2) = (self.first, self.last());
        for i in i1..i2 {
            if inf >= self.knot(i) && inf < self.knot(i + 1) {
                i1 = i;
                break;
            }
        }
        for i in (i1 + 1..=i2).rev() {
            if sup <= self.knot(i) && sup > self.knot(i - 1) {
                i2 = i;
                break;
            }
        }
        (i1, i2)
    }
}

fn knot_lines(surface: &GeomBSplineSurface) -> Result<(KnotLine, KnotLine), Failure> {
    let line = |first: i32, last: i32, knot: &dyn Fn(i32) -> Result<f64, Failure>, degree: i32| {
        let knots: Result<Vec<f64>, Failure> = (first..=last).map(knot).collect();
        Ok::<KnotLine, Failure>(KnotLine {
            knots: knots?,
            first,
            degree,
        })
    };
    let u = line(
        surface.first_u_knot_index()?,
        surface.last_u_knot_index()?,
        &|i| surface.u_knot(i),
        surface.u_degree(),
    )?;
    let v = line(
        surface.first_v_knot_index()?,
        surface.last_v_knot_index()?,
        &|i| surface.v_knot(i),
        surface.v_degree(),
    )?;
    Ok((u, v))
}

// `Degree` parameters per span from `inf` to `sup` (the knots in between), and the
// last one; the flags stay false.
fn knot_parameters(
    line: &KnotLine,
    (i1, i2): (i32, i32),
    (inf, sup): (f64, f64),
    count: i32,
) -> Result<Array1<'static, f64>, Failure> {
    let mut values: Array1<'static, f64> = Array1::with_bounds(1, count).map_err(array_bounds)?;
    let (mut k, mut t1) = (0, inf);
    for i in i1 + 1..=i2 {
        let t2: f64 = if i == i2 { sup } else { line.knot(i) };
        let dt: f64 = (t2 - t1) / f64::from(line.degree);
        for _ in 0..line.degree {
            k += 1;
            values.set_value(k, t1).map_err(array_bounds)?;
            t1 += dt;
        }
        t1 = t2;
    }
    values.set_value(k + 1, t1).map_err(array_bounds)?;
    Ok(values)
}

/// The sample counts and whether each direction is sampled uniformly, after the
/// minimum counts (the "NIZHNY-EMV 2013" balancing, then the bounds).
fn balanced_counts(
    mut nbsu: i32,
    mut nbsv: i32,
    (nu_min, nv_min): (i32, i32),
) -> (i32, i32, bool, bool) {
    let (mut u_uniform, mut v_uniform) = (false, false);
    if nbsu < nu_min || nbsv < nv_min {
        if nbsu < nbsv {
            let count: i32 =
                ((f64::from(nbsv) * f64::from(nu_min) / f64::from(nbsu)) as i32).min(30);
            v_uniform = count > nbsv;
            nbsv = if v_uniform { count } else { nbsv };
        } else {
            let count: i32 =
                ((f64::from(nbsu) * f64::from(nv_min) / f64::from(nbsv)) as i32).min(30);
            u_uniform = count > nbsu;
            nbsu = if u_uniform { count } else { nbsu };
        }
    }
    (nbsu, u_uniform) = bounded(nbsu, nu_min, u_uniform);
    (nbsv, v_uniform) = bounded(nbsv, nv_min, v_uniform);
    (nbsu, nbsv, u_uniform, v_uniform)
}
fn bounded(count: i32, minimum: i32, uniform: bool) -> (i32, bool) {
    if count < minimum {
        return (minimum, true);
    }
    if count > MAX_PNTS {
        return (MAX_PNTS, true);
    }
    (count, uniform)
}

/// The parameters of one direction and its flags (all false).
fn direction_parameters(
    line: &KnotLine,
    range: (f64, f64),
    count: i32,
    uniform: bool,
) -> Result<(Vec<f64>, Vec<bool>), Failure> {
    let values: Array1<'static, f64> = if uniform {
        super::samples::spread(range.0, range.1, count)?
    } else {
        knot_parameters(line, line.span_range(range.0, range.1), range, count)?
    };
    Ok((
        values.iter().copied().collect(),
        vec![false; count as usize],
    ))
}

/// One deflection pass: along each line at `fixed` parameters, the points of the
/// other direction that a chord does not approach within the deflection are flagged.
struct DeflectionPass<'a> {
    surface: &'a dyn Adaptor3dSurface,
    along_u: bool,
    deflection2: f64,
    tolerance: f64,
}
impl DeflectionPass<'_> {
    fn point(&self, running: f64, fixed: f64) -> Result<Point3, Failure> {
        if self.along_u {
            self.surface.value(running, fixed)
        } else {
            self.surface.value(fixed, running)
        }
    }
    // The flagging of one line; `j` and `k` keep their native meaning across the loops.
    fn line(
        &self,
        fixed: f64,
        pars: &[f64],
        flags: &mut [bool],
        j: &mut i32,
    ) -> Result<(), Failure> {
        let n: i32 = pars.len() as i32;
        let (at, flag) = (
            |i: i32| pars[(i - 1) as usize],
            |f: &[bool], i: i32| f[(i - 1) as usize],
        );
        *j = 1;
        while *j < n - 1 {
            if flag(flags, *j + 1) {
                *j += 1;
                continue;
            }
            let p1: Point3 = self.point(at(*j), fixed)?;
            let mut k: i32 = *j + 2;
            while k <= n {
                let p2: Point3 = self.point(at(k), fixed)?;
                if p1.square_distance(&p2) <= self.tolerance {
                    k += 1;
                    continue;
                }
                let line = Line3::new(p1, Direction3::from_vector(Vector3::between(p1, p2))?);
                if !self.chord_fits(&line, (*j, k), (pars, flags), fixed)? {
                    *j = k - 1;
                    flags[(*j - 1) as usize] = true;
                    break;
                }
                if flag(flags, k) {
                    *j = k;
                    break;
                }
                k += 1;
            }
            if k >= n {
                break;
            }
        }
        Ok(())
    }
    // Whether every unflagged point strictly between j and k is within the deflection.
    fn chord_fits(
        &self,
        line: &Line3,
        (j, k): (i32, i32),
        (pars, flags): (&[f64], &[bool]),
        fixed: f64,
    ) -> Result<bool, Failure> {
        for l in j + 1..k {
            if flags[(l - 1) as usize] {
                return Ok(false);
            }
            let pp: Point3 = self.point(pars[(l - 1) as usize], fixed)?;
            if line.square_distance(pp) > self.deflection2 {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

// Too few flagged samples: add samples, with the native choice (a stale `j` for two
// samples, the middle of the larger segment for three).
fn ensure_minimum(flags: &mut [bool], mut j: i32, count: i32) -> i32 {
    if count >= MIN_PNTS {
        return count;
    }
    let n: i32 = flags.len() as i32;
    if count == 2 {
        // "Uniform" distribution.
        let nn: i32 = n / MIN_PNTS;
        flags[nn as usize] = true;
        flags[(n - nn - 1) as usize] = true;
    } else {
        // Insert in the bigger segment.
        let mut i: i32 = 2;
        loop {
            let set: bool = flags[(i - 1) as usize];
            i += 1;
            if set {
                break;
            }
        }
        j = if i < n / 2 {
            (i + (n - i) / 2).min(n - 1)
        } else {
            (i / 2).max(2)
        };
    }
    flags[(j - 1) as usize] = true;
    MIN_PNTS
}

impl Adaptor3dTopolTool {
    /// `BSplSamplePnts`: the B-spline sample parameters, at least `nu_min` x `nv_min`
    /// (below them the whole candidate set is kept); `tool.bspl_sample_pnts(0.01, 10, 10)?`.
    pub fn bspl_sample_pnts(
        &self,
        deflection: f64,
        nu_min: i32,
        nv_min: i32,
    ) -> Result<(), Failure> {
        let surface: Handle<dyn Adaptor3dSurface> = self.live_surface()?;
        let adapted: &dyn Adaptor3dSurface = &*surface;
        let bspline = adapted.bspline()?;
        let spline = bspline
            .get()
            .ok_or_else(|| null_object("Adaptor3d_TopolTool: BSpline() is null"))?;
        let u_range = (adapted.first_u_parameter()?, adapted.last_u_parameter()?);
        let v_range = (adapted.first_v_parameter()?, adapted.last_v_parameter()?);
        let (u_line, v_line) = knot_lines(spline)?;
        let (ui1, ui2) = u_line.span_range(u_range.0, u_range.1);
        let (vi1, vi2) = v_line.span_range(v_range.0, v_range.1);
        let nbsu: i32 = (ui2 - ui1) * u_line.degree + 1;
        let nbsv: i32 = (vi2 - vi1) * v_line.degree + 1;
        let (nbsu, nbsv, u_uniform, v_uniform) = balanced_counts(nbsu, nbsv, (nu_min, nv_min));
        let (u_pars, mut u_flags) = direction_parameters(&u_line, u_range, nbsu, u_uniform)?;
        let (v_pars, mut v_flags) = direction_parameters(&v_line, v_range, nbsv, v_uniform)?;
        let deflection2: f64 = (deflection * deflection).max(1.0e-9);
        let tolerance: f64 = (0.01 * deflection2).max(1.0e-9);
        let pass = |along_u: bool| DeflectionPass {
            surface: adapted,
            along_u,
            deflection2,
            tolerance,
        };
        let nb_u: i32 = flag_direction(&pass(true), (&v_pars, &u_pars), &mut u_flags)?;
        let nb_v: i32 = flag_direction(&pass(false), (&u_pars, &v_pars), &mut v_flags)?;
        let u_selected = selected(&u_pars, &u_flags, nb_u, nu_min)?;
        let v_selected = selected(&v_pars, &v_flags, nb_v, nv_min)?;
        let mut state = self.state.write();
        (state.nb_samples_u, state.nb_samples_v) =
            (u_selected.len() as i32, v_selected.len() as i32);
        (state.u_pars, state.v_pars) = (Some(u_selected), Some(v_selected));
        Ok(())
    }
}

// Flag the ends, run the pass along every line of the other direction, count the
// flags and ensure the minimum; the flagged count (or MIN_PNTS).
fn flag_direction(
    pass: &DeflectionPass<'_>,
    (lines, pars): (&[f64], &[f64]),
    flags: &mut [bool],
) -> Result<i32, Failure> {
    let n: usize = flags.len();
    flags[0] = true;
    flags[n - 1] = true;
    let mut j: i32 = 1;
    for &fixed in lines {
        pass.line(fixed, pars, flags, &mut j)?;
    }
    let count: i32 = flags.iter().filter(|flag| **flag).count() as i32;
    Ok(ensure_minimum(flags, j, count))
}

// The kept parameters: the flagged ones, or all of them when fewer than the minimum
// are flagged ("NIZNHY-PKV 2011").
fn selected(
    pars: &[f64],
    flags: &[bool],
    count: i32,
    minimum: i32,
) -> Result<Array1<'static, f64>, Failure> {
    let keep_all: bool = count < minimum;
    let size: i32 = if keep_all { pars.len() as i32 } else { count };
    let mut values: Array1<'static, f64> = Array1::with_bounds(1, size).map_err(array_bounds)?;
    let mut j: i32 = 0;
    for (value, flag) in pars.iter().zip(flags) {
        if keep_all || *flag {
            j += 1;
            values.set_value(j, *value).map_err(array_bounds)?;
        }
    }
    Ok(values)
}
