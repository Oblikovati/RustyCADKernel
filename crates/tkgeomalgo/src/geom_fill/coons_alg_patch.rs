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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_CoonsAlgPatch.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_CoonsAlgPatch`: the algorithmic Coons patch of four boundaries.
use super::GeomFillBoundary;
use crate::law::{LawFunction, LawLinear};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkmath::gp::{Point3, Vector3, Xyz};

/// The Coons patch of four boundaries (`GeomFill_CoonsAlgPatch`): the boundaries
/// blended by two laws (linear from 1 to 0 along the second boundary and from 0 to 1
/// along the first), corrected at the four corners, the mean points of the adjacent
/// boundary ends. The laws are shared state (`SetFunc` changes the patch that the
/// tangent fields hold).
///
/// `let patch = GeomFillCoonsAlgPatch::new([&b1, &b2, &b3, &b4])?; patch.value(0.5, 0.5)?`
pub struct GeomFillCoonsAlgPatch {
    parent: StandardTransient,
    bound: [Handle<dyn GeomFillBoundary>; 4],
    c: [Point3; 4],
    a: SharedState<[Handle<dyn LawFunction>; 2]>,
}
impl_transient!(
    GeomFillCoonsAlgPatch,
    "GeomFill_CoonsAlgPatch",
    StandardTransient,
    parent
);

/// A boundary of the patch, or the Standard_NullObject of a null handle.
fn live(bound: &Handle<dyn GeomFillBoundary>) -> Result<&dyn GeomFillBoundary, Failure> {
    bound
        .get()
        .ok_or_else(|| errors::null_object("GeomFill_CoonsAlgPatch: null boundary"))
}

/// A blending law, or the Standard_NullObject of a null handle.
fn law(function: &Handle<dyn LawFunction>) -> Result<&dyn LawFunction, Failure> {
    function
        .get()
        .ok_or_else(|| errors::null_object("GeomFill_CoonsAlgPatch: null blending law"))
}

/// The linear law from `valdeb` at `pdeb` to `valfin` at `pfin`.
fn linear(pdeb: f64, valdeb: f64, pfin: f64, valfin: f64) -> Handle<dyn LawFunction> {
    let function = LawLinear::new();
    function.set(pdeb, valdeb, pfin, valfin);
    Handle::new(function).cast()
}

/// The mean of two points (`temp = A.XYZ().Added(B.XYZ()); temp.Multiply(0.5)`).
fn middle(a: Point3, b: Point3) -> Point3 {
    let temp: Xyz = (a.xyz() + b.xyz()) * 0.5;
    Point3::from(temp)
}

impl GeomFillCoonsAlgPatch {
    /// `GeomFill_CoonsAlgPatch(B1, B2, B3, B4)`: the laws on the ranges of `B2` and
    /// `B1`, and the corners from the boundary ends;
    /// `GeomFillCoonsAlgPatch::new([&b1, &b2, &b3, &b4])?`.
    pub fn new(bounds: [&Handle<dyn GeomFillBoundary>; 4]) -> Result<Self, Failure> {
        let [b1, b2, b3, b4] = bounds.map(live);
        let (b1, b2, b3, b4) = (b1?, b2?, b3?, b4?);
        let (deb1, fin1) = b2.bounds()?;
        let a0 = linear(deb1, 1., fin1, 0.);
        let (deb0, fin0) = b1.bounds()?;
        let a1 = linear(deb0, 0., fin0, 1.);
        let c = [
            middle(b4.value(deb1)?, b1.value(deb0)?),
            middle(b1.value(fin0)?, b2.value(deb1)?),
            middle(b2.value(fin1)?, b3.value(fin0)?),
            middle(b3.value(deb0)?, b4.value(fin1)?),
        ];
        Ok(Self {
            parent: StandardTransient::default(),
            bound: bounds.map(Handle::clone),
            c,
            a: SharedState::new([a0, a1]),
        })
    }
    /// `SetFunc(f1, f2)`: the two blending laws; `patch.set_func(&f1, &f2)`.
    pub fn set_func(&self, f1: &Handle<dyn LawFunction>, f2: &Handle<dyn LawFunction>) {
        *self.a.write() = [f1.clone(), f2.clone()];
    }
    /// `Func(f1, f2)`: the two blending laws; `let (f1, f2) = patch.func_pair();`.
    pub fn func_pair(&self) -> (Handle<dyn LawFunction>, Handle<dyn LawFunction>) {
        let a = self.a.read();
        (a[0].clone(), a[1].clone())
    }
    /// `Func(I)`: blending law `i` (0 or 1); the native array is read without a
    /// check, Standard_OutOfRange here; `patch.func(0)?`.
    pub fn func(&self, i: i32) -> Result<Handle<dyn LawFunction>, Failure> {
        let a = self.a.read();
        slot(&*a, i, "Func").cloned()
    }
    /// `Bound(I)`: boundary `i` (0 to 3), Standard_OutOfRange outside;
    /// `patch.bound(2)?`.
    pub fn bound(&self, i: i32) -> Result<Handle<dyn GeomFillBoundary>, Failure> {
        slot(&self.bound, i, "Bound").cloned()
    }
    /// `Corner(I)`: corner `i` (0 to 3), Standard_OutOfRange outside;
    /// `patch.corner(0)?`.
    pub fn corner(&self, i: i32) -> Result<Point3, Failure> {
        slot(&self.c, i, "Corner").copied()
    }
    /// The laws' values at `(V, U)`, read once per evaluation.
    fn laws(&self) -> [Handle<dyn LawFunction>; 2] {
        self.a.read().clone()
    }
    /// The corner correction `-a0 a3 c0 - a0 a1 c1 - a1 a2 c2 - a2 a3 c3` added to
    /// `cor`, term by term as natively.
    fn add_corners(&self, mut cor: Xyz, [a0, a1, a2, a3]: [f64; 4]) -> Xyz {
        cor += self.c[0].xyz() * (-a0 * a3);
        cor += self.c[1].xyz() * (-a0 * a1);
        cor += self.c[2].xyz() * (-a1 * a2);
        cor + self.c[3].xyz() * (-a2 * a3)
    }
    /// `Value(U, V)`. Quirk kept: every boundary is evaluated at `V`.
    pub fn value(&self, u: f64, v: f64) -> Result<Point3, Failure> {
        let [f0, f1] = self.laws();
        let a0: f64 = law(&f0)?.value(v)?;
        let a1: f64 = law(&f1)?.value(u)?;
        let (a2, a3) = (1. - a0, 1. - a1);
        let mut cor: Xyz = self.bound_value(0, v)? * a0;
        cor += self.bound_value(1, v)? * a1;
        cor += self.bound_value(2, v)? * a2;
        cor += self.bound_value(3, v)? * a3;
        Ok(Point3::from(self.add_corners(cor, [a0, a1, a2, a3])))
    }
    /// `D1U(U, V)`: the derivative in U.
    pub fn d1u(&self, u: f64, v: f64) -> Result<Vector3, Failure> {
        let [f0, f1] = self.laws();
        let a0: f64 = law(&f0)?.value(v)?;
        let (_, a1) = law(&f1)?.d1(u)?;
        let (a2, a3) = (1. - a0, -a1);
        let mut cor: Xyz = self.bound_d1(0, u)? * a0;
        cor += self.bound_value(1, v)? * a1;
        cor += self.bound_d1(2, u)? * a2;
        cor += self.bound_value(3, v)? * a3;
        Ok(Vector3::from(self.add_corners(cor, [a0, a1, a2, a3])))
    }
    /// `D1V(U, V)`: the derivative in V.
    pub fn d1v(&self, u: f64, v: f64) -> Result<Vector3, Failure> {
        let [f0, f1] = self.laws();
        let (_, a0) = law(&f0)?.d1(v)?;
        let a1: f64 = law(&f1)?.value(u)?;
        let (a2, a3) = (-a0, 1. - a1);
        let mut cor: Xyz = self.bound_value(0, u)? * a0;
        cor += self.bound_d1(1, v)? * a1;
        cor += self.bound_value(2, u)? * a2;
        cor += self.bound_d1(3, v)? * a3;
        Ok(Vector3::from(self.add_corners(cor, [a0, a1, a2, a3])))
    }
    /// `DUV(U, V)`: the mixed derivative.
    pub fn duv(&self, u: f64, v: f64) -> Result<Vector3, Failure> {
        let [f0, f1] = self.laws();
        let (_, a0) = law(&f0)?.d1(v)?;
        let (_, a1) = law(&f1)?.d1(u)?;
        let (a2, a3) = (-a0, -a1);
        let mut cor: Xyz = self.bound_d1(0, u)? * a0;
        cor += self.bound_d1(1, v)? * a1;
        cor += self.bound_d1(2, u)? * a2;
        cor += self.bound_d1(3, v)? * a3;
        Ok(Vector3::from(self.add_corners(cor, [a0, a1, a2, a3])))
    }
    fn bound_value(&self, k: usize, t: f64) -> Result<Xyz, Failure> {
        Ok(live(&self.bound[k])?.value(t)?.xyz())
    }
    fn bound_d1(&self, k: usize, t: f64) -> Result<Xyz, Failure> {
        Ok(*live(&self.bound[k])?.d1(t)?.1.xyz())
    }
}

/// `array[i]` of a fixed native array, Standard_OutOfRange outside it.
fn slot<'s, T>(array: &'s [T], i: i32, member: &str) -> Result<&'s T, Failure> {
    usize::try_from(i)
        .ok()
        .and_then(|k| array.get(k))
        .ok_or_else(|| {
            errors::out_of_range(&format!(
                "GeomFill_CoonsAlgPatch::{member}: index {i}; expected 0..{}",
                array.len()
            ))
        })
}
