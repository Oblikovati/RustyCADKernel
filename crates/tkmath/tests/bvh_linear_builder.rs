// Copyright (c) 2025 OPEN CASCADE SAS
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

//! Direct BVH_LinearBuilder_Test.cxx translation at pinned OCCT 3d097a0.
#![allow(non_snake_case, unused_mut)]
use rustycad_tkmath::bvh::{
    BvhBox, BvhBoxSet, BvhBuilder, BvhLinearBuilder, BvhPrimitiveSetAccess, BvhVec2d, BvhVec3d,
    BvhVector,
};
use std::sync::Arc;

/// `new BVH_LinearBuilder<double, N>(leaf, depth)` as a builder handle.
fn linear<V: BvhVector>(leaf: i32, depth: i32) -> Arc<dyn BvhBuilder<V>> {
    Arc::new(BvhLinearBuilder::<V>::new(leaf, depth))
}

/// `new BVH_LinearBuilder<double, 3>()` as a builder handle.
fn default_linear() -> Arc<dyn BvhBuilder<BvhVec3d>> {
    Arc::new(BvhLinearBuilder::<BvhVec3d>::default())
}

#[test]
fn BVH_LinearBuilderTest__DefaultConstructor() {
    let aBuilder = BvhLinearBuilder::<BvhVec3d>::default();

    assert!(aBuilder.builder_transient().leaf_node_size() > 0);
    assert!(aBuilder.builder_transient().max_tree_depth() > 0);
}

#[test]
fn BVH_LinearBuilderTest__CustomParameters() {
    let aBuilder = BvhLinearBuilder::<BvhVec3d>::new(5, 20);

    assert_eq!(aBuilder.builder_transient().leaf_node_size(), 5);
    assert_eq!(aBuilder.builder_transient().max_tree_depth(), 20);
}

#[test]
fn BVH_LinearBuilderTest__BuildEmptySet() {
    let aBuilder = default_linear();
    let mut aBoxSet = BvhBoxSet::<BvhVec3d>::new(Some(aBuilder));

    aBoxSet.build().unwrap();

    let aBVH = aBoxSet.bvh().unwrap();
    assert_eq!(aBVH.length(), 0);
}

#[test]
fn BVH_LinearBuilderTest__BuildSingleElement() {
    let aBuilder = default_linear();
    let mut aBoxSet = BvhBoxSet::<BvhVec3d>::new(Some(aBuilder));

    aBoxSet.add(
        0,
        BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0)),
    );
    aBoxSet.build().unwrap();

    let aBVH = aBoxSet.bvh().unwrap();
    assert_eq!(aBVH.length(), 1);
    assert!(aBVH.is_outer(0).unwrap());
}

#[test]
fn BVH_LinearBuilderTest__BuildMultipleElements() {
    let aBuilder = linear::<BvhVec3d>(1, 32);
    let mut aBoxSet = BvhBoxSet::<BvhVec3d>::new(Some(aBuilder));

    // Add boxes along X axis
    for i in 0..10 {
        let aBox = BvhBox::from_corners(
            BvhVec3d::new(i as f64 * 2.0, 0.0, 0.0),
            BvhVec3d::new(i as f64 * 2.0 + 1.0, 1.0, 1.0),
        );
        aBoxSet.add(i, aBox);
    }

    aBoxSet.build().unwrap();

    let aBVH = aBoxSet.bvh().unwrap();
    assert!(aBVH.length() > 1);
    assert!(aBVH.depth() > 0);
}

#[test]
fn BVH_LinearBuilderTest__MortonCodeSorting() {
    // Linear builder uses Morton codes for spatial sorting
    let aBuilder = linear::<BvhVec3d>(1, 32);
    let mut aBoxSet = BvhBoxSet::<BvhVec3d>::new(Some(aBuilder));

    // Add boxes in a pattern that tests Morton code sorting
    // Diagonal pattern should group nearby boxes
    for i in 0..8 {
        let x = if i & 1 != 0 { 10.0 } else { 0.0 };
        let y = if i & 2 != 0 { 10.0 } else { 0.0 };
        let z = if i & 4 != 0 { 10.0 } else { 0.0 };

        let aBox = BvhBox::from_corners(
            BvhVec3d::new(x, y, z),
            BvhVec3d::new(x + 1.0, y + 1.0, z + 1.0),
        );
        aBoxSet.add(i, aBox);
    }

    aBoxSet.build().unwrap();

    let aBVH = aBoxSet.bvh().unwrap();
    assert!(aBVH.length() > 1);

    // SAH should be reasonable for spatially sorted data
    let aSAH = aBVH.estimate_sah().unwrap();
    assert!(aSAH > 0.0);
}

