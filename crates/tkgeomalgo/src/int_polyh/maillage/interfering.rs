// Created on: 1999-03-05
// Created by: Fabrice SERVANT
// Copyright (c) 1999-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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
// Rust translation of IntPolyh_BoxBndTreeSelector and GetInterferingTriangles of
// ModelingAlgorithms/TKGeomAlgo/IntPolyh/IntPolyh_MaillageAffinage.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The pairs of triangles of the two surfaces whose boxes interfere, found by a parallel
//! descent of two BVH trees of the triangle boxes and sorted.
use crate::int_polyh::array::{IntPolyhArrayOfPoints, IntPolyhArrayOfTriangles};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::stl::sort_by;
use rustycad_tkmath::bnd::box3_to_bvh;
use rustycad_tkmath::bvh::constants::MAX_TREE_DEPTH;
use rustycad_tkmath::bvh::{
    BvhBaseTraverse, BvhBox, BvhBoxSet, BvhLinearBuilder, BvhPairTraverse, BvhSet, BvhTreeOwner,
    BvhVec3d,
};
use std::sync::Arc;

/// `IntPolyh_BoxBndTree`: the boxes of the triangles of one surface.
type BoxBndTree = BvhBoxSet<BvhVec3d, i32>;

/// `IntPolyh_IndexedDataMapOfIntegerListOfInteger`: each triangle of the first surface
/// with the triangles of the second one whose boxes interfere with its box, the first
/// ones in increasing order (the order of the sorted pairs).
pub(crate) type InterferingTriangles = Vec<(i32, Vec<i32>)>;

/// `IntPolyh_BoxBndTreeSelector`: accepts the pairs of elements whose boxes interfere.
struct BoxBndTreeSelector<'s> {
    set1: &'s BoxBndTree,
    set2: &'s BoxBndTree,
    pairs: Vec<(i32, i32)>,
}

impl BvhBaseTraverse<f64> for BoxBndTreeSelector<'_> {}

impl BvhPairTraverse<BvhVec3d, f64> for BoxBndTreeSelector<'_> {
    fn reject_node(
        &self,
        cmin1: &BvhVec3d,
        cmax1: &BvhVec3d,
        cmin2: &BvhVec3d,
        cmax2: &BvhVec3d,
        _: &mut f64,
    ) -> bool {
        BvhBox::from_corners(*cmin1, *cmax1).is_out_corners(cmin2, cmax2)
    }
    fn accept(&mut self, id1: i32, id2: i32) -> Result<bool, Failure> {
        if self
            .set1
            .element_box(id1)?
            .is_out(&self.set2.element_box(id2)?)
        {
            return Ok(false);
        }
        self.pairs
            .push((self.set1.element(id1)?, self.set2.element(id2)?));
        Ok(true)
    }
}

/// The boxes of the triangles that may intersect and are not degenerated.
fn fill_tree(
    tree: &mut BoxBndTree,
    triangles: &mut IntPolyhArrayOfTriangles,
    points: &IntPolyhArrayOfPoints,
) -> Result<(), Failure> {
    let nb_t: i32 = triangles.nb_items();
    tree.set_size(nb_t.max(0) as usize);
    for j in 0..nb_t {
        let triangle = &mut triangles[j];
        if !triangle.is_intersection_possible() || triangle.is_degenerated() {
            continue;
        }
        tree.add(j, box3_to_bvh(triangle.bounding_box(points))?);
    }
    Ok(())
}

/// `GetInterferingTriangles(theTriangles1, thePoints1, theTriangles2, thePoints2,
/// theCouples)`: empty when a surface has no candidate triangle.
pub(crate) fn get_interfering_triangles(
    (triangles1, points1): (&mut IntPolyhArrayOfTriangles, &IntPolyhArrayOfPoints),
    (triangles2, points2): (&mut IntPolyhArrayOfTriangles, &IntPolyhArrayOfPoints),
) -> Result<InterferingTriangles, Failure> {
    // One linear builder with leaves of 10 elements serves both trees.
    let builder = Arc::new(BvhLinearBuilder::<BvhVec3d>::new(10, MAX_TREE_DEPTH));
    let mut tree1 = BoxBndTree::new(Some(builder.clone()));
    let mut tree2 = BoxBndTree::new(Some(builder));
    fill_tree(&mut tree1, triangles1, points1)?;
    if tree1.size() == 0 {
        return Ok(Vec::new());
    }
    fill_tree(&mut tree2, triangles2, points2)?;
    if tree2.size() == 0 {
        return Ok(Vec::new());
    }
    tree1.build()?;
    tree2.build()?;
    let bvh1 = tree1.take_built_bvh()?;
    let bvh2 = tree2.take_built_bvh()?;
    let mut selector = BoxBndTreeSelector {
        set1: &tree1,
        set2: &tree2,
        pairs: Vec::new(),
    };
    selector.select_trees(Some(&bvh1), Some(&bvh2))?;
    let mut pairs: Vec<(i32, i32)> = selector.pairs;
    sort_by(&mut pairs, |a, b| a.0 < b.0 || (a.0 == b.0 && a.1 < b.1));
    Ok(group_pairs(&pairs))
}

/// The sorted pairs grouped by their first triangle.
fn group_pairs(pairs: &[(i32, i32)]) -> InterferingTriangles {
    let mut couples: InterferingTriangles = Vec::new();
    for &(id1, id2) in pairs {
        match couples.last_mut() {
            Some((key, list)) if *key == id1 => list.push(id2),
            _ => couples.push((id1, vec![id2])),
        }
    }
    couples
}

/// `theCouples.Seek(i_S1)`: the list of the triangle `key`, when it has one.
pub(crate) fn seek(couples: &InterferingTriangles, key: i32) -> Option<&Vec<i32>> {
    couples
        .binary_search_by_key(&key, |(k, _)| *k)
        .ok()
        .map(|index| &couples[index].1)
}
