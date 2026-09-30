// Rust translation of the two CommonBox members of
// ModelingAlgorithms/TKGeomAlgo/IntPolyh/IntPolyh_MaillageAffinage.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The common box of the two point boxes, enlarged by 10% of its size, and the out codes
//! of every point against it.
use super::IntPolyhMaillageAffinage;
use crate::int_polyh::array::IntPolyhArrayOfPoints;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bnd::BoundingBox3;

impl IntPolyhMaillageAffinage {
    /// `CommonBox()`: `CommonBox(GetBox(1), GetBox(2), ...)`. `m.common_box()?`.
    pub fn common_box(&mut self) -> Result<(), Failure> {
        let (b1, b2) = (self.get_box(1), self.get_box(2));
        self.common_box_of(&b1, &b2)?;
        Ok(())
    }

    /// `CommonBox(B1, B2, XMin, YMin, ZMin, XMax, YMax, ZMax)`: the intersection of the
    /// two point boxes (all zero when they are disjoint), each side widened by 10% of its
    /// size (of another size when it is flat), and the out codes of every point (1, 2 on
    /// X, 4, 8 on Y, 16, 32 on Z). The box arguments are not read: the member boxes are.
    /// Returns `[XMin, YMin, ZMin, XMax, YMax, ZMax]`. `m.common_box_of(&b1, &b2)?`.
    pub fn common_box_of(
        &mut self,
        _b1: &BoundingBox3,
        _b2: &BoundingBox3,
    ) -> Result<[f64; 6], Failure> {
        let [x10, y10, z10, x11, y11, z11] = self.sides[0].bbox.get()?;
        let [x20, y20, z20, x21, y21, z21] = self.sides[1].bbox.get()?;
        let (mut min, mut max) = ([0.0_f64; 3], [0.0_f64; 3]);
        let disjoint: bool =
            x10 > x21 || x20 > x11 || y10 > y21 || y20 > y11 || z10 > z21 || z20 > z11;
        if !disjoint {
            for (axis, (lo1, hi1, lo2, hi2)) in [
                (x10, x11, x20, x21),
                (y10, y11, y20, y21),
                (z10, z11, z20, z21),
            ]
            .into_iter()
            .enumerate()
            {
                (min[axis], max[axis]) = overlap(lo1, hi1, lo2, hi2, (min[axis], max[axis]));
            }
        }
        let size: [f64; 3] = widening([max[0] - min[0], max[1] - min[1], max[2] - min[2]]);
        for axis in 0..3 {
            min[axis] -= size[axis];
            max[axis] += size[axis];
        }
        mark_points(&mut self.sides[0].points, min, max);
        mark_points(&mut self.sides[1].points, min, max);
        Ok([min[0], min[1], min[2], max[0], max[1], max[2]])
    }
}

/// The overlap `[max(lo1, lo2), min(hi1, hi2)]` of one axis, written as the native
/// comparisons (a NaN bound keeps the previous value).
fn overlap(lo1: f64, hi1: f64, lo2: f64, hi2: f64, (mut min, mut max): (f64, f64)) -> (f64, f64) {
    if hi1 <= hi2 {
        max = hi1;
    } else if hi2 <= hi1 {
        max = hi2;
    }
    if lo2 <= lo1 {
        min = lo1;
    } else if lo1 <= lo2 {
        min = lo2;
    }
    (min, max)
}

/// The widening of each axis: 10% of its size, or of the next non-null size when it is
/// null (X from Y then Z, Y from the new X then Z, Z from the new X then the new Y).
fn widening([mut x, mut y, mut z]: [f64; 3]) -> [f64; 3] {
    if x == 0.0 && y != 0.0 {
        x = y * 0.1;
    } else if x == 0.0 && z != 0.0 {
        x = z * 0.1;
    } else {
        x *= 0.1;
    }
    if y == 0.0 && x != 0.0 {
        y = x * 0.1;
    } else if y == 0.0 && z != 0.0 {
        y = z * 0.1;
    } else {
        y *= 0.1;
    }
    if z == 0.0 && x != 0.0 {
        z = x * 0.1;
    } else if z == 0.0 && y != 0.0 {
        z = y * 0.1;
    } else {
        z *= 0.1;
    }
    [x, y, z]
}

/// The out codes of every stored point against the box `[min, max]`.
fn mark_points(points: &mut IntPolyhArrayOfPoints, min: [f64; 3], max: [f64; 3]) {
    for i in 0..points.nb_items() {
        let pt = &mut points[i];
        let coords: [f64; 3] = [pt.x(), pt.y(), pt.z()];
        let mut r: i32 = 0;
        for (axis, c) in coords.into_iter().enumerate() {
            let (below, above): (i32, i32) = (1 << (2 * axis), 2 << (2 * axis));
            if c < min[axis] {
                r |= below;
            } else if c > max[axis] {
                r |= above;
            }
        }
        pt.set_part_of_common(r);
    }
}
