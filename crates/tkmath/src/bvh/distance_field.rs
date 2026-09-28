// Created on: 2014-09-06
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
// Rust translation of FoundationClasses/TKMath/BVH/BVH_DistanceField.hxx and
// BVH_DistanceField.lxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_DistanceField<T, N>`: a voxel grid of (signed) distances to a geometry
//! of BVH triangulations, for N = 3 and 4.
use super::geometry::BvhGeometry;
use super::scalar::{BvhScalar, std_max, std_min};
use super::set::BvhSet;
use super::vector::BvhVector;
use rustycad_tkernel::ncollection::{Vec3, Vec4};
use rustycad_tkernel::standard::Failure;
mod nearest_point;
mod square_distance;
pub use nearest_point::{direction_to_nearest_point, distance_to_box};
pub use square_distance::{
    BvhPointGeometrySquareDistance, BvhPointTriangulationSquareDistance, BvhSquareDistanceToPoint,
    square_distance_to_geometry, square_distance_to_object,
};

/// The `Standard_STATIC_ASSERT(N == 3 || N == 4)` of the distance field: the
/// vector types whose X, Y and Z span the voxel grid.
pub trait BvhFieldVector: BvhVector {}
impl BvhFieldVector for Vec3<f32> {}
impl BvhFieldVector for Vec3<f64> {}
impl BvhFieldVector for Vec4<f32> {}
impl BvhFieldVector for Vec4<f64> {}

/// Empty grid voxels between the geometry box and the grid border, per side.
const VOXEL_OFFSET: i32 = 2;
/// Smallest grid dimension `Build` chooses (still capped by the maximum size).
const MIN_DIMENSION: i32 = 16;

/// Distance field: the distance from each voxel center to the nearest
/// triangle, negative inside when signing is enabled.
///
/// The voxel buffer holds `maximum_size^3` values; upstream leaves them
/// uninitialized, the port starts them at zero. A parallel field still builds
/// its Z slices in order (see `build`).
/// Example: `let mut field = BvhDistanceField::<BvhVec3d>::new(32, true)?; field.build(&mut geometry)?;`.
#[derive(Clone, Debug)]
pub struct BvhDistanceField<V: BvhFieldVector> {
    voxel_data: Vec<V::Scalar>,
    voxel_size: V,
    corner_min: V,
    corner_max: V,
    dimension_x: i32,
    dimension_y: i32,
    dimension_z: i32,
    maximum_size: i32,
    compute_sign: bool,
    is_parallel: bool,
}

impl<V: BvhFieldVector> BvhDistanceField<V> {
    /// Creates an empty field of at most `maximum_size` voxels per side.
    ///
    /// A negative voxel count makes `new T[]` throw upstream; it fails here as
    /// `std::bad_array_new_length`, and so does a count past `int` range.
    /// Example: `BvhDistanceField::<BvhVec3d>::new(16, true)?`.
    pub fn new(maximum_size: i32, compute_sign: bool) -> Result<Self, Failure> {
        let count = i64::from(maximum_size).pow(3);
        let count = usize::try_from(count)
            .ok()
            .filter(|count| *count <= i32::MAX as usize)
            .ok_or_else(|| {
                Failure::with_type(
                    "std::bad_array_new_length",
                    &format!("BVH_DistanceField voxel count {count}, expected 0..=2147483647"),
                    "",
                )
            })?;
        Ok(Self {
            voxel_data: vec![V::Scalar::ZERO; count],
            voxel_size: V::default(),
            corner_min: V::default(),
            corner_max: V::default(),
            dimension_x: 0,
            dimension_y: 0,
            dimension_z: 0,
            maximum_size,
            compute_sign,
            is_parallel: false,
        })
    }
    /// Returns parallel flag; `field.is_parallel()`.
    pub fn is_parallel(&self) -> bool {
        self.is_parallel
    }
    /// Set parallel flag controlling possibility of parallel execution.
    /// Example: `field.set_parallel(true)`.
    pub fn set_parallel(&mut self, is_parallel: bool) {
        self.is_parallel = is_parallel;
    }
    /// Returns packed voxel data (all `maximum_size^3` values); `field.packed_data()`.
    pub fn packed_data(&self) -> &[V::Scalar] {
        &self.voxel_data
    }
    /// Returns distance value for the given voxel; `field.voxel(0, 0, 0)?`.
    pub fn voxel(&self, x: i32, y: i32, z: i32) -> Result<V::Scalar, Failure> {
        let index = self.voxel_index(x, y, z)?;
        Ok(self.voxel_data[index])
    }
    /// Mutable distance value for the given voxel; `*field.voxel_mut(0, 0, 0)? = 1.0`.
    pub fn voxel_mut(&mut self, x: i32, y: i32, z: i32) -> Result<&mut V::Scalar, Failure> {
        let index = self.voxel_index(x, y, z)?;
        Ok(&mut self.voxel_data[index])
    }
    /// `theX + (theY + theZ * myDimensionY) * myDimensionX`; upstream does not
    /// check it, the port rejects offsets outside the buffer.
    fn voxel_index(&self, x: i32, y: i32, z: i32) -> Result<usize, Failure> {
        let index = i64::from(x)
            + (i64::from(y) + i64::from(z) * i64::from(self.dimension_y))
                * i64::from(self.dimension_x);
        usize::try_from(index)
            .ok()
            .filter(|index| *index < self.voxel_data.len())
            .ok_or_else(|| {
                Failure::out_of_range(&format!(
                    "BVH_DistanceField::Voxel({x}, {y}, {z}): offset {index}, expected 0..{}",
                    self.voxel_data.len()
                ))
            })
    }
    /// Returns size of voxel grid in X dimension; `field.dimension_x()`.
    pub fn dimension_x(&self) -> i32 {
        self.dimension_x
    }
    /// Returns size of voxel grid in Y dimension; `field.dimension_y()`.
    pub fn dimension_y(&self) -> i32 {
        self.dimension_y
    }
    /// Returns size of voxel grid in Z dimension; `field.dimension_z()`.
    pub fn dimension_z(&self) -> i32 {
        self.dimension_z
    }
    /// Returns size of single voxel; `field.voxel_size().x`.
    pub fn voxel_size(&self) -> &V {
        &self.voxel_size
    }
    /// Returns minimum corner of voxel grid; `field.corner_min().x`.
    pub fn corner_min(&self) -> &V {
        &self.corner_min
    }
    /// Returns maximum corner of voxel grid; `field.corner_max().x`.
    pub fn corner_max(&self) -> &V {
        &self.corner_max
    }
}

