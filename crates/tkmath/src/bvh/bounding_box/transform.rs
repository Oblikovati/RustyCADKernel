// Rust translation of FoundationClasses/TKMath/BVH/BVH_Box.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_BaseBox<T, 3, BVH_Box>`: affine transformation, defined only for N = 3.
use super::BvhBox;
use crate::bvh::scalar::{BvhScalar, std_max, std_min};
use crate::bvh::vector::BvhVector;
use rustycad_tkernel::ncollection::{Mat4, MatrixScalar, Vec3};

impl<T> BvhBox<Vec3<T>>
where
    T: BvhScalar + MatrixScalar,
    Vec3<T>: BvhVector<Scalar = T>,
{
    /// Transforms this box in place; identity matrices and invalid boxes are
    /// left untouched. Example: `bounding.transform(&matrix)`.
    pub fn transform(&mut self, matrix: &Mat4<T>) {
        if matrix.is_identity() || !self.is_valid() {
            return;
        }
        *self = self.transformed(matrix);
    }

    /// Returns the box of the transformed box with James Arvo's algorithm:
    /// start at the translation, then add per-element min/max products.
    /// Example: `let moved = bounding.transformed(&matrix);`.
    pub fn transformed(&self, matrix: &Mat4<T>) -> Self {
        if matrix.is_identity() || !self.is_valid() {
            return *self;
        }
        let translation = matrix.get_column(3).xyz();
        let mut new_min = translation;
        let mut new_max = translation;
        for column in 0..3 {
            for row in 0..3 {
                let element = matrix.get_value(row, column);
                let first = element * self.min_point[column];
                let second = element * self.max_point[column];
                new_min[row] += std_min(first, second);
                new_max[row] += std_max(first, second);
            }
        }
        Self::from_corners(new_min, new_max)
    }
}