#[test]
fn BVH_LinearBuilderTest__LeafNodeSizeRespected() {
    let aLeafSize = 3;
    let aBuilder = linear::<BvhVec3d>(aLeafSize, 32);
    let mut aBoxSet = BvhBoxSet::<BvhVec3d>::new(Some(aBuilder));

    for i in 0..10 {
        let aBox = BvhBox::from_corners(
            BvhVec3d::new(i as f64 * 2.0, 0.0, 0.0),
            BvhVec3d::new(i as f64 * 2.0 + 1.0, 1.0, 1.0),
        );
        aBoxSet.add(i, aBox);
    }

    aBoxSet.build().unwrap();

    let aBVH = aBoxSet.bvh().unwrap();

    // Check that leaf nodes don't exceed leaf size
    for i in 0..aBVH.length() {
        if aBVH.is_outer(i).unwrap() {
            let aNbPrims = aBVH.nb_primitives(i).unwrap();
            assert!(aNbPrims <= aLeafSize);
        }
    }
}

#[test]
fn BVH_LinearBuilderTest__Build2D() {
    let aBuilder = linear::<BvhVec2d>(1, 32);
    let mut aBoxSet = BvhBoxSet::<BvhVec2d>::new(Some(aBuilder));

    for i in 0..10 {
        let aBox = BvhBox::from_corners(
            BvhVec2d::new(i as f64 * 2.0, 0.0),
            BvhVec2d::new(i as f64 * 2.0 + 1.0, 1.0),
        );
        aBoxSet.add(i, aBox);
    }

    aBoxSet.build().unwrap();

    let aBVH = aBoxSet.bvh().unwrap();
    assert!(aBVH.length() > 1);
}

// Note: Float tests skipped due to BVH_BoxSet::Center return type issue

#[test]
fn BVH_LinearBuilderTest__LargeDataSet() {
    let aBuilder = linear::<BvhVec3d>(4, 32);
    let mut aBoxSet = BvhBoxSet::<BvhVec3d>::new(Some(aBuilder));

    // Add many boxes - this tests the Morton code optimization
    let mut aCount = 0;
    for x in 0..10 {
        for y in 0..10 {
            for z in 0..10 {
                let aBox = BvhBox::from_corners(
                    BvhVec3d::new(x as f64 * 2.0, y as f64 * 2.0, z as f64 * 2.0),
                    BvhVec3d::new(
                        x as f64 * 2.0 + 1.0,
                        y as f64 * 2.0 + 1.0,
                        z as f64 * 2.0 + 1.0,
                    ),
                );
                aBoxSet.add(aCount, aBox);
                aCount += 1;
            }
        }
    }

    aBoxSet.build().unwrap();

    let aBVH = aBoxSet.bvh().unwrap();
    assert!(aBVH.length() > 1);

    // Verify tree covers all primitives
    let mut aTotalPrims = 0;
    for i in 0..aBVH.length() {
        if aBVH.is_outer(i).unwrap() {
            aTotalPrims += aBVH.nb_primitives(i).unwrap();
        }
    }
    assert_eq!(aTotalPrims, 1000);
}

#[test]
fn BVH_LinearBuilderTest__MaxDepthParameter() {
    // Linear builder uses max depth parameter but may exceed it in some cases
    let aMaxDepth = 10;
    let aBuilder = linear::<BvhVec3d>(1, aMaxDepth);
    let mut aBoxSet = BvhBoxSet::<BvhVec3d>::new(Some(aBuilder));

    for i in 0..100 {
        let aBox = BvhBox::from_corners(
            BvhVec3d::new(i as f64 * 2.0, 0.0, 0.0),
            BvhVec3d::new(i as f64 * 2.0 + 1.0, 1.0, 1.0),
        );
        aBoxSet.add(i, aBox);
    }

    aBoxSet.build().unwrap();

    let aBVH = aBoxSet.bvh().unwrap();
    // Tree should have a reasonable depth
    assert!(aBVH.depth() > 0);
    assert!(aBVH.depth() < 20);
}