impl<V: BvhFieldVector> BvhDistanceField<V> {
    /// Builds the field from a geometry of triangulations; false for an empty
    /// geometry.
    ///
    /// Each dimension is `maximum_size * side / longest side`, truncated, then
    /// raised to 16 and capped by the maximum size. The grid adds two voxels
    /// of margin per side. Upstream builds the Z slices with
    /// `OSD_Parallel::For` when parallel; the slices only read the geometry,
    /// except for its lazy tree builds, which race upstream. The port builds
    /// them in order, which gives the same voxels.
    /// Example: `field.build(&mut geometry)?`.
    pub fn build(&mut self, geometry: &mut BvhGeometry<V>) -> Result<bool, Failure> {
        if geometry.size() == 0 {
            return Ok(false);
        }
        let size = geometry.cached_box()?.size();
        let longest = std_max(std_max(size[0], size[1]), size[2]);
        self.dimension_x = self.grid_dimension(size[0], longest);
        self.dimension_y = self.grid_dimension(size[1], longest);
        self.dimension_z = self.grid_dimension(size[2], longest);
        let bounding = geometry.cached_box()?;
        self.place_grid(bounding.corner_min(), bounding.corner_max(), &size);
        for slice in 0..self.dimension_z {
            self.build_slices(geometry, slice, slice + 1)?;
        }
        Ok(true)
    }

    /// `static_cast<int>(myMaximumSize * side / longest)` clamped to
    /// `[16, maximum_size]` with `std::max` then `std::min`.
    fn grid_dimension(&self, side: V::Scalar, longest: V::Scalar) -> i32 {
        let dimension = (V::Scalar::from_int(self.maximum_size) * side / longest).truncate_to_int();
        std_min(self.maximum_size, std_max(dimension, MIN_DIMENSION))
    }

    /// Grid corners with the voxel margin, and the voxel size.
    fn place_grid(&mut self, box_min: &V, box_max: &V, size: &V) {
        let dimensions = [self.dimension_x, self.dimension_y, self.dimension_z];
        for (axis, dimension) in dimensions.iter().enumerate() {
            let inner = V::Scalar::from_int(dimension - 2 * VOXEL_OFFSET);
            let margin = V::Scalar::from_int(VOXEL_OFFSET) * size[axis] / inner;
            self.corner_min[axis] = box_min[axis] - margin;
            self.corner_max[axis] = box_max[axis] + margin;
        }
        for (axis, dimension) in dimensions.iter().enumerate() {
            let span = self.corner_max[axis] - self.corner_min[axis];
            self.voxel_size[axis] = span / V::Scalar::from_int(*dimension);
        }
    }

    /// `BuildSlices`: the distances of every voxel center of slices
    /// `[start_slice, final_slice)`.
    ///
    /// `sqrt` of a `float` is evaluated through `double` and rounded back,
    /// which equals the correctly rounded single-precision root.
    /// Example: `field.build_slices(&mut geometry, 0, 1)?`.
    pub fn build_slices(
        &mut self,
        geometry: &mut BvhGeometry<V>,
        start_slice: i32,
        final_slice: i32,
    ) -> Result<(), Failure> {
        for z in start_slice..final_slice {
            for y in 0..self.dimension_y {
                for x in 0..self.dimension_x {
                    let value = self.voxel_distance(geometry, [x, y, z])?;
                    *self.voxel_mut(x, y, z)? = value;
                }
            }
        }
        Ok(())
    }

    /// The (signed) distance at the center of one voxel.
    fn voxel_distance(
        &self,
        geometry: &mut BvhGeometry<V>,
        voxel: [i32; 3],
    ) -> Result<V::Scalar, Failure> {
        let half = V::Scalar::from_double(0.5);
        let mut center = V::default();
        for axis in 0..3 {
            let offset = V::Scalar::from_int(voxel[axis]) + half;
            center[axis] = self.corner_min[axis] + self.voxel_size[axis] * offset;
        }
        let mut is_outside = true;
        let square = square_distance_to_geometry(geometry, &center, &mut is_outside)?;
        let distance = V::Scalar::from_double(square.to_double().sqrt());
        if !self.compute_sign || is_outside {
            return Ok(distance);
        }
        Ok(-distance)
    }
}
