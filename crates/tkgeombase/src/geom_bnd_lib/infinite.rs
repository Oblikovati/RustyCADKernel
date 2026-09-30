// Rust translation of GeomBndLib_InfiniteHelpers.pxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomBndLib_InfiniteHelpers`: open a bounding box along the direction of an infinite
//! parameter. BndLib.cxx keeps private copies of the same three functions with the same
//! bodies; both packages call these.
use rustycad_tkernel::precision;
use rustycad_tkmath::bnd::{BoundingBox2, BoundingBox3};
use rustycad_tkmath::gp::{self, Direction2, Direction3};

/// A direction whose infinite parameter opens sides of its box type (the 3D and 2D
/// template specializations); `Direction3::new(1.0, 0.0, 0.0)?.open_min(&mut bounds)`.
pub trait InfiniteDirection: Copy {
    /// `Bnd_Box` for `gp_Dir`, `Bnd_Box2d` for `gp_Dir2d`.
    type Bounds;
    /// `OpenMin`: open the sides reached as the parameter decreases to -infinity.
    fn open_min(self, bounds: &mut Self::Bounds);
    /// `OpenMax`: open the sides reached as the parameter increases to +infinity.
    fn open_max(self, bounds: &mut Self::Bounds);
    /// `OpenMinMax`: open both sides along the direction.
    fn open_min_max(self, bounds: &mut Self::Bounds);
}

/// `OpenMin(theDir, theBox)`; `open_min(direction, &mut bounds)`.
pub fn open_min<D: InfiniteDirection>(direction: D, bounds: &mut D::Bounds) {
    direction.open_min(bounds);
}

/// `OpenMax(theDir, theBox)`; `open_max(direction, &mut bounds)`.
pub fn open_max<D: InfiniteDirection>(direction: D, bounds: &mut D::Bounds) {
    direction.open_max(bounds);
}

/// `OpenMinMax(theDir, theBox)`; `open_min_max(direction, &mut bounds)`.
pub fn open_min_max<D: InfiniteDirection>(direction: D, bounds: &mut D::Bounds) {
    direction.open_min_max(bounds);
}

// The first global axis (X, then Y, then Z) the direction is parallel to within
// Precision::Angular(), with the sign of its component; the native if/else-if order.
fn aligned_axis3(direction: Direction3) -> Option<(usize, bool)> {
    let axes: [Direction3; 3] = [gp::dx(), gp::dy(), gp::dz()];
    let coordinates: [f64; 3] = [direction.x(), direction.y(), direction.z()];
    (0..3)
        .find(|&axis| direction.is_parallel(axes[axis], precision::angular()))
        .map(|axis| (axis, coordinates[axis] > 0.0))
}

fn aligned_axis2(direction: Direction2) -> Option<(usize, bool)> {
    let axes: [Direction2; 2] = [gp::dx2d(), gp::dy2d()];
    let coordinates: [f64; 2] = [direction.x(), direction.y()];
    (0..2)
        .find(|&axis| direction.is_parallel(axes[axis], precision::angular()))
        .map(|axis| (axis, coordinates[axis] > 0.0))
}

fn open_side3(bounds: &mut BoundingBox3, axis: usize, upper: bool) {
    match (axis, upper) {
        (0, false) => bounds.open_xmin(),
        (0, true) => bounds.open_xmax(),
        (1, false) => bounds.open_ymin(),
        (1, true) => bounds.open_ymax(),
        (_, false) => bounds.open_zmin(),
        (_, true) => bounds.open_zmax(),
    }
}

fn open_side2(bounds: &mut BoundingBox2, axis: usize, upper: bool) {
    match (axis, upper) {
        (0, false) => bounds.open_xmin(),
        (0, true) => bounds.open_xmax(),
        (_, false) => bounds.open_ymin(),
        (_, true) => bounds.open_ymax(),
    }
}

impl InfiniteDirection for Direction3 {
    type Bounds = BoundingBox3;
    fn open_min(self, bounds: &mut BoundingBox3) {
        // A positive component runs to -infinity (min side), a negative one to +infinity.
        match aligned_axis3(self) {
            Some((axis, positive)) => open_side3(bounds, axis, !positive),
            None => (0..3).for_each(|axis| open_side3(bounds, axis, false)),
        }
    }
    fn open_max(self, bounds: &mut BoundingBox3) {
        match aligned_axis3(self) {
            Some((axis, positive)) => open_side3(bounds, axis, positive),
            None => (0..3).for_each(|axis| open_side3(bounds, axis, true)),
        }
    }
    fn open_min_max(self, bounds: &mut BoundingBox3) {
        match aligned_axis3(self) {
            Some((axis, _)) => [true, false].map(|upper| open_side3(bounds, axis, upper)),
            None => [false, true].map(|upper| (0..3).for_each(|a| open_side3(bounds, a, upper))),
        };
    }
}

impl InfiniteDirection for Direction2 {
    type Bounds = BoundingBox2;
    fn open_min(self, bounds: &mut BoundingBox2) {
        match aligned_axis2(self) {
            Some((axis, positive)) => open_side2(bounds, axis, !positive),
            None => (0..2).for_each(|axis| open_side2(bounds, axis, false)),
        }
    }
    fn open_max(self, bounds: &mut BoundingBox2) {
        match aligned_axis2(self) {
            Some((axis, positive)) => open_side2(bounds, axis, positive),
            None => (0..2).for_each(|axis| open_side2(bounds, axis, true)),
        }
    }
    fn open_min_max(self, bounds: &mut BoundingBox2) {
        match aligned_axis2(self) {
            Some((axis, _)) => [true, false].map(|upper| open_side2(bounds, axis, upper)),
            None => [false, true].map(|upper| (0..2).for_each(|a| open_side2(bounds, a, upper))),
        };
    }
}