#[test]
fn BVH_LinearBuilderTest__ClusteredData() {
    let aBuilder = linear::<BvhVec3d>(1, 32);
    let mut aBoxSet = BvhBoxSet::<BvhVec3d>::new(Some(aBuilder));

    // Create two clusters far apart
    // Morton codes should group each cluster together
    for i in 0..10 {
        let offset = i as f64 * 0.1;
        // Cluster 1 near origin
        let aBox1 = BvhBox::from_corners(
            BvhVec3d::new(offset, offset, offset),
            BvhVec3d::new(offset + 0.1, offset + 0.1, offset + 0.1),
        );
        aBoxSet.add(i, aBox1);

        // Cluster 2 far from origin
        let aBox2 = BvhBox::from_corners(
            BvhVec3d::new(100.0 + offset, 100.0 + offset, 100.0 + offset),
            BvhVec3d::new(100.1 + offset, 100.1 + offset, 100.1 + offset),
        );
        aBoxSet.add(i + 10, aBox2);
    }

    aBoxSet.build().unwrap();

    let aBVH = aBoxSet.bvh().unwrap();
    assert!(aBVH.length() > 1);

    // SAH should be reasonable for clustered data
    let aSAH = aBVH.estimate_sah().unwrap();
    assert!(aSAH > 0.0);
}

#[test]
fn BVH_LinearBuilderTest__OverlappingBoxes() {
    let aBuilder = linear::<BvhVec3d>(1, 32);
    let mut aBoxSet = BvhBoxSet::<BvhVec3d>::new(Some(aBuilder));

    // Add overlapping boxes
    for i in 0..10 {
        let aBox = BvhBox::from_corners(
            BvhVec3d::new(i as f64 * 0.5, 0.0, 0.0),
            BvhVec3d::new(i as f64 * 0.5 + 2.0, 1.0, 1.0),
        );
        aBoxSet.add(i, aBox);
    }

    aBoxSet.build().unwrap();

    let aBVH = aBoxSet.bvh().unwrap();
    assert!(aBVH.length() > 1);
}

#[test]
fn BVH_LinearBuilderTest__IdenticalBoxes() {
    let aBuilder = linear::<BvhVec3d>(1, 32);
    let mut aBoxSet = BvhBoxSet::<BvhVec3d>::new(Some(aBuilder));

    // Add identical boxes (same Morton code)
    for i in 0..10 {
        let aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));
        aBoxSet.add(i, aBox);
    }

    aBoxSet.build().unwrap();

    let aBVH = aBoxSet.bvh().unwrap();
    assert!(aBVH.length() >= 1);
}

#[test]
fn BVH_LinearBuilderTest__CompareWithBinnedBuilder() {
    // Linear builder should produce reasonable trees compared to binned builder
    let aLinearBuilder = linear::<BvhVec3d>(1, 32);
    let mut aLinearSet = BvhBoxSet::<BvhVec3d>::new(Some(aLinearBuilder));

    for i in 0..50 {
        let aBox = BvhBox::from_corners(
            BvhVec3d::new(i as f64 * 2.0, 0.0, 0.0),
            BvhVec3d::new(i as f64 * 2.0 + 1.0, 1.0, 1.0),
        );
        aLinearSet.add(i, aBox);
    }

    aLinearSet.build().unwrap();

    let aLinearBVH = aLinearSet.bvh().unwrap();

    // Linear builder produces a valid tree
    assert!(aLinearBVH.length() > 1);
    assert!(aLinearBVH.depth() > 0);

    // SAH should be positive
    let aSAH = aLinearBVH.estimate_sah().unwrap();
    assert!(aSAH > 0.0);
}

#[test]
fn BVH_LinearBuilderTest__NegativeCoordinates() {
    let aBuilder = linear::<BvhVec3d>(1, 32);
    let mut aBoxSet = BvhBoxSet::<BvhVec3d>::new(Some(aBuilder));

    // Add boxes with negative coordinates
    for i in 0..10 {
        let aBox = BvhBox::from_corners(
            BvhVec3d::new(-10.0 + i as f64 * 2.0, -5.0, -5.0),
            BvhVec3d::new(-9.0 + i as f64 * 2.0, -4.0, -4.0),
        );
        aBoxSet.add(i, aBox);
    }

    aBoxSet.build().unwrap();

    let aBVH = aBoxSet.bvh().unwrap();
    assert!(aBVH.length() > 1);
}
