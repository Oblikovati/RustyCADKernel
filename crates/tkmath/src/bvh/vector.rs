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
// Rust translation of FoundationClasses/TKMath/BVH/BVH_Types.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH::VectorType<T, N>::Type` and the per-dimension tool classes that use it.
use super::scalar::BvhScalar;
use crate::bnd::BoundingBox3;
use crate::gp::Point3;
use rustycad_tkernel::ncollection::{LinearVector, Mat4, Vec2, Vec3, Vec4};
use std::fmt::Debug;
use std::ops::{Add, Div, Index, IndexMut, Mul, Sub};

/// An `NCollection_Vec2/3/4<T>` usable as the BVH point type `BVH_VecNt`.
///
/// All BVH generics take this one parameter; `T` is [`BvhVector::Scalar`] and `N`
/// is [`BvhVector::DIMENSION`]. Example: `BvhBox::<Vec3<f64>>::default()`.
pub trait BvhVector:
    Copy
    + Debug
    + Default
    + PartialEq
    + Send
    + Sync
    + 'static
    + Index<usize, Output = Self::Scalar>
    + IndexMut<usize>
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Mul<Self::Scalar, Output = Self>
    + Div<Self::Scalar, Output = Self>
{
    /// Numeric data type `T`.
    type Scalar: BvhScalar;
    /// Vector dimension `N`.
    const DIMENSION: usize;
    /// Every component `numeric_limits<T>::max()`: an invalid box minimum.
    const FILLED_MAX: Self;
    /// Every component `numeric_limits<T>::lowest()`: an invalid box maximum.
    const FILLED_LOWEST: Self;
    /// The explicit scalar constructor `BVH_VecNt(value)`; `Vec3::splat(0.0)`.
    fn splat(value: Self::Scalar) -> Self;
    /// NCollection `cwiseMin` (`a < b ? a : b`); `a.componentwise_min(&b)`.
    fn componentwise_min(&self, other: &Self) -> Self;
    /// NCollection `cwiseMax` (`a > b ? a : b`); `a.componentwise_max(&b)`.
    fn componentwise_max(&self, other: &Self) -> Self;
    /// NCollection `Dot` in component order; `a.inner_product(&b)`.
    fn inner_product(&self, other: &Self) -> Self::Scalar;
    /// `BVH::VecComp<T, N>::Get`: any axis past the named ones selects the last
    /// component, as the upstream conditional chain does; `v.component(5)`.
    fn component(&self, axis: i32) -> Self::Scalar;
    /// `BVH::UnitVector<T, N>::DX()`; `Vec3::<f64>::unit_x()`.
    fn unit_x() -> Self;
    /// `BVH::UnitVector<T, N>::DY()`; `Vec3::<f64>::unit_y()`.
    fn unit_y() -> Self;
    /// `BVH::UnitVector<T, N>::DZ()`, the zero vector in 2D; `Vec3::<f64>::unit_z()`.
    fn unit_z() -> Self;
    /// `BVH::ToBndBox`: widens to a `Bnd_Box`, 2D with zero Z, 4D dropping W.
    /// Example: `Vec3::to_bnd_box(&min, &max)`.
    fn to_bnd_box(min: &Self, max: &Self) -> BoundingBox3;
}

/// `BVH::ToBndBox(const T&, const T&)`: the scalar (N = 1) overload, placing the
/// interval on the X axis with zero Y and Z.
/// Example: `bvh::to_bnd_box_scalar(1.0_f32, 2.0_f32)`.
pub fn to_bnd_box_scalar<T: BvhScalar>(min: T, max: T) -> BoundingBox3 {
    bnd_box([min.to_double(), 0.0, 0.0], [max.to_double(), 0.0, 0.0])
}

fn bnd_box(min: [f64; 3], max: [f64; 3]) -> BoundingBox3 {
    BoundingBox3::new(
        Point3::new(min[0], min[1], min[2]),
        Point3::new(max[0], max[1], max[2]),
    )
}

