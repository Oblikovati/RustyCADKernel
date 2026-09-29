// Copyright (c) 1996-1999 Matra Datavision
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

//! The `ad_var` requests: AppDef_Variational.
use super::super::app_par_curves::{dump, read_constraints, read_line};
use super::super::approx::{array, bspline};
use super::super::common::{self as p, Reader, attempt, end};
use super::super::fem_tool::{matrix, read_vector, shape};
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeombase::app_def::{Variational, VariationalOptions};
use rustycad_tkmath::math::Matrix;

fn harray(a: &Handle<HArray1<f64>>) {
    array(&a.array1());
}

fn read_harray(r: &mut Reader<'_>) -> Handle<HArray1<f64>> {
    let v = read_vector(r);
    let values: Vec<f64> = (v.lower()..=v.upper()).map(|i| v[i]).collect();
    Handle::new(HArray1::from_array(&Array1::from_shared(
        &values,
        v.lower(),
    )))
}

fn flags(v: &Variational) {
    p::flag(v.is_created());
    p::flag(v.is_done());
    p::flag(v.is_over_constrained());
}

fn settings(v: &Variational) {
    p::int(v.max_degree());
    p::int(v.max_segment());
    p::int(v.continuity() as i32);
    p::flag(v.with_min_max());
    p::flag(v.with_cutting());
    p::real(v.tolerance());
    p::int(v.nb_iterations());
    v.criterium_weight().into_iter().for_each(p::real);
}

fn results(v: &Variational) {
    attempt(|| bspline(&v.value()?));
    attempt(|| {
        let e = v.max_error()?;
        p::real(e);
        if e != 0. {
            p::int(v.max_error_index()?);
        }
        Ok(())
    });
    attempt(|| v.quadratic_error().map(p::real));
    attempt(|| v.average_error().map(p::real));
    attempt(|| v.parameters().map(harray));
    attempt(|| v.knots().map(harray));
    attempt(|| {
        v.criterium()?.into_iter().for_each(p::real);
        Ok(())
    });
}

fn var_command(r: &mut Reader<'_>, v: &mut Variational, npoints: i32) -> Result<(), Failure> {
    let code = r.integer();
    match code {
        1 => {
            v.approximate()?;
            flags(v);
            Ok(())
        }
        2 => {
            settings(v);
            Ok(())
        }
        3 => {
            results(v);
            Ok(())
        }
        4 => {
            let (lr, lc, rows) = (r.integer(), r.integer(), r.integer());
            let mut m = Matrix::filled(lr, lr + rows - 1, lc, lc + npoints - 1, -7.).unwrap();
            v.distance(&mut m)?;
            matrix(&m);
            Ok(())
        }
        5 => {
            if v.is_done() && v.max_error()? == 0. {
                p::int(-1);
            } else {
                let mut text = String::new();
                let _ = v.dump(&mut text);
                dump(&text);
            }
            Ok(())
        }
        6 => v.set_constraints(&read_constraints(r)).map(p::flag),
        7 => v.set_parameters(&read_harray(r)),
        8 => v.set_knots(&read_harray(r)).map(p::flag),
        9 => v.set_max_degree(r.integer()).map(p::flag),
        10 => {
            p::flag(v.set_max_segment(r.integer()));
            Ok(())
        }
        11 => v.set_continuity(shape(r.integer())).map(p::flag),
        12 => v.set_with_min_max(r.flag()),
        13 => v.set_with_cutting(r.flag()).map(p::flag),
        14 => {
            let (a, b, c) = (r.real(), r.real(), r.real());
            v.set_criterium_weight(a, b, c)
        }
        15 => {
            let order = r.integer();
            v.set_criterium_weight_of(order, r.real())
        }
        16 => v.set_tolerance(r.real()),
        _ => {
            v.set_nb_iterations(r.integer());
            Ok(())
        }
    }
}

pub fn var_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let line = read_line(&mut r);
    let (first, last) = (r.integer(), r.integer());
    let constraints = read_constraints(&mut r);
    let options = VariationalOptions {
        max_degree: r.integer(),
        max_segment: r.integer(),
        continuity: shape(r.integer()),
        with_min_max: r.flag(),
        with_cutting: r.flag(),
        tolerance: r.real(),
        nb_iterations: r.integer(),
    };
    let count = r.integer();
    attempt(|| {
        let mut v = Variational::new(&line, first, last, &constraints, options)?;
        flags(&v);
        for _ in 0..count {
            attempt(|| var_command(&mut r, &mut v, last - first + 1));
        }
        Ok(())
    });
    end();
}
