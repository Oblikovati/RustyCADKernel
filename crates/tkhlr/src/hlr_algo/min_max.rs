// Rust translation of ModelingAlgorithms/TKHLR/HLRAlgo/HLRAlgo.hxx/.cxx and HLRAlgo_EdgesBlock::MinMaxIndices at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The 16-direction bounding boxes of the hidden line algorithm and their integer
//! encoding (the package functions of HLRAlgo and `HLRAlgo_EdgesBlock::MinMaxIndices`).

/// `cos(k*PI/14)` and `sin(k*PI/14)` for k = 0..6, the file statics of HLRAlgo.cxx, as
/// the pinned build computes them (bit patterns, so that no platform `cos` can differ).
const DIRECTIONS: [(f64, f64); 7] = [
    (
        f64::from_bits(0x3ff0000000000000),
        f64::from_bits(0x0000000000000000),
    ),
    (
        f64::from_bits(0x3fef329c0558e969),
        f64::from_bits(0x3fcc7b90e3024582),
    ),
    (
        f64::from_bits(0x3fecd4bca9cb5c71),
        f64::from_bits(0x3fdbc4c04d71abc1),
    ),
    (
        f64::from_bits(0x3fe904c37505de4b),
        f64::from_bits(0x3fe3f3a0e28bedd1),
    ),
    (
        f64::from_bits(0x3fe3f3a0e28bedd2),
        f64::from_bits(0x3fe904c37505de4b),
    ),
    (
        f64::from_bits(0x3fdbc4c04d71abc2),
        f64::from_bits(0x3fecd4bca9cb5c71),
    ),
    (
        f64::from_bits(0x3fcc7b90e3024584),
        f64::from_bits(0x3fef329c0558e969),
    ),
];

/// HLRAlgo_EdgesBlock::MinMaxIndices: eight minimum and eight maximum integers, the
/// encoded or decoded box of an edge, a wire or a face. Natively the arrays are left
/// uninitialized; the Rust default is zero.
///
/// ```
/// use rustycad_tkhlr::hlr_algo::MinMaxIndices;
/// let mut a = MinMaxIndices { min: [5; 8], max: [9; 8] };
/// a.minimize(&MinMaxIndices { min: [3; 8], max: [7; 8] });
/// assert_eq!((a.min[0], a.max[0]), (3, 7));
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MinMaxIndices {
    /// The minimum indices (`Min[8]`).
    pub min: [i32; 8],
    /// The maximum indices (`Max[8]`).
    pub max: [i32; 8],
}

impl MinMaxIndices {
    /// Minimize: lowers both arrays to the smaller values of `other`, element by
    /// element (the maximum array is minimized too, as natively);
    /// `a.minimize(&b)`.
    pub fn minimize(&mut self, other: &MinMaxIndices) -> &mut Self {
        for i in 0..8 {
            if self.min[i] > other.min[i] {
                self.min[i] = other.min[i];
            }
            if self.max[i] > other.max[i] {
                self.max[i] = other.max[i];
            }
        }
        self
    }

    /// Maximize: raises both arrays to the larger values of `other`, element by
    /// element (the minimum array is maximized too, as natively);
    /// `a.maximize(&b)`.
    pub fn maximize(&mut self, other: &MinMaxIndices) -> &mut Self {
        for i in 0..8 {
            if self.min[i] < other.min[i] {
                self.min[i] = other.min[i];
            }
            if self.max[i] < other.max[i] {
                self.max[i] = other.max[i];
            }
        }
        self
    }
}

/// HLRAlgo::UpdateMinMax: widens the 16-direction box `(min, max)` by the point
/// `(x, y, z)`: its projections on 7 directions of the plane and their normals, then
/// `z` twice; `update_min_max(1.0, 2.0, 3.0, &mut min, &mut max)`.
pub fn update_min_max(x: f64, y: f64, z: f64, min: &mut [f64; 16], max: &mut [f64; 16]) {
    let mut d = [0.0_f64; 16];
    for (k, (cosu, sinu)) in DIRECTIONS.iter().enumerate() {
        d[2 * k] = cosu * x + sinu * y;
        d[2 * k + 1] = sinu * x - cosu * y;
    }
    d[14] = z;
    d[15] = z;
    for i in 0..16 {
        if min[i] > d[i] {
            min[i] = d[i];
        }
        if max[i] < d[i] {
            max[i] = d[i];
        }
    }
}

