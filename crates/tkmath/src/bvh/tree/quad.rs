// Created on: 2016-06-20
// Created by: Denis BOGOLEPOV
// Copyright (c) 2016 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKMath/BVH/BVH_QuadTree.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_Tree<T, N, BVH_QuadTree>`: children stored contiguously after a first index.
use super::{BvhQuadTree, BvhTree};
use crate::bvh::vector::BvhVector;
use rustycad_tkernel::standard::Failure;

impl<V: BvhVector> BvhTree<V, BvhQuadTree> {
    /// Returns index of the K-th child: first child index + K, unchecked against
    /// the child count as upstream; `quad.child::<3>(0)?`.
    pub fn child<const K: usize>(&self, node: i32) -> Result<i32, Failure> {
        Ok(self.node_info(node)?.y + K as i32)
    }
}
