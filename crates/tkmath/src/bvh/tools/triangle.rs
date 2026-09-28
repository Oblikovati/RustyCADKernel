// Created by: Eugeny MALTCHIKOV
// Created on: 2019-04-17
// Copyright (c) 2019 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKMath/BVH/BVH_Tools.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Point-triangle projection by Voronoi region tests, in the native operation order.
use crate::bvh::scalar::BvhScalar;
use crate::bvh::vector::BvhVector;

/// `BVH_Tools::BVH_PrjStateInTriangle`: the triangle feature nearest to the point.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PrjStateInTriangle {
    /// `BVH_PrjStateInTriangle_VERTEX`.
    #[default]
    Vertex,
    /// `BVH_PrjStateInTriangle_EDGE`.
    Edge,
    /// `BVH_PrjStateInTriangle_INNER`.
    Inner,
}

/// The optional output arguments `thePrjState`, `theNumberOfFirstNode` and
/// `theNumberOfLastNode`. An interior projection writes only `state` and leaves
/// the node numbers untouched, as upstream does.
/// Example: `let mut out = TriangleProjectionState::default();`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TriangleProjectionState {
    /// Nearest feature kind.
    pub state: PrjStateInTriangle,
    /// First node (0..=2) of the nearest vertex or edge.
    pub first_node: i32,
    /// Last node (0..=2) of the nearest vertex or edge.
    pub last_node: i32,
}

fn set_vertex(output: &mut Option<&mut TriangleProjectionState>, vertex: i32) {
    set_feature(output, PrjStateInTriangle::Vertex, vertex, vertex);
}

fn set_feature(
    output: &mut Option<&mut TriangleProjectionState>,
    state: PrjStateInTriangle,
    first: i32,
    last: i32,
) {
    if let Some(output) = output {
        output.state = state;
        output.first_node = first;
        output.last_node = last;
    }
}

/// `ProjectToEdge`: `start + edge * (d1 / (d1 + d2))`.
fn project_to_edge<V: BvhVector>(start: &V, edge: &V, first: V::Scalar, second: V::Scalar) -> V {
    *start + *edge * (first / (first + second))
}

/// The six dot products shared by the vertex and edge region tests.
struct RegionDots<T> {
    ab_ap: T,
    ac_ap: T,
    ba_bp: T,
    bc_bp: T,
    cb_cp: T,
    ca_cp: T,
}

/// Nearest point of triangle (n0, n1, n2) to `point`, filling `output` if given.
///
/// A degenerate triangle returns the centroid (divided with the NCollection
/// operator, a reciprocal product in 3D/4D) and reports vertex 0.
/// Example: `point_triangle_projection(&p, &a, &b, &c, Some(&mut state))`.
pub fn point_triangle_projection<V: BvhVector>(
    point: &V,
    node0: &V,
    node1: &V,
    node2: &V,
    mut output: Option<&mut TriangleProjectionState>,
) -> V {
    let (ab, ac, bc) = (*node1 - *node0, *node2 - *node0, *node2 - *node1);
    let (ap, bp, cp) = (*point - *node0, *point - *node1, *point - *node2);
    let zero = V::Scalar::ZERO;
    let ab_ap = ab.inner_product(&ap);
    let ac_ap = ac.inner_product(&ap);
    if ab_ap <= zero && ac_ap <= zero {
        set_vertex(&mut output, 0);
        return *node0;
    }
    let ba_bp = -ab.inner_product(&bp);
    let bc_bp = bc.inner_product(&bp);
    if ba_bp <= zero && bc_bp <= zero {
        set_vertex(&mut output, 1);
        return *node1;
    }
    let cb_cp = -bc.inner_product(&cp);
    let ca_cp = -ac.inner_product(&cp);
    if ca_cp <= zero && cb_cp <= zero {
        set_vertex(&mut output, 2);
        return *node2;
    }
    let dots = RegionDots {
        ab_ap,
        ac_ap,
        ba_bp,
        bc_bp,
        cb_cp,
        ca_cp,
    };
    project_outside_vertices(
        &[*node0, *node1, *node2],
        [ab, ac, bc],
        [bp, cp],
        &dots,
        output,
    )
}

fn project_outside_vertices<V: BvhVector>(
    nodes: &[V; 3],
    [ab, ac, bc]: [V; 3],
    [bp, cp]: [V; 2],
    dots: &RegionDots<V::Scalar>,
    mut output: Option<&mut TriangleProjectionState>,
) -> V {
    let zero = V::Scalar::ZERO;
    let ac_bp = ac.inner_product(&bp);
    let vc = dots.ab_ap * ac_bp + dots.ba_bp * dots.ac_ap;
    if vc <= zero && dots.ab_ap > zero && dots.ba_bp > zero {
        set_feature(&mut output, PrjStateInTriangle::Edge, 0, 1);
        return project_to_edge(&nodes[0], &ab, dots.ab_ap, dots.ba_bp);
    }
    let ab_cp = ab.inner_product(&cp);
    let va = dots.ba_bp * dots.ca_cp - ab_cp * ac_bp;
    if va <= zero && dots.bc_bp > zero && dots.cb_cp > zero {
        set_feature(&mut output, PrjStateInTriangle::Edge, 1, 2);
        return project_to_edge(&nodes[1], &bc, dots.bc_bp, dots.cb_cp);
    }
    let vb = ab_cp * dots.ac_ap + dots.ab_ap * dots.ca_cp;
    if vb <= zero && dots.ac_ap > zero && dots.ca_cp > zero {
        set_feature(&mut output, PrjStateInTriangle::Edge, 2, 0);
        return project_to_edge(&nodes[0], &ac, dots.ac_ap, dots.ca_cp);
    }
    project_inside(nodes, [va, vb, vc], output)
}

fn project_inside<V: BvhVector>(
    nodes: &[V; 3],
    [va, vb, vc]: [V::Scalar; 3],
    mut output: Option<&mut TriangleProjectionState>,
) -> V {
    let norm = va + vb + vc;
    let scale = va.absolute() + vb.absolute() + vc.absolute();
    if norm <= V::Scalar::EPSILON * scale {
        set_vertex(&mut output, 0);
        return (nodes[0] + nodes[1] + nodes[2]) / V::Scalar::from_double(3.0);
    }
    if let Some(output) = output {
        output.state = PrjStateInTriangle::Inner;
    }
    (nodes[0] * va + nodes[1] * vb + nodes[2] * vc) / norm
}

/// Square distance between a point and its triangle projection.
/// Example: `point_triangle_square_distance(&p, &a, &b, &c)`.
pub fn point_triangle_square_distance<V: BvhVector>(
    point: &V,
    node0: &V,
    node1: &V,
    node2: &V,
) -> V::Scalar {
    let projection = point_triangle_projection(point, node0, node1, node2, None);
    let offset = projection - *point;
    offset.inner_product(&offset)
}