/// HLRAlgo::EnlargeMinMax: widens every direction of the box by `tol`;
/// `enlarge_min_max(1e-3, &mut min, &mut max)`.
pub fn enlarge_min_max(tol: f64, min: &mut [f64; 16], max: &mut [f64; 16]) {
    for i in 0..16 {
        min[i] -= tol;
        max[i] += tol;
    }
}

/// HLRAlgo::InitMinMax: the empty box, `big` below and `-big` above;
/// `init_min_max(1e100, &mut min, &mut max)`.
pub fn init_min_max(big: f64, min: &mut [f64; 16], max: &mut [f64; 16]) {
    min.fill(big);
    max.fill(-big);
}

/// The 15-bit field `value & 0x7fff` placed in the high half-word (`<< 16`).
fn high_field(value: i32) -> i32 {
    (value & 0x0000_7fff) << 16
}

/// HLRAlgo::EncodeMinMax: packs the 16 minimum and 16 maximum indices of `min` and
/// `max` (15 bits each) two per integer into `min_max`;
/// `encode_min_max(&min, &max, &mut packed)`.
pub fn encode_min_max(min: &MinMaxIndices, max: &MinMaxIndices, min_max: &mut MinMaxIndices) {
    for k in 0..4 {
        min_max.min[k] = (min.min[2 * k + 1] & 0x0000_7fff) + high_field(min.min[2 * k]);
        min_max.max[k] = (max.min[2 * k + 1] & 0x0000_7fff) + high_field(max.min[2 * k]);
        min_max.min[k + 4] = (min.max[2 * k + 1] & 0x0000_7fff) + high_field(min.max[2 * k]);
        min_max.max[k + 4] = (max.max[2 * k + 1] & 0x0000_7fff) + high_field(max.max[2 * k]);
    }
}

/// HLRAlgo::SizeBox: the product of the 14 index extents `max - min` (the 8 minimum
/// fields and the first 6 maximum fields), each an `int` difference converted to
/// double; `size_box(&min, &max)`.
pub fn size_box(min: &MinMaxIndices, max: &MinMaxIndices) -> f64 {
    // The native `int` subtraction wraps on overflow on the pinned profile.
    let extent = |high: i32, low: i32| f64::from(high.wrapping_sub(low));
    let mut s = extent(max.min[0], min.min[0]);
    for i in 1..8 {
        s *= extent(max.min[i], min.min[i]);
    }
    for i in 0..6 {
        s *= extent(max.max[i], min.max[i]);
    }
    s
}

/// HLRAlgo::DecodeMinMax: unpacks `min_max` (see [`encode_min_max`]) into the 16
/// minimum and 16 maximum indices of `min` and `max`;
/// `decode_min_max(&packed, &mut min, &mut max)`.
pub fn decode_min_max(min_max: &MinMaxIndices, min: &mut MinMaxIndices, max: &mut MinMaxIndices) {
    let high = |value: i32| (value & 0x7fff_0000) >> 16;
    let low = |value: i32| value & 0x0000_7fff;
    for k in 0..4 {
        min.min[2 * k] = high(min_max.min[k]);
        max.min[2 * k] = high(min_max.max[k]);
        min.min[2 * k + 1] = low(min_max.min[k]);
        max.min[2 * k + 1] = low(min_max.max[k]);
        min.max[2 * k] = high(min_max.min[k + 4]);
        max.max[2 * k] = high(min_max.max[k + 4]);
        min.max[2 * k + 1] = low(min_max.min[k + 4]);
        max.max[2 * k + 1] = low(min_max.max[k + 4]);
    }
}

/// HLRAlgo::CopyMinMax: copies `i_min` into `o_min` and `i_max` into `o_max`;
/// `copy_min_max(&a, &b, &mut c, &mut d)`.
pub fn copy_min_max(
    i_min: &MinMaxIndices,
    i_max: &MinMaxIndices,
    o_min: &mut MinMaxIndices,
    o_max: &mut MinMaxIndices,
) {
    *o_min = *i_min;
    *o_max = *i_max;
}

/// HLRAlgo::AddMinMax: `o_min.minimize(i_min)` and `o_max.maximize(i_max)`;
/// `add_min_max(&a, &b, &mut c, &mut d)`.
pub fn add_min_max(
    i_min: &MinMaxIndices,
    i_max: &MinMaxIndices,
    o_min: &mut MinMaxIndices,
    o_max: &mut MinMaxIndices,
) {
    o_min.minimize(i_min);
    o_max.maximize(i_max);
}
