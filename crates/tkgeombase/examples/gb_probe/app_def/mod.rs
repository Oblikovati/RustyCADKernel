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

//! The Rust twin of tools/oracle/tkgeombase_probe_appdef.cxx and
//! tkgeombase_probe_appdef_variational.cxx.
mod criteria;
mod line;
mod variational;

pub use criteria::criteria_case;
pub use line::line_case;
pub use variational::var_case;

use super::app_par_curves::{dump, point, read_line};
use super::approx::{build_compute, compute_results, constraint, read_options};
use super::common::{self as p, Reader, attempt, end};
use super::fem_tool::read_vector;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::app_def::{Compute, MultiPointConstraint};
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};

/// An array of `values` from `lower`.
pub fn array<T: Clone + Default>(lower: i32, values: &[T]) -> Array1<'static, T> {
    let mut a: Array1<'static, T> = Array1::with_size(values.len());
    a.update_lower_bound(lower);
    a.as_mut_slice().clone_from_slice(values);
    a
}

/// The points and vector arrays of an `ad_mpc` request.
struct MpcArrays {
    p3: Vec<Point3>,
    p2: Vec<Point2>,
    v3: Vec<Vector3>,
    v2: Vec<Vector2>,
    c3: Vec<Vector3>,
    c2: Vec<Vector2>,
}

fn read_arrays(r: &mut Reader<'_>, counts: [i32; 6]) -> MpcArrays {
    let [n3, n2, nv3, nv2, nc3, nc2] = counts;
    MpcArrays {
        p3: (0..n3).map(|_| r.pnt()).collect(),
        p2: (0..n2).map(|_| r.pnt2d()).collect(),
        v3: (0..nv3).map(|_| r.vec()).collect(),
        v2: (0..nv2).map(|_| r.vec2d()).collect(),
        c3: (0..nc3).map(|_| r.vec()).collect(),
        c2: (0..nc2).map(|_| r.vec2d()).collect(),
    }
}

/// `gb_ad_read_mpc`: the constructor of `kind` over arrays starting at `lower`.
fn read_mpc(r: &mut Reader<'_>) -> Result<MultiPointConstraint, Failure> {
    let (kind, lower) = (r.integer(), r.integer());
    let counts = [0; 6].map(|_| r.integer());
    let a = read_arrays(r, counts);
    let (p3, p2) = (array(lower, &a.p3), array(lower, &a.p2));
    let (v3, v2, c3, c2) = (
        array(lower, &a.v3),
        array(lower, &a.v2),
        array(lower, &a.c3),
        array(lower, &a.c2),
    );
    Ok(match kind {
        0 => MultiPointConstraint::new(),
        1 => MultiPointConstraint::with_counts(counts[0], counts[1])?,
        2 => MultiPointConstraint::from_points(&p3),
        3 => MultiPointConstraint::from_points2d(&p2),
        4 => MultiPointConstraint::from_points_and_2d(&p3, &p2),
        5 => MultiPointConstraint::with_tangents_and_curvatures(&p3, &p2, &v3, &v2, &c3, &c2)?,
        6 => MultiPointConstraint::with_tangents(&p3, &p2, &v3, &v2)?,
        7 => MultiPointConstraint::with_tangents_and_curvatures_3d(&p3, &v3, &c3)?,
        8 => MultiPointConstraint::with_tangents_3d(&p3, &v3)?,
        9 => MultiPointConstraint::with_tangents_2d(&p2, &v2)?,
        _ => MultiPointConstraint::with_tangents_and_curvatures_2d(&p2, &v2, &c2)?,
    })
}

/// `gb_ad_mpc`: the points and the two flags.
pub fn mpc(m: &MultiPointConstraint) -> Result<(), Failure> {
    point(m)?;
    p::flag(m.is_tangency_point());
    p::flag(m.is_curvature_point());
    Ok(())
}

fn mpc_command(r: &mut Reader<'_>, m: &mut MultiPointConstraint) -> Result<(), Failure> {
    let code = r.integer();
    match code {
        1 => {
            let i = r.integer();
            m.set_tang(i, r.vec())
        }
        2 => m.tang(r.integer()).map(p::vec),
        3 => {
            let i = r.integer();
            m.set_tang2d(i, r.vec2d())
        }
        4 => m.tang2d(r.integer()).map(p::vec2d),
        5 => {
            let i = r.integer();
            m.set_curv(i, r.vec())
        }
        6 => m.curv(r.integer()).map(p::vec),
        7 => {
            let i = r.integer();
            m.set_curv2d(i, r.vec2d())
        }
        8 => m.curv2d(r.integer()).map(p::vec2d),
        9 => {
            p::flag(m.is_tangency_point());
            p::flag(m.is_curvature_point());
            Ok(())
        }
        10 => {
            let mut text = String::new();
            let _ = m.dump(&mut text);
            dump(&text);
            Ok(())
        }
        11 => {
            let mut copy = m.clone();
            let (i, v) = (r.integer(), r.vec());
            attempt(|| copy.set_tang(i, v));
            p::flag(m.is_tangency_point());
            attempt(|| m.tang(i).map(p::vec));
            Ok(())
        }
        _ => mpc(m),
    }
}

pub fn mpc_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let mut m = read_mpc(&mut r)?;
        mpc(&m)?;
        let count = r.integer();
        for _ in 0..count {
            attempt(|| mpc_command(&mut r, &mut m));
        }
        Ok(())
    });
    end();
}

/// `ad_compute line variant options firstC lastC params repeat`: AppDef_Compute.
pub fn compute_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let line = read_line(&mut r);
    let variant = r.integer();
    let o = read_options(&mut r);
    let (fc, lc) = (constraint(r.integer()), constraint(r.integer()));
    let u = read_vector(&mut r);
    let repeat = r.flag();
    attempt(|| {
        let mut c: Compute = build_compute(&line, variant, o, &u)?;
        if variant >= 2 {
            c.set_constraints(fc, lc);
            c.perform(&line)?;
        }
        compute_results(&mut c)?;
        if repeat {
            let mut again = o;
            again.degree_max += 1;
            again.tolerance_3d *= 10.;
            again.tolerance_2d *= 10.;
            c.init(again);
            c.perform(&line)?;
            compute_results(&mut c)?;
        }
        Ok(())
    });
    end();
}