macro_rules! bvh_vector {
    ($scalar:ty) => {
        impl BvhVector for Vec2<$scalar> {
            type Scalar = $scalar;
            const DIMENSION: usize = 2;
            const FILLED_MAX: Self = Self::from_value(<$scalar>::MAX);
            const FILLED_LOWEST: Self = Self::from_value(<$scalar>::MIN);
            fn splat(value: $scalar) -> Self {
                Self::from_value(value)
            }
            fn componentwise_min(&self, other: &Self) -> Self {
                self.cwise_min(other)
            }
            fn componentwise_max(&self, other: &Self) -> Self {
                self.cwise_max(other)
            }
            fn inner_product(&self, other: &Self) -> $scalar {
                self.dot(other)
            }
            fn component(&self, axis: i32) -> $scalar {
                if axis == 0 { self.x } else { self.y }
            }
            fn unit_x() -> Self {
                Self::new(1.0, 0.0)
            }
            fn unit_y() -> Self {
                Self::new(0.0, 1.0)
            }
            fn unit_z() -> Self {
                Self::new(0.0, 0.0)
            }
            fn to_bnd_box(min: &Self, max: &Self) -> BoundingBox3 {
                bnd_box(
                    [min.x.into(), min.y.into(), 0.0],
                    [max.x.into(), max.y.into(), 0.0],
                )
            }
        }
        impl BvhVector for Vec3<$scalar> {
            type Scalar = $scalar;
            const DIMENSION: usize = 3;
            const FILLED_MAX: Self = Self::from_value(<$scalar>::MAX);
            const FILLED_LOWEST: Self = Self::from_value(<$scalar>::MIN);
            fn splat(value: $scalar) -> Self {
                Self::from_value(value)
            }
            fn componentwise_min(&self, other: &Self) -> Self {
                self.cwise_min(other)
            }
            fn componentwise_max(&self, other: &Self) -> Self {
                self.cwise_max(other)
            }
            fn inner_product(&self, other: &Self) -> $scalar {
                self.dot(other)
            }
            fn component(&self, axis: i32) -> $scalar {
                match axis {
                    0 => self.x,
                    1 => self.y,
                    _ => self.z,
                }
            }
            fn unit_x() -> Self {
                Self::new(1.0, 0.0, 0.0)
            }
            fn unit_y() -> Self {
                Self::new(0.0, 1.0, 0.0)
            }
            fn unit_z() -> Self {
                Self::new(0.0, 0.0, 1.0)
            }
            fn to_bnd_box(min: &Self, max: &Self) -> BoundingBox3 {
                bnd_box(
                    [min.x.into(), min.y.into(), min.z.into()],
                    [max.x.into(), max.y.into(), max.z.into()],
                )
            }
        }
        impl BvhVector for Vec4<$scalar> {
            type Scalar = $scalar;
            const DIMENSION: usize = 4;
            const FILLED_MAX: Self = Self::from_value(<$scalar>::MAX);
            const FILLED_LOWEST: Self = Self::from_value(<$scalar>::MIN);
            fn splat(value: $scalar) -> Self {
                Self::from_value(value)
            }
            fn componentwise_min(&self, other: &Self) -> Self {
                self.cwise_min(other)
            }
            fn componentwise_max(&self, other: &Self) -> Self {
                self.cwise_max(other)
            }
            fn inner_product(&self, other: &Self) -> $scalar {
                self.dot(other)
            }
            fn component(&self, axis: i32) -> $scalar {
                match axis {
                    0 => self.x,
                    1 => self.y,
                    2 => self.z,
                    _ => self.w,
                }
            }
            fn unit_x() -> Self {
                Self::new(1.0, 0.0, 0.0, 0.0)
            }
            fn unit_y() -> Self {
                Self::new(0.0, 1.0, 0.0, 0.0)
            }
            fn unit_z() -> Self {
                Self::new(0.0, 0.0, 1.0, 0.0)
            }
            fn to_bnd_box(min: &Self, max: &Self) -> BoundingBox3 {
                bnd_box(
                    [min.x.into(), min.y.into(), min.z.into()],
                    [max.x.into(), max.y.into(), max.z.into()],
                )
            }
        }
    };
}
bvh_vector!(f32);
bvh_vector!(f64);

