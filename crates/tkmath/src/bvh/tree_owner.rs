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
// Rust translation of FoundationClasses/TKMath/BVH/BVH_Traverse.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `BVHSetType::BVH()` member that `BVH_Traverse::Select()` requires.
use super::tree::BvhTree;
use super::vector::BvhVector;
use rustycad_tkernel::standard::Failure;

/// A set that owns its BVH tree and builds it on demand, as `BVH()` does for
/// `BVH_PrimitiveSet` and `BVH_Geometry`.
///
/// Upstream traversals keep a raw set pointer and a tree handle side by side.
/// Safe Rust cannot borrow the tree while `Accept` reads the set, so the tree
/// is moved out for the traversal and moved back afterwards. During that time
/// the set holds an empty tree.
/// Example: `let tree = set.take_built_bvh()?; ...; set.restore_bvh(tree);`.
pub trait BvhTreeOwner<V: BvhVector> {
    /// `BVH()`: rebuilds the tree while dirty, then moves it out of the set.
    fn take_built_bvh(&mut self) -> Result<BvhTree<V>, Failure>;
    /// Moves a tree taken by [`BvhTreeOwner::take_built_bvh`] back into the set.
    fn restore_bvh(&mut self, bvh: BvhTree<V>);
}
