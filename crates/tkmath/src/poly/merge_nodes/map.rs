// Copyright (c) 2015-2021 OPEN CASCADE SAS
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
// Rust translation of Poly_MergeNodesTool::MergedNodesMap in
// FoundationClasses/TKMath/Poly/Poly_MergeNodesTool.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The node map of `Poly_MergeNodesTool`.
//!
//! With a merge tolerance several stored nodes can match a new one, and the first match
//! found wins, so the result depends on the bucket layout. The map therefore keeps the
//! native `NCollection_BaseMap` policy: the bucket count, the growth steps, head insertion
//! and the chain reversal of a resize, and the native DJB2 and cell hashes.
use rustycad_tkernel::ncollection::{Vec3, next_map_prime};
#[derive(Clone, Copy)]
struct MapNode {
    position: Vec3<f32>,
    normal: Vec3<f32>,
    index: i32,
}
/// Cell offsets of the 26 neighbours, in the native search order.
const NEIGHBOURS: [[i64; 3]; 26] = [
    [-1, 0, 0],
    [1, 0, 0],
    [0, -1, 0],
    [0, 1, 0],
    [0, 0, -1],
    [0, 0, 1],
    [-1, -1, 0],
    [1, -1, 0],
    [1, 1, 0],
    [-1, 1, 0],
    [0, -1, -1],
    [0, 1, -1],
    [0, 1, 1],
    [0, -1, 1],
    [-1, 0, -1],
    [1, 0, -1],
    [1, 0, 1],
    [-1, 0, 1],
    [-1, -1, -1],
    [1, -1, -1],
    [-1, 1, -1],
    [1, 1, -1],
    [-1, -1, 1],
    [1, -1, 1],
    [-1, 1, 1],
    [1, 1, 1],
];
/// Outcome of [`MergedNodesMap::bind`]: the node index and whether it is new. The native
/// `theIsOpposite` output is set but never read by the tool, so it is not reported.
pub(super) struct Binding {
    pub(super) index: i32,
    pub(super) is_new: bool,
}
pub(super) struct MergedNodesMap {
    buckets: Vec<Vec<MapNode>>,
    bucket_count: usize,
    size: usize,
    tolerance: f32,
    inverse_tolerance: f32,
    angle: f32,
    angle_cos: f32,
    merge_opposite: bool,
}
impl MergedNodesMap {
    pub(super) fn new(bucket_count: usize) -> Self {
        Self {
            buckets: Vec::new(),
            bucket_count,
            size: 0,
            tolerance: 0.0,
            inverse_tolerance: 0.0,
            angle: 1.0,
            angle_cos: 0.0,
            merge_opposite: false,
        }
    }
    pub(super) fn merge_angle(&self) -> f64 {
        f64::from(self.angle)
    }
    pub(super) fn set_merge_angle(&mut self, angle: f64) {
        self.angle = angle as f32;
        self.angle_cos = angle.cos() as f32;
    }
    pub(super) fn has_merge_angle(&self) -> bool {
        self.angle > 0.0
    }
    pub(super) fn to_merge_any_angle(&self) -> bool {
        self.angle_cos <= 0.01
    }
    pub(super) const fn to_merge_opposite(&self) -> bool {
        self.merge_opposite
    }
    pub(super) const fn set_merge_opposite(&mut self, merge: bool) {
        self.merge_opposite = merge;
    }
    pub(super) fn merge_tolerance(&self) -> f64 {
        f64::from(self.tolerance)
    }
    pub(super) fn set_merge_tolerance(&mut self, tolerance: f64) {
        self.tolerance = tolerance as f32;
        self.inverse_tolerance = 0.0;
        if self.tolerance > 0.0 {
            self.inverse_tolerance = (1.0 / tolerance) as f32;
        }
    }
    pub(super) fn has_merge_tolerance(&self) -> bool {
        self.tolerance > 0.0
    }
    /// Finds a matching node or binds `index` to a new one, as `Bind`.
    pub(super) fn bind(&mut self, index: i32, position: Vec3<f32>, normal: Vec3<f32>) -> Binding {
        if self.size == 0 || self.size > self.bucket_count {
            self.resize(self.size);
        }
        let home = self.position_hash(position, self.bucket_count);
        if let Some(found) = self.search(home, position, normal) {
            return Binding {
                index: found,
                is_new: false,
            };
        }
        if self.inverse_tolerance > 0.0 {
            let center = self.cell(position);
            for offset in NEIGHBOURS {
                let neighbour = std::array::from_fn(|k| center[k].wrapping_add(offset[k]));
                let bucket = cell_hash(neighbour, self.bucket_count);
                if let Some(found) = self.search(bucket, position, normal) {
                    return Binding {
                        index: found,
                        is_new: false,
                    };
                }
            }
        }
        self.buckets[home].insert(
            0,
            MapNode {
                position,
                normal,
                index,
            },
        );
        self.size += 1;
        Binding {
            index,
            is_new: true,
        }
    }
    fn search(&self, bucket: usize, position: Vec3<f32>, normal: Vec3<f32>) -> Option<i32> {
        self.buckets[bucket]
            .iter()
            .find(|node| self.is_equal(node, position, normal))
            .map(|node| node.index)
    }
    fn is_equal(&self, node: &MapNode, position: Vec3<f32>, normal: Vec3<f32>) -> bool {
        if !self.positions_equal(node.position, position) {
            return false;
        }
        let cosine = node.normal.dot(&normal);
        cosine >= self.angle_cos || (self.merge_opposite && cosine <= -self.angle_cos)
    }
    fn positions_equal(&self, first: Vec3<f32>, second: Vec3<f32>) -> bool {
        if self.inverse_tolerance <= 0.0 {
            return first.is_equal(&second);
        }
        (first.x() - second.x()).abs() <= self.tolerance
            && (first.y() - second.y()).abs() <= self.tolerance
            && (first.z() - second.z()).abs() <= self.tolerance
    }
    /// `NCollection_BaseMap::beginResize`/`endResize` with the native rehash order.
    fn resize(&mut self, extent: usize) {
        let mut count = next_map_prime(extent);
        if count <= self.bucket_count {
            if !self.buckets.is_empty() {
                return;
            }
            count = self.bucket_count;
        }
        let mut buckets: Vec<Vec<MapNode>> = vec![Vec::new(); count + 1];
        for chain in std::mem::take(&mut self.buckets) {
            for node in chain {
                buckets[self.position_hash(node.position, count)].insert(0, node);
            }
        }
        self.buckets = buckets;
        self.bucket_count = count;
    }
    fn cell(&self, position: Vec3<f32>) -> [i64; 3] {
        let scaled = position * self.inverse_tolerance;
        [scaled.x(), scaled.y(), scaled.z()].map(truncate_to_i64)
    }
    fn position_hash(&self, position: Vec3<f32>, upper: usize) -> usize {
        if self.inverse_tolerance > 0.0 {
            return cell_hash(self.cell(position), upper);
        }
        // DJB2 over the three floats' bytes, each byte a sign-extended `char`.
        let mut hash: u32 = 0;
        let bytes = [position.x(), position.y(), position.z()].map(f32::to_ne_bytes);
        for byte in bytes.iter().flatten() {
            let signed = i32::from(*byte as i8) as u32;
            hash = (hash << 5).wrapping_add(hash) ^ signed;
        }
        hash as usize % upper + 1
    }
}
/// `static_cast<int64_t>(float)` on x86-64: truncation, with the "integer indefinite" value
/// for NaN and out-of-range inputs.
fn truncate_to_i64(value: f32) -> i64 {
    const BOUND: f32 = 9_223_372_036_854_775_808.0;
    if value.is_nan() || value >= BOUND || value < -BOUND {
        return i64::MIN;
    }
    value as i64
}
/// `vec3iHashCode`, copied natively from NCollection_CellFilter.
fn cell_hash(cell: [i64; 3], upper: usize) -> usize {
    const SHIFT: u32 = (i64::BITS - 1) / 3;
    let mut hash: u64 = 0;
    for coordinate in cell {
        hash = (hash << SHIFT) ^ coordinate as u64;
    }
    (hash % upper as u64) as usize + 1
}
