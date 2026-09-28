// Created on: 2013-12-20
// Created by: Denis BOGOLEPOV
// Copyright (c) 2013-2014 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKMath/BVH/BVH_Properties.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_Properties` and its transformation implementation `BVH_Transform<T, 4>`.
use super::bounding_box::BvhBox;
use super::scalar::BvhScalar;
use super::vector::BvhVector;
use rustycad_tkernel::ncollection::{Mat4, MatrixScalar, Vec4};
use std::any::Any;
use std::fmt::Debug;

/// Abstract properties of a geometric object, shared through a handle.
///
/// `as_any` stands in for `Handle(...)::DownCast`; `properties.as_any().downcast_ref::<BvhTransform<f64>>()`.
pub trait BvhProperties: Any + Debug + Send + Sync {
    /// Returns the object for run-time downcasting.
    fn as_any(&self) -> &dyn Any;
}

/// Transform properties of a geometric object (the only `BVH_MatrixType` is 4x4).
///
/// The matrix constructor does NOT compute the inverse; only `set_transform`
/// does, as upstream. Example: `BvhTransform::<f64>::default().transform().is_identity()`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BvhTransform<T> {
    transform: Mat4<T>,
    transform_inversed: Mat4<T>,
}
impl<T: MatrixScalar> Default for BvhTransform<T> {
    fn default() -> Self {
        Self {
            transform: Mat4::default(),
            transform_inversed: Mat4::default(),
        }
    }
}
impl<T> BvhTransform<T>
where
    T: BvhScalar + MatrixScalar,
    Vec4<T>: BvhVector<Scalar = T>,
{
    /// Stores the matrix; the inverse stays identity; `BvhTransform::new(matrix)`.
    pub fn new(transform: Mat4<T>) -> Self {
        Self {
            transform,
            transform_inversed: Mat4::default(),
        }
    }
    /// Returns transformation matrix; `properties.transform()`.
    pub fn transform(&self) -> &Mat4<T> {
        &self.transform
    }
    /// Stores the matrix and its inverse (`BVH::MatrixOp::Inverse`). A singular
    /// matrix leaves the unscaled cofactors, as `Inverted(out)` does upstream.
    /// Example: `properties.set_transform(matrix)`.
    pub fn set_transform(&mut self, transform: Mat4<T>) {
        self.transform = transform;
        matrix_op_inverse(&self.transform, &mut self.transform_inversed);
    }
    /// Returns inversed transformation matrix; `properties.inversed()`.
    pub fn inversed(&self) -> &Mat4<T> {
        &self.transform_inversed
    }
    /// Box of the eight projectively transformed corners (`Apply`).
    /// Example: `properties.apply(&bounding)`.
    pub fn apply(&self, bounding: &BvhBox<Vec4<T>>) -> BvhBox<Vec4<T>> {
        let size = bounding.size();
        let mut result = BvhBox::default();
        for x in 0..=1 {
            for y in 0..=1 {
                for z in 0..=1 {
                    let corner = *bounding.corner_min()
                        + Vec4::unit_x() * size * T::from_int(x)
                        + Vec4::unit_y() * size * T::from_int(y)
                        + Vec4::unit_z() * size * T::from_int(z);
                    result.add(&matrix_op_multiply(&self.transform, &corner));
                }
            }
        }
        result
    }
}

/// `BVH::MatrixOp<T, 4>::Inverse`: `theIn.Inverted(theOut)` with the success flag
/// discarded, so a singular matrix leaves the unscaled cofactors in `output`.
/// Example: `bvh::matrix_op_inverse(&matrix, &mut inverse)`.
pub fn matrix_op_inverse<T>(input: &Mat4<T>, output: &mut Mat4<T>)
where
    T: BvhScalar + MatrixScalar,
{
    // The one-argument `Inverted` overload forwards a dummy determinant.
    let mut determinant = T::ZERO;
    input.inverted_into(output, &mut determinant);
}

/// `BVH::MatrixOp<T, 4>::Multiply`: `M * v` scaled by `(T)(1.0 / w)`, the
/// reciprocal being evaluated in double precision as the literal `1.0` forces.
/// Example: `bvh::matrix_op_multiply(&matrix, &Vec4::new(1.0, 2.0, 3.0, 1.0))`.
pub fn matrix_op_multiply<T>(matrix: &Mat4<T>, vector: &Vec4<T>) -> Vec4<T>
where
    T: BvhScalar + MatrixScalar,
    Vec4<T>: BvhVector<Scalar = T>,
{
    let product = *matrix * *vector;
    product * T::from_double(1.0 / product.w.to_double())
}
impl<T> BvhProperties for BvhTransform<T>
where
    T: MatrixScalar + Debug + Send + Sync + 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
}
