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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/FairCurve/FairCurve_DistributionOfTension.hxx/.lxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `FairCurve_DistributionOfTension`: the local tension `H (|C'| - L)^2 / L` of a
//! batten of active length `L`, with its gradient and Hessian in the poles (and the
//! sliding when it is free).
use super::FairCurveBattenLaw;
use super::distribution::{FairCurveDistributionOfEnergy, LocalBasis, walk_pole_hessian};
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::{Point2, Xy};
use rustycad_tkmath::math::{FunctionSet, Vector};
use std::ops::{Deref, DerefMut};

/// The tension density of a batten.
///
/// With `uniform`, the height is frozen at the law's middle value (used by the
/// minimal-variation energy against parametrization problems);
/// `FairCurveDistributionOfTension::new(10, &flat, &poles, 1, 2.0, &law, 0, false)`.
#[derive(Clone)]
pub struct FairCurveDistributionOfTension {
    density: FairCurveDistributionOfEnergy,
    length_sliding: f64,
    law: FairCurveBattenLaw,
    height: f64,
}

/// The point quantities the gradient and Hessian terms share.
struct TensionPoint {
    cprim: Xy,
    norme_cprim: f64,
    hauteur: f64,
    difference: f64,
}

impl FairCurveDistributionOfTension {
    /// The density of `derivative_order` over the B-spline of order `bspl_order`
    /// (it keeps a copy of `law`); `nb_val_aux` is 1 for a free sliding.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        bspl_order: i32,
        flat_knots: &Handle<HArray1<f64>>,
        poles: &Handle<HArray1<Point2>>,
        derivative_order: i32,
        length_sliding: f64,
        law: &FairCurveBattenLaw,
        nb_val_aux: i32,
        uniform: bool,
    ) -> Self {
        let density = FairCurveDistributionOfEnergy::new(
            bspl_order,
            flat_knots,
            poles,
            derivative_order,
            nb_val_aux,
        );
        // The native constructor ignores the positivity flag of the middle height.
        let height: f64 = if uniform { law.height(0.5).1 } else { 0.0 };
        Self {
            density,
            length_sliding,
            law: *law,
            height,
        }
    }
    /// Changes the active length; `tension.set_length_sliding(2.5)`.
    pub fn set_length_sliding(&mut self, length_sliding: f64) {
        self.length_sliding = length_sliding;
    }
    /// The height at `t`: the frozen one, or the law's (None when not positive).
    fn hauteur(&self, t: f64) -> Option<f64> {
        if self.height > 0.0 {
            return Some(self.height);
        }
        let (positive, hauteur) = self.law.height(t);
        positive.then_some(hauteur)
    }
    /// `Value(TParam, FTension)`: the density, then its derivatives, at `t`.
    fn evaluate(&self, t: f64, f: &mut Vector<'_>) -> bool {
        f.init(0.0);
        let Some(basis) = self.density.local_basis(1, t) else {
            return false;
        };
        let cprim: Xy = basis.derivatives[0];
        let norme_cprim: f64 = cprim.modulus();
        let Some(hauteur) = self.hauteur(t) else {
            return false;
        };
        let difference: f64 = norme_cprim - self.length_sliding;
        let lower: i32 = f.lower();
        f[lower] = hauteur * (difference * difference) / self.length_sliding;
        if self.density.derivative_order >= 1 {
            let point = TensionPoint {
                cprim,
                norme_cprim,
                hauteur,
                difference,
            };
            self.derivatives(&basis, &point, f);
        }
        true
    }
    fn derivatives(&self, basis: &LocalBasis, point: &TensionPoint, f: &mut Vector<'_>) {
        let order: i32 = self.density.bspl_order;
        let mut grad: Vector<'static> =
            Vector::filled(1, 2 * order + self.density.nb_val_aux, 0.0).expect("gradient size");
        let xaux: f64 = point.cprim.x() / point.norme_cprim;
        let yaux: f64 = point.cprim.y() / point.norme_cprim;
        let facteur: f64 = 2.0 * point.hauteur * point.difference / self.length_sliding;
        let mut kk: i32 = f.lower() + basis.first_non_zero();
        for ii in 1..=order {
            grad[2 * ii - 1] = basis.at(2, ii) * xaux;
            f[kk] = facteur * grad[2 * ii - 1];
            grad[2 * ii] = basis.at(2, ii) * yaux;
            f[kk + 1] = facteur * grad[2 * ii];
            kk += 2;
        }
        let last_gradient_index: i32 = self.density.last_gradient_index(f.lower());
        if self.density.nb_val_aux == 1 {
            let ratio: f64 = point.norme_cprim / self.length_sliding;
            let upper: i32 = grad.upper();
            grad[upper] = 1.0 - ratio * ratio;
            f[last_gradient_index] = point.hauteur * grad[upper];
        }
        if self.density.derivative_order >= 2 {
            let frame = [basis.first_non_zero(), last_gradient_index, order];
            self.hessian(basis, point, (&grad, xaux, yaux), frame, f);
        }
    }
    fn hessian(
        &self,
        basis: &LocalBasis,
        point: &TensionPoint,
        (grad, xaux, yaux): (&Vector<'_>, f64, f64),
        frame: [i32; 3],
        f: &mut Vector<'_>,
    ) {
        let n: f64 = point.norme_cprim;
        let facteur_x: f64 = point.difference * (1.0 - xaux * xaux) / n;
        let facteur_y: f64 = point.difference * (1.0 - yaux * yaux) / n;
        let facteur_xy: f64 = -point.difference * xaux * yaux / n;
        let facteur: f64 = 2.0 * point.hauteur / self.length_sliding;
        let term = |a: f64, b: f64, factor: f64, produit: f64| facteur * (a * b + factor * produit);
        walk_pole_hessian(
            f,
            frame,
            |ii, jj| {
                let produit: f64 = basis.at(2, ii / 2) * basis.at(2, jj / 2);
                [
                    term(grad[ii - 1], grad[jj - 1], facteur_x, produit),
                    term(grad[ii], grad[jj - 1], facteur_xy, produit),
                    term(grad[ii - 1], grad[jj], facteur_xy, produit),
                    term(grad[ii], grad[jj], facteur_y, produit),
                ]
            },
            |ii| {
                let b: f64 = basis.at(2, ii / 2);
                let produit: f64 = b * b;
                [
                    term(grad[ii - 1], grad[ii - 1], facteur_x, produit),
                    term(grad[ii], grad[ii - 1], facteur_xy, produit),
                    term(grad[ii], grad[ii], facteur_y, produit),
                ]
            },
        );
        if self.density.nb_val_aux == 1 {
            self.sliding_hessian(basis, point, frame, f);
        }
    }
    /// The Hessian row of the free sliding.
    fn sliding_hessian(
        &self,
        basis: &LocalBasis,
        point: &TensionPoint,
        [first_non_zero, last_gradient_index, order]: [i32; 3],
        f: &mut Vector<'_>,
    ) {
        let l: f64 = self.length_sliding;
        let facteur_x: f64 = -2.0 * point.cprim.x() * point.hauteur / (l * l);
        let facteur_y: f64 = -2.0 * point.cprim.y() * point.hauteur / (l * l);
        let row: i32 = last_gradient_index - f.lower();
        let mut kk: i32 = last_gradient_index + (row - 1) * row / 2 + first_non_zero;
        for ii in 1..=order {
            f[kk] = facteur_x * basis.at(2, ii);
            f[kk + 1] = facteur_y * basis.at(2, ii);
            kk += 2;
        }
        let ratio: f64 = point.norme_cprim / l;
        let upper: i32 = f.upper();
        f[upper] = 2.0 * point.hauteur * (ratio * ratio) / l;
    }
}

impl Deref for FairCurveDistributionOfTension {
    type Target = FairCurveDistributionOfEnergy;
    fn deref(&self) -> &FairCurveDistributionOfEnergy {
        &self.density
    }
}

impl DerefMut for FairCurveDistributionOfTension {
    fn deref_mut(&mut self) -> &mut FairCurveDistributionOfEnergy {
        &mut self.density
    }
}

impl FunctionSet for FairCurveDistributionOfTension {
    fn variables(&self) -> i32 {
        self.density.nb_variables()
    }
    fn equations(&self) -> i32 {
        self.density.nb_equations()
    }
    fn value(&mut self, point: &Vector<'_>, residual: &mut Vector<'_>) -> bool {
        self.evaluate(point[point.lower()], residual)
    }
}
