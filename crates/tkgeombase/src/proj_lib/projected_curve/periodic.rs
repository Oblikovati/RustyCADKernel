// Copyright (c) 1997-1999 Matra Datavision
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
// Rust translation of the periodic placement at the end of ProjLib_ProjectedCurve::Perform
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The result moved into the parameter space of a periodic surface: a B-spline by
//! the whole periods that put most of its sample points in the first period, a
//! line by `UFrame`/`VFrame`.
use super::ProjectedState;
use super::perform::{live_curve, live_surface};
use rustycad_tkernel::ncollection::{DataMap, DefaultHashPolicy};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve, Geom2dGeometry};
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Point2, Vector2};

impl ProjectedState {
    /// The check of the result against the periodic parameter space.
    pub(super) fn place_in_periods(&mut self) -> Result<(), Failure> {
        let s = live_surface(self)?;
        let periodic: [bool; 2] = [s.is_u_periodic()?, s.is_v_periodic()?];
        if !self.result.is_done() || !(periodic[0] || periodic[1]) {
            return Ok(());
        }
        let first: [f64; 2] = [s.first_u_parameter()?, s.first_v_parameter()?];
        let mut period: [f64; 2] = [0.0, 0.0];
        if periodic[0] {
            period[0] = s.u_period()?;
        }
        if periodic[1] {
            period[1] = s.v_period()?;
        }
        for idx in 1..=2_usize {
            if !periodic[idx - 1] {
                continue;
            }
            if self.result.get_type() == CurveType::BSplineCurve {
                translate_home(&self.result.bspline(), idx, first[idx - 1], period[idx - 1])?;
            }
            if self.result.get_type() == CurveType::Line {
                let c = live_curve(&self.curve)?;
                let (t1, t2) = (c.first_parameter()?, c.last_parameter()?);
                if idx == 1 {
                    self.result.u_frame(t1, t2, first[0], period[0]);
                } else {
                    self.result.v_frame(t1, t2, first[1], period[1]);
                }
            }
        }
        Ok(())
    }
}

// The number of the period containing coordinate `idx` of each of `deg + 1` samples
// per knot span; the translation by the most frequent one, the first met in the
// NCollection_DataMap iteration order winning a tie.
fn translate_home(
    res: &Handle<Geom2dBSplineCurve>,
    idx: usize,
    first: f64,
    period: f64,
) -> Result<(), Failure> {
    let Some(spline) = res.get() else {
        return Ok(());
    };
    let mut map: DataMap<i32, i32> = DataMap::with_hasher(DefaultHashPolicy, 1);
    let degree: i32 = spline.degree();
    for knot in spline.first_u_knot_index()?..spline.last_u_knot_index()? {
        let (first_param, last_param) = (spline.knot(knot)?, spline.knot(knot + 1)?);
        for int_idx in 0..=degree {
            let param: f64 = first_param
                + (last_param - first_param) * f64::from(int_idx) / (f64::from(degree) + 1.0);
            let coord: f64 = coordinate(spline.value(param)?, idx);
            let mut key: i32 = ((coord - first) / period) as i32;
            if coord - first < 0.0 {
                key -= 1;
            }
            if map.is_bound(&key) {
                *map.change_find(&key)? += 1;
            } else {
                map.bind(key, 1);
            }
        }
    }
    let (mut max_points, mut max_idx): (i32, i32) = (0, 0);
    for (key, value) in map.items() {
        if *value > max_points {
            (max_points, max_idx) = (*value, *key);
        }
    }
    if max_idx == 0 {
        return Ok(());
    }
    let first_pnt: Point2 = spline.value(spline.first_parameter())?;
    let mut second_pnt: Point2 = first_pnt;
    second_pnt.set_coord(
        idx as i32,
        coordinate(first_pnt, idx) - period * f64::from(max_idx),
    )?;
    spline.translate(Vector2::between(first_pnt, second_pnt))
}

fn coordinate(point: Point2, idx: usize) -> f64 {
    if idx == 1 { point.x() } else { point.y() }
}
