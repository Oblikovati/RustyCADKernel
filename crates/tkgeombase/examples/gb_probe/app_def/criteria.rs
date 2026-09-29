// Copyright (c) 1998-1999 Matra Datavision
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

//! The `ad_criteria` requests: AppDef_LinearCriteria through its base handle.
use super::super::app_par_curves::read_line;
use super::super::common::{self as p, Reader, attempt, end};
use super::super::fem_tool::{matrix, read_vector, shape, vector};
use rustycad_tkernel::ncollection::{Array1, Array2, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeombase::app_def::{LinearCriteria, SmoothCriterion};
use rustycad_tkgeombase::fem_tool::{AssemblyTable, Curve};
use rustycad_tkmath::math::{Matrix, Vector};
use rustycad_tkmath::plib::HermitJacobi;

/// `gb_ad_read_curve`: `dim nbelm workdeg shape knots... coefficients...`.
fn read_curve(r: &mut Reader<'_>) -> Result<Handle<Curve>, Failure> {
    let (dim, nb, wd, sh) = (r.integer(), r.integer(), r.integer(), r.integer());
    let knots: Vec<f64> = (0..=nb).map(|_| r.real()).collect();
    let coeffs: Vec<f64> = (0..nb * (wd + 1) * dim).map(|_| r.real()).collect();
    let curve = Curve::new(dim, nb, &HermitJacobi::new(wd, shape(sh))?, 1e-7)?;
    for (i, k) in (1..).zip(&knots) {
        curve.knots()[i] = *k;
    }
    let mut values = coeffs.into_iter();
    for el in 1..=nb {
        let mut a = Array2::with_bounds(0, wd, 1, dim).unwrap();
        for i in 0..=wd {
            for d in 1..=dim {
                a[(i, d)] = values.next().unwrap_or(0.);
            }
        }
        curve.set_element(el, &a)?;
    }
    Ok(Handle::new(curve))
}

fn table(t: &AssemblyTable) {
    let cells = t.array2();
    [
        cells.lower_row(),
        cells.upper_row(),
        cells.lower_col(),
        cells.upper_col(),
    ]
    .into_iter()
    .for_each(p::int);
    for i in cells.lower_row()..=cells.upper_row() {
        for j in cells.lower_col()..=cells.upper_col() {
            let a = cells[(i, j)].array1();
            p::int(a.lower());
            p::int(a.upper());
            a.iter().for_each(|&v| p::int(v));
        }
    }
}

fn harray(v: &Vector<'_>) -> Handle<HArray1<f64>> {
    let values: Vec<f64> = (v.lower()..=v.upper()).map(|i| v[i]).collect();
    Handle::new(HArray1::from_array(&Array1::from_shared(
        &values,
        v.lower(),
    )))
}

fn curve_elements(j: &dyn SmoothCriterion) -> Result<(), Failure> {
    let handle = j.curve();
    let c = handle
        .get()
        .ok_or_else(|| Failure::new("probe: null curve"))?;
    for el in 1..=c.nb_elements() {
        p::int(c.degree(el)?);
        let mut a = Array2::with_bounds(0, c.base().work_degree(), 1, c.dimension()).unwrap();
        a.init(-7.);
        c.get_element(el, &mut a)?;
        a.iter().for_each(|&v| p::real(v));
    }
    Ok(())
}

fn estimation(j: &dyn SmoothCriterion) {
    j.get_estimation().into_iter().for_each(p::real);
}

fn criteria_command(r: &mut Reader<'_>, j: &dyn SmoothCriterion) -> Result<(), Failure> {
    let code = r.integer();
    match code {
        1 => {
            j.set_parameters(&harray(&read_vector(r)));
            Ok(())
        }
        2 => j.set_curve(&read_curve(r)?),
        3 => j.set_curve(&j.curve()),
        4 => {
            let (a, b, c) = (r.real(), r.real(), r.real());
            j.set_estimation(a, b, c);
            Ok(())
        }
        5 => {
            estimation(j);
            Ok(())
        }
        6 => {
            j.set_est_length(r.real());
            Ok(())
        }
        7 => {
            p::real(j.est_length());
            Ok(())
        }
        8 => j.assembly_table().map(|t| table(&t)),
        9 => {
            let t = j.dependence_table()?;
            let t = t.array2();
            p::int(t.lower_row());
            p::int(t.upper_row());
            t.iter().for_each(|&v| p::int(v));
            Ok(())
        }
        10 => {
            let (a, b, c) = (r.real(), r.real(), r.real());
            let q = j.quality_values(a, b, c)?;
            p::int(q.icdana);
            q.criteria.into_iter().for_each(p::real);
            estimation(j);
            Ok(())
        }
        11 => {
            let e = j.error_values()?;
            [e.max_error, e.quadratic_error, e.average_error]
                .into_iter()
                .for_each(p::real);
            Ok(())
        }
        12 => {
            let (el, d1, d2, rl, cl, n) = (
                r.integer(),
                r.integer(),
                r.integer(),
                r.integer(),
                r.integer(),
                r.integer(),
            );
            let mut h = Matrix::filled(rl, rl + n - 1, cl, cl + n - 1, -7.).unwrap();
            j.hessian(el, d1, d2, &mut h)?;
            matrix(&h);
            Ok(())
        }
        13 => {
            let (el, d, lower, n) = (r.integer(), r.integer(), r.integer(), r.integer());
            let mut g = Vector::filled(lower, lower + n - 1, -7.)?;
            j.gradient(el, d, &mut g)?;
            vector(&g);
            Ok(())
        }
        14 => {
            let x = read_vector(r);
            j.input_vector(&x, &j.assembly_table()?)?;
            curve_elements(j)
        }
        15 => {
            let w = [0; 5].map(|_| r.real());
            j.set_weight(w[0], w[1], [w[2], w[3], w[4]])
        }
        16 => {
            let (a, b) = j.get_weight();
            p::real(a);
            p::real(b);
            Ok(())
        }
        17 => {
            let v = read_vector(r);
            let values: Vec<f64> = (v.lower()..=v.upper()).map(|i| v[i]).collect();
            j.set_point_weights(&Array1::from_shared(&values, v.lower()));
            Ok(())
        }
        _ => {
            p::flag(j.curve().is_null());
            Ok(())
        }
    }
}

pub fn criteria_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let line = read_line(&mut r);
    let (first, last, count) = (r.integer(), r.integer(), r.integer());
    attempt(|| {
        let j: Handle<dyn SmoothCriterion> =
            Handle::new(LinearCriteria::new(&line, first, last)?).cast();
        let j = j
            .get()
            .ok_or_else(|| Failure::new("probe: null criterion"))?;
        for _ in 0..count {
            attempt(|| criteria_command(&mut r, j));
        }
        Ok(())
    });
    end();
}
