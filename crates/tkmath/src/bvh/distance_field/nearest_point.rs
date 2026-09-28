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
// Rust translation of FoundationClasses/TKMath/BVH/BVH_DistanceField.lxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH::DistanceToBox` and `BVH::DirectionToNearestPoint`: X/Y/Z-only point
//! queries of the distance field (a W component is carried but not measured).
use super::BvhFieldVector;
use crate::bvh::scalar::{BvhScalar, std_max, std_min};

/// `BVH_DOT3(A, B)`: `A.x*B.x + A.y*B.y + A.z*B.z`, left to right.
pub(super) fn dot3<V: BvhFieldVector>(first: &V, second: &V) -> V::Scalar {
    first[0] * second[0] + first[1] * second[1] + first[2] * second[2]
}

/// `BVH::DistanceToBox`: square distance from a point to a box, exactly zero
/// when the clamped point equals the point.
/// Example: `distance_to_box(&point, &min, &max)`.
pub fn distance_to_box<V: BvhFieldVector>(point: &V, min: &V, max: &V) -> V::Scalar {
    let nearest = |axis: usize| std_min(std_max(point[axis], min[axis]), max[axis]);
    let (x, y, z) = (nearest(0), nearest(1), nearest(2));
    if x == point[0] && y == point[1] && z == point[2] {
        return V::Scalar::ZERO;
    }
    let (x, y, z) = (x - point[0], y - point[1], z - point[2]);
    x * x + y * y + z * z
}

/// The dot products of the vertex and edge Voronoi region tests.
struct RegionDots<T> {
    ab_ap: T,
    ac_ap: T,
    ba_bp: T,
    bc_bp: T,
    cb_cp: T,
    ca_cp: T,
}

/// The vectors the region tests use (`aAB`, `aAC`, `aBC`, `aAP`, `aBP`, `aCP`).
struct TriangleVectors<V> {
    ab: V,
    ac: V,
    bc: V,
    ap: V,
    bp: V,
    cp: V,
}

/// `BVH::DirectionToNearestPoint`: the vector from the nearest triangle point
/// to `point`, by Voronoi region tests.
///
/// Unlike `BVH_Tools::PointTriangleProjection`, the edge tests accept zero dot
/// products (`>=`) and the interior case multiplies by the reciprocal norm.
/// Example: `let direction = direction_to_nearest_point(&p, &a, &b, &c);`.
pub fn direction_to_nearest_point<V: BvhFieldVector>(point: &V, a: &V, b: &V, c: &V) -> V {
    let zero = V::Scalar::ZERO;
    let (ab, ac, ap) = (*b - *a, *c - *a, *point - *a);
    let ab_ap = dot3(&ab, &ap);
    let ac_ap = dot3(&ac, &ap);
    if ab_ap <= zero && ac_ap <= zero {
        return ap;
    }
    let (bc, bp) = (*c - *b, *point - *b);
    let ba_bp = -dot3(&ab, &bp);
    let bc_bp = dot3(&bc, &bp);
    if ba_bp <= zero && bc_bp <= zero {
        return bp;
    }
    let cp = *point - *c;
    let cb_cp = -dot3(&bc, &cp);
    let ca_cp = -dot3(&ac, &cp);
    if ca_cp <= zero && cb_cp <= zero {
        return cp;
    }
    let dots = RegionDots {
        ab_ap,
        ac_ap,
        ba_bp,
        bc_bp,
        cb_cp,
        ca_cp,
    };
    let vectors = TriangleVectors {
        ab,
        ac,
        bc,
        ap,
        bp,
        cp,
    };
    direction_outside_vertices(point, [a, b, c], &vectors, &dots)
}

fn direction_outside_vertices<V: BvhFieldVector>(
    point: &V,
    nodes: [&V; 3],
    vectors: &TriangleVectors<V>,
    dots: &RegionDots<V::Scalar>,
) -> V {
    let zero = V::Scalar::ZERO;
    let ac_bp = dot3(&vectors.ac, &vectors.bp);
    let vc = dots.ab_ap * ac_bp + dots.ba_bp * dots.ac_ap;
    if vc <= zero && dots.ab_ap >= zero && dots.ba_bp >= zero {
        return vectors.ap - vectors.ab * (dots.ab_ap / (dots.ab_ap + dots.ba_bp));
    }
    let ab_cp = dot3(&vectors.ab, &vectors.cp);
    let va = dots.ba_bp * dots.ca_cp - ab_cp * ac_bp;
    if va <= zero && dots.bc_bp >= zero && dots.cb_cp >= zero {
        return vectors.bp - vectors.bc * (dots.bc_bp / (dots.bc_bp + dots.cb_cp));
    }
    let vb = ab_cp * dots.ac_ap + dots.ab_ap * dots.ca_cp;
    if vb <= zero && dots.ac_ap >= zero && dots.ca_cp >= zero {
        return vectors.ap - vectors.ac * (dots.ac_ap / (dots.ac_ap + dots.ca_cp));
    }
    let one = V::Scalar::ONE;
    let norm = one / (va + vb + vc);
    let (u, v) = (va * norm, vb * norm);
    *point - (*nodes[0] * u + *nodes[1] * v + *nodes[2] * (one - u - v))
}