/// Compile-time dimension tag selecting a vector type; `Dimension::<3>`.
pub struct Dimension<const N: usize>;

/// `BVH::VectorType<T, N>`: maps a dimension to its NCollection vector type.
pub trait VectorType<T> {
    /// The selected type (`T` itself for N = 1).
    type Vector;
}
impl<T> VectorType<T> for Dimension<1> {
    type Vector = T;
}
impl<T> VectorType<T> for Dimension<2> {
    type Vector = Vec2<T>;
}
impl<T> VectorType<T> for Dimension<3> {
    type Vector = Vec3<T>;
}
impl<T> VectorType<T> for Dimension<4> {
    type Vector = Vec4<T>;
}

/// `BVH::VectorType<T, N>::Type`; `BvhVecNt<f64, 3>` is `Vec3<f64>`.
pub type BvhVecNt<T, const N: usize> = <Dimension<N> as VectorType<T>>::Vector;
/// `BVH::ArrayType<T, N>::Type`; `BvhArrayNt<f64, 3>` is `LinearVector<Vec3<f64>>`.
pub type BvhArrayNt<T, const N: usize> = LinearVector<BvhVecNt<T, N>>;

/// `BVH_Vec2i`: 2D vector of integers.
pub type BvhVec2i = Vec2<i32>;
/// `BVH_Vec3i`: 3D vector of integers.
pub type BvhVec3i = Vec3<i32>;
/// `BVH_Vec4i`: 4D vector of integers (tree node records, triangle indices).
pub type BvhVec4i = Vec4<i32>;
/// `BVH_Vec2f`: 2D vector of single precision reals.
pub type BvhVec2f = Vec2<f32>;
/// `BVH_Vec3f`: 3D vector of single precision reals.
pub type BvhVec3f = Vec3<f32>;
/// `BVH_Vec4f`: 4D vector of single precision reals.
pub type BvhVec4f = Vec4<f32>;
/// `BVH_Vec2d`: 2D vector of double precision reals.
pub type BvhVec2d = Vec2<f64>;
/// `BVH_Vec3d`: 3D vector of double precision reals.
pub type BvhVec3d = Vec3<f64>;
/// `BVH_Vec4d`: 4D vector of double precision reals.
pub type BvhVec4d = Vec4<f64>;
/// `BVH_Array2i`: array of 2D integer vectors.
pub type BvhArray2i = LinearVector<BvhVec2i>;
/// `BVH_Array3i`: array of 3D integer vectors.
pub type BvhArray3i = LinearVector<BvhVec3i>;
/// `BVH_Array4i`: array of 4D integer vectors.
pub type BvhArray4i = LinearVector<BvhVec4i>;
/// `BVH_Array2f`: array of 2D float vectors.
pub type BvhArray2f = LinearVector<BvhVec2f>;
/// `BVH_Array3f`: array of 3D float vectors.
pub type BvhArray3f = LinearVector<BvhVec3f>;
/// `BVH_Array4f`: array of 4D float vectors.
pub type BvhArray4f = LinearVector<BvhVec4f>;
/// `BVH_Array2d`: array of 2D double vectors.
pub type BvhArray2d = LinearVector<BvhVec2d>;
/// `BVH_Array3d`: array of 3D double vectors.
pub type BvhArray3d = LinearVector<BvhVec3d>;
/// `BVH_Array4d`: array of 4D double vectors.
pub type BvhArray4d = LinearVector<BvhVec4d>;
/// `BVH_Mat4f`: 4x4 matrix of single precision reals.
pub type BvhMat4f = Mat4<f32>;
/// `BVH_Mat4d`: 4x4 matrix of double precision reals.
pub type BvhMat4d = Mat4<f64>;
