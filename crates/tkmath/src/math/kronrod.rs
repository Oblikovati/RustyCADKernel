// Created on: 2005-12-21
// Created by: Julia GERASIMOVA
// Copyright (c) 2005-2014 OPEN CASCADE SAS
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

//! Kronrod quadrature generated from the native Jacobi-Kronrod recurrence.
use super::{EigenValuesSearcher, Vector};
use rustycad_tkernel::{ncollection::Array1, standard::Failure};
mod recurrence;
/// Computed quadrature nodes and weights; `KronrodPointsAndWeights::new(125)?`.
pub struct KronrodPointsAndWeights {
    points: Vector<'static>,
    weights: Vector<'static>,
    done: bool,
}
impl KronrodPointsAndWeights {
    /// Computes 2*n+1 nodes on [-1,1]; `KronrodPointsAndWeights::new(7)?`.
    pub fn new(order: i32) -> Result<Self, Failure> {
        if order < 1 {
            return Err(Failure::out_of_range(&format!(
                "Kronrod order {order}; expected positive order"
            )));
        }
        let size: i32 = order
            .checked_mul(2)
            .and_then(|n| n.checked_add(1))
            .ok_or_else(|| Failure::out_of_range("Kronrod order overflows i32"))?;
        let (diagonal, subdiagonal): (Array1<'static, f64>, Array1<'static, f64>) =
            recurrence::coefficients(order, size);
        let eigensolver: EigenValuesSearcher = EigenValuesSearcher::new(&diagonal, &subdiagonal)?;
        let mut result: Self = Self {
            points: Vector::with_bounds(1, size)?,
            weights: Vector::with_bounds(1, size)?,
            done: false,
        };
        if eigensolver.is_done() {
            result.collect(&eigensolver)?;
        }
        Ok(result)
    }
    fn collect(&mut self, eigensolver: &EigenValuesSearcher) -> Result<(), Failure> {
        let mut pairs: Vec<(f64, f64)> = Vec::with_capacity(eigensolver.dimension() as usize);
        for index in 1..=eigensolver.dimension() {
            let first: f64 = eigensolver.eigen_vector(index)?[1];
            pairs.push((eigensolver.eigen_value(index)?, 2.0 * first * first));
        }
        pairs.sort_unstable_by(|left, right| {
            left.0
                .partial_cmp(&right.0)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for (offset, (point, weight)) in pairs.into_iter().enumerate() {
            self.points[offset as i32 + 1] = point;
            self.weights[offset as i32 + 1] = weight;
        }
        self.done = true;
        Ok(())
    }
    /// Reports convergence of the Jacobi-Kronrod eigenproblem; `quadrature.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// Copies ascending nodes with one-based bounds; `quadrature.points()`.
    pub fn points(&self) -> Vector<'static> {
        self.points.clone()
    }
    /// Copies the corresponding quadrature weights; `quadrature.weights()`.
    pub fn weights(&self) -> Vector<'static> {
        self.weights.clone()
    }
}
