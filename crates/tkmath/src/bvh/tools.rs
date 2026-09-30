// Rust translation of FoundationClasses/TKMath/BVH/BVH_Tools.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_Tools<T, N>`: static distance, projection and ray-box operations.
use super::bounding_box::BvhBox;
use super::ray::BvhRay;
use super::scalar::{BvhScalar, std_max, std_min};
use super::vector::BvhVector;
mod triangle;
pub use triangle::{
    PrjStateInTriangle, TriangleProjectionState, point_triangle_projection,
    point_triangle_square_distance,
};

/// Square distance between two boxes; zero when either is invalid.
/// Example: `box_box_square_distance(&first, &second)`.
pub fn box_box_square_distance<V: BvhVector>(first: &BvhBox<V>, second: &BvhBox<V>) -> V::Scalar {
    if !first.is_valid() || !second.is_valid() {
        return V::Scalar::ZERO;
    }
    box_box_square_distance_by_corners(
        first.corner_min(),
        first.corner_max(),
        second.corner_min(),
        second.corner_max(),
    )
}

/// Square distance between boxes given by corners (no validity check).
/// Example: `box_box_square_distance_by_corners(&min1, &max1, &min2, &max2)`.
pub fn box_box_square_distance_by_corners<V: BvhVector>(
    min1: &V,
    max1: &V,
    min2: &V,
    max2: &V,
) -> V::Scalar {
    let mut distance = V::Scalar::ZERO;
    for axis in 0..V::DIMENSION {
        if min1[axis] > max2[axis] {
            let gap = min1[axis] - max2[axis];
            distance += gap * gap;
        } else if max1[axis] < min2[axis] {
            let gap = min2[axis] - max1[axis];
            distance += gap * gap;
        }
    }
    distance
}

/// Square distance from a point to a box; zero for an invalid box.
/// Example: `point_box_square_distance(&point, &bounding)`.
pub fn point_box_square_distance<V: BvhVector>(point: &V, bounding: &BvhBox<V>) -> V::Scalar {
    if !bounding.is_valid() {
        return V::Scalar::ZERO;
    }
    point_box_square_distance_by_corners(point, bounding.corner_min(), bounding.corner_max())
}

/// Square distance from a point to a box given by corners.
/// Example: `point_box_square_distance_by_corners(&point, &min, &max)`.
pub fn point_box_square_distance_by_corners<V: BvhVector>(
    point: &V,
    min: &V,
    max: &V,
) -> V::Scalar {
    let mut distance = V::Scalar::ZERO;
    for axis in 0..V::DIMENSION {
        if point[axis] < min[axis] {
            let gap = min[axis] - point[axis];
            distance += gap * gap;
        } else if point[axis] > max[axis] {
            let gap = point[axis] - max[axis];
            distance += gap * gap;
        }
    }
    distance
}

/// Projects a point on a box; an invalid box returns the point unchanged.
/// Example: `point_box_projection(&point, &bounding)`.
pub fn point_box_projection<V: BvhVector>(point: &V, bounding: &BvhBox<V>) -> V {
    if !bounding.is_valid() {
        return *point;
    }
    point_box_projection_by_corners(point, bounding.corner_min(), bounding.corner_max())
}

/// `point.cwiseMax(min).cwiseMin(max)` with NCollection tie rules.
/// Example: `point_box_projection_by_corners(&point, &min, &max)`.
pub fn point_box_projection_by_corners<V: BvhVector>(point: &V, min: &V, max: &V) -> V {
    point.componentwise_max(min).componentwise_min(max)
}

/// Ray-box hit times `(enter, leave)` using the precomputed reciprocal direction.
///
/// Returns `None` for an invalid box, a miss, or a box behind the origin.
/// Example: `ray_box_intersection(&ray, &bounding)`.
pub fn ray_box_intersection<V: BvhVector>(
    ray: &BvhRay<V>,
    bounding: &BvhBox<V>,
) -> Option<(V::Scalar, V::Scalar)> {
    if !bounding.is_valid() {
        return None;
    }
    ray_box_intersection_by_corners(ray, bounding.corner_min(), bounding.corner_max())
}

/// Slab test with `1/Direct`; an infinite reciprocal (parallel axis) only checks
/// the origin against the slab, avoiding `0 * inf`.
/// Example: `ray_box_intersection_by_corners(&ray, &min, &max)`.
pub fn ray_box_intersection_by_corners<V: BvhVector>(
    ray: &BvhRay<V>,
    min: &V,
    max: &V,
) -> Option<(V::Scalar, V::Scalar)> {
    let mut enter = V::Scalar::LOWEST;
    let mut leave = V::Scalar::MAX;
    for axis in 0..V::DIMENSION {
        let origin = ray.origin[axis];
        if ray.inv_direct[axis].is_infinite_value() {
            if origin < min[axis] || origin > max[axis] {
                return None;
            }
            continue;
        }
        let first = (min[axis] - origin) * ray.inv_direct[axis];
        let second = (max[axis] - origin) * ray.inv_direct[axis];
        enter = std_max(enter, std_min(first, second));
        leave = std_min(leave, std_max(first, second));
        if enter > leave {
            return None;
        }
    }
    accepted_hit(enter, leave)
}

/// Ray-box hit times from origin and direction, dividing per axis.
/// Example: `ray_box_intersection_by_direction(&origin, &direction, &bounding)`.
pub fn ray_box_intersection_by_direction<V: BvhVector>(
    origin: &V,
    direction: &V,
    bounding: &BvhBox<V>,
) -> Option<(V::Scalar, V::Scalar)> {
    if !bounding.is_valid() {
        return None;
    }
    ray_box_intersection_by_direction_corners(
        origin,
        direction,
        bounding.corner_min(),
        bounding.corner_max(),
    )
}

/// Single-pass slab test with early exit; a zero direction component only checks
/// the origin against that slab.
/// Example: `ray_box_intersection_by_direction_corners(&origin, &direction, &min, &max)`.
pub fn ray_box_intersection_by_direction_corners<V: BvhVector>(
    origin: &V,
    direction: &V,
    min: &V,
    max: &V,
) -> Option<(V::Scalar, V::Scalar)> {
    let mut enter = V::Scalar::LOWEST;
    let mut leave = V::Scalar::MAX;
    for axis in 0..V::DIMENSION {
        if direction[axis] == V::Scalar::ZERO {
            if origin[axis] < min[axis] || origin[axis] > max[axis] {
                return None;
            }
            continue;
        }
        let first = (min[axis] - origin[axis]) / direction[axis];
        let second = (max[axis] - origin[axis]) / direction[axis];
        enter = std_max(enter, std_min(first, second));
        leave = std_min(leave, std_max(first, second));
        if enter > leave {
            return None;
        }
    }
    accepted_hit(enter, leave)
}

/// Rejects intervals behind the origin (`leave < 0`).
fn accepted_hit<T: BvhScalar>(enter: T, leave: T) -> Option<(T, T)> {
    if leave < T::ZERO {
        return None;
    }
    Some((enter, leave))
}
