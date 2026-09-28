// Copyright (c) 2025 OPEN CASCADE SAS
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

//! Direct MathLin_EigenSearch_Test.cxx translation at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkmath::math::{EigenValuesSearcher, Matrix, Vector};
use rustycad_tkmath::math_lin::*;
use rustycad_tkmath::math_utils::Status;
fn near(a: f64, b: f64, tol: f64) {
    assert!((a - b).abs() <= tol, "{a} != {b}");
}
fn vector(values: &[f64], lower: i32) -> Vector<'static> {
    let mut out: Vector<'static> =
        Vector::with_bounds(lower, lower + values.len() as i32 - 1).unwrap();
    for (i, v) in values.iter().enumerate() {
        *out.change_at(i).unwrap() = *v;
    }
    out
}
fn sorted(result: &TridiagonalResult) -> Vec<f64> {
    let Some(v) = result.eigen_values.as_ref() else {
        return Vec::new();
    };
    let mut values: Vec<f64> = (0..v.length() as usize)
        .map(|i| *v.at(i).unwrap())
        .collect();
    values.sort_by(f64::total_cmp);
    values
}
fn legacy(diag: &Vector<'_>, sub: &Vector<'_>) -> EigenValuesSearcher {
    let mut d: Array1<'_, f64> = Array1::with_bounds(diag.lower(), diag.upper()).unwrap();
    let mut e: Array1<'_, f64> = Array1::with_bounds(sub.lower(), sub.upper()).unwrap();
    for i in 0..diag.length() as usize {
        d.as_mut_slice()[i] = *diag.at(i).unwrap();
        e.as_mut_slice()[i] = *sub.at(i).unwrap();
    }
    EigenValuesSearcher::new(&d, &e).unwrap()
}
fn sorted_legacy(result: &EigenValuesSearcher) -> Vec<f64> {
    let mut values: Vec<f64> = (1..=result.dimension())
        .map(|i| result.eigen_value(i).unwrap())
        .collect();
    values.sort_by(f64::total_cmp);
    values
}
fn compare_values(old: &EigenValuesSearcher, new: &TridiagonalResult) {
    let a: Vec<f64> = sorted_legacy(old);
    let b: Vec<f64> = sorted(new);
    assert_eq!(a.len(), b.len());
    for (left, right) in a.iter().zip(b) {
        near(*left, right, 1e-10);
    }
}
fn check_pair(diag: &Vector<'_>, sub: &Vector<'_>, result: &TridiagonalResult, index: usize) {
    let v: Vector<'static> = get_eigen_vector(result, index).unwrap();
    let mut norm: f64 = 0.0;
    for i in 0..v.length() {
        norm += v[i] * v[i];
    }
    near(norm.sqrt(), 1.0, 1e-10);
    let lambda: f64 = *result.eigen_values.as_ref().unwrap().at(index).unwrap();
    let mut residual: f64 = 0.0;
    for i in 0..v.length() {
        let mut sum: f64 = 0.0;
        // Keep the full matrix multiplication order used by the legacy helper.
        for j in 0..v.length() {
            let a: f64 = if i == j {
                *diag.at(i as usize).unwrap()
            } else if i == j + 1 {
                *sub.at(i as usize).unwrap()
            } else if j == i + 1 {
                *sub.at(j as usize).unwrap()
            } else {
                0.0
            };
            sum += a * v[j];
        }
        residual = residual.max((sum - lambda * v[i]).abs());
    }
    near(residual, 0.0, 1e-10);
}
#[test]
fn MathLin_EigenSearch_Test__CoexistsWithJacobiInDangerousIncludeOrder() {
    let d: Vector<'_> = vector(&[1.0, 2.0], 0);
    let e: Vector<'_> = vector(&[0.0, 0.5], 0);
    let tridiagonal = eigen_tridiagonal(&d, &e, 30);
    let mut a: Matrix<'_> = Matrix::filled(0, 1, 0, 1, 0.0).unwrap();
    a[(0, 0)] = 1.0;
    a[(0, 1)] = 0.5;
    a[(1, 0)] = 0.5;
    a[(1, 1)] = 2.0;
    let result = jacobi(&a, true);
    assert!(tridiagonal.is_done());
    assert!(result.is_done());
}
#[test]
fn MathLin_EigenSearch_Test__AcceptsDocumentedSubdiagonalShape() {
    let d = vector(&[2.0; 3], 0);
    let e = vector(&[1.0; 2], 0);
    let r = eigen_tridiagonal(&d, &e, 30);
    assert!(r.is_done());
    assert_eq!(r.dimension, 3);
    assert_eq!(r.eigen_values.as_ref().unwrap().length(), 3);
    let values = sorted(&r);
    near(values[0], 2.0 - 2.0_f64.sqrt(), 1e-10);
    near(values[1], 2.0, 1e-10);
    near(values[2], 2.0 + 2.0_f64.sqrt(), 1e-10);
}
#[test]
fn MathLin_EigenSearch_Test__LargeFiniteDiagonalDoesNotDeflateCoupling() {
    let magnitude: f64 = f64::MAX / 2.0;
    let d = vector(&[magnitude; 2], 0);
    let e = vector(&[magnitude / 4.0], 0);
    let r = eigen_tridiagonal(&d, &e, 30);
    assert!(r.is_done());
    let v = sorted(&r);
    assert_eq!(v.len(), 2);
    near(v[0] / magnitude, 0.75, 1e-12);
    near(v[1] / magnitude, 1.25, 1e-12);
}
#[test]
fn MathLin_EigenSearch_Test__InputAndIterationStatuses() {
    let one = vector(&[7.0], 0);
    let empty = Vector::with_size(0);
    let single = eigen_tridiagonal(&one, &empty, 30);
    assert!(single.is_done());
    assert_eq!(*single.eigen_values.as_ref().unwrap().at(0).unwrap(), 7.0);
    let wrong = vector(&[0.0; 2], 0);
    assert_eq!(
        eigen_tridiagonal(&one, &wrong, 30).status,
        Status::InvalidInput
    );
    assert_eq!(
        eigen_tridiagonal(&one, &empty, 0).status,
        Status::InvalidInput
    );
    let d = vector(&[2.0; 4], 0);
    let e = vector(&[1.0; 3], 0);
    assert_eq!(eigen_tridiagonal(&d, &e, 1).status, Status::MaxIterations);
}
#[test]
fn MathLin_EigenSearch_Test__BasicParityWithLegacy_3x3() {
    let d = vector(&[4.0; 3], 0);
    let e = vector(&[0.0, 1.0, 1.0], 0);
    let old = legacy(&d, &e);
    let new = eigen_tridiagonal(&d, &e, 30);
    assert!(old.is_done());
    assert!(new.is_done());
    assert!(new.eigen_values.is_some());
    assert!(new.eigen_vectors.is_some());
    compare_values(&old, &new);
    for i in 0..new.dimension {
        check_pair(&d, &e, &new, i);
    }
}
#[test]
fn MathLin_EigenSearch_Test__HandlesNonOneLowerBounds() {
    let d = vector(&[1.0, 3.0, -2.0, 5.0, 4.0], -2);
    let e = vector(&[0.0, 0.3, -0.2, 0.7, -1.1], -2);
    let old = legacy(&d, &e);
    let new = eigen_tridiagonal(&d, &e, 30);
    assert_eq!(old.is_done(), new.is_done());
    assert!(old.is_done());
    compare_values(&old, &new);
}
fn orthogonality(result: &TridiagonalResult) {
    for i in 0..result.dimension {
        let left = get_eigen_vector(result, i).unwrap();
        for j in i + 1..result.dimension {
            let right = get_eigen_vector(result, j).unwrap();
            let mut dot: f64 = 0.0;
            for k in 0..left.length() {
                dot += left[k] * right[k];
            }
            near(dot, 0.0, 1e-10);
        }
    }
}
#[test]
fn MathLin_EigenSearch_Test__RandomParityAndOrthogonality() {
    // Exact libstdc++ mt19937(123456) distributions from the pinned native test.
    let inputs: &str = include_str!("mathlin_eigen_search/random_inputs.txt");
    assert_eq!(inputs.lines().count(), 120);
    for line in inputs.lines() {
        let entries: Vec<f64> = line
            .split_whitespace()
            .map(|v| v.parse().unwrap())
            .collect();
        let n: usize = entries[0] as usize;
        let d = vector(&(0..n).map(|i| entries[1 + 2 * i]).collect::<Vec<_>>(), 0);
        let e = vector(&(0..n).map(|i| entries[2 + 2 * i]).collect::<Vec<_>>(), 0);
        let old = legacy(&d, &e);
        let new = eigen_tridiagonal(&d, &e, 30);
        assert_eq!(old.is_done(), new.is_done());
        if !old.is_done() {
            continue;
        }
        assert!(new.eigen_values.is_some());
        assert!(new.eigen_vectors.is_some());
        compare_values(&old, &new);
        for i in 0..n {
            check_pair(&d, &e, &new, i);
        }
        orthogonality(&new);
    }
}
