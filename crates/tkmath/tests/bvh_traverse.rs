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

//! Direct BVH_Traverse_Test.cxx translation at pinned OCCT 3d097a0.
#![allow(non_snake_case, non_camel_case_types, unused_mut, dead_code)]
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bvh::{
    BvhBaseTraverse, BvhBinnedBuilder, BvhBox, BvhBuilder, BvhObject, BvhPairTraverse, BvhSet,
    BvhTraverse, BvhTree, BvhTriangulation, BvhVec3d, BvhVec4i, array,
};
use std::cell::Cell;

fn assert_near(left: f64, right: f64, tolerance: f64) {
    assert!(
        (left - right).abs() <= tolerance,
        "{left} != {right} within {tolerance}"
    );
}

// =======================================================================================
// Test implementations of BVH_Traverse
// =======================================================================================

/// Simple traverse implementation that counts all elements
#[derive(Default)]
struct BVH_CountAllElements {
    myAcceptedCount: i32,
}
impl BVH_CountAllElements {
    fn AcceptedCount(&self) -> i32 {
        self.myAcceptedCount
    }
    fn Reset(&mut self) {
        self.myAcceptedCount = 0;
    }
}
impl BvhBaseTraverse<f64> for BVH_CountAllElements {}
impl BvhTraverse<BvhVec3d, f64> for BVH_CountAllElements {
    fn reject_node(&self, _: &BvhVec3d, _: &BvhVec3d, theMetric: &mut f64) -> bool {
        *theMetric = 0.0; // All nodes have same metric
        false // Never reject
    }
    fn accept(&mut self, _: i32, _: &f64) -> Result<bool, Failure> {
        self.myAcceptedCount += 1;
        Ok(true)
    }
}

/// Traverse that rejects nodes outside a bounding box
struct BVH_BoxSelector {
    myBox: BvhBox<BvhVec3d>,
    myAcceptedCount: i32,
}
impl BVH_BoxSelector {
    fn new(theBox: BvhBox<BvhVec3d>) -> Self {
        Self {
            myBox: theBox,
            myAcceptedCount: 0,
        }
    }
    fn AcceptedCount(&self) -> i32 {
        self.myAcceptedCount
    }
    fn Reset(&mut self) {
        self.myAcceptedCount = 0;
    }
}
impl BvhBaseTraverse<f64> for BVH_BoxSelector {}
impl BvhTraverse<BvhVec3d, f64> for BVH_BoxSelector {
    fn reject_node(&self, theMin: &BvhVec3d, theMax: &BvhVec3d, theMetric: &mut f64) -> bool {
        // Reject if box doesn't intersect with selection box
        *theMetric = 0.0;
        self.myBox.is_out_corners(theMin, theMax)
    }
    fn accept(&mut self, _: i32, _: &f64) -> Result<bool, Failure> {
        self.myAcceptedCount += 1;
        Ok(true)
    }
}

/// Traverse with distance-based metric and early termination
struct BVH_DistanceSelector {
    myPoint: BvhVec3d,
    myMaxDistSq: f64,
    myMinDistSq: f64,
    myAcceptedCount: i32,
    myClosestIndex: i32,
}
impl BVH_DistanceSelector {
    fn new(thePoint: BvhVec3d, theMaxDist: f64) -> Self {
        Self {
            myPoint: thePoint,
            myMaxDistSq: theMaxDist * theMaxDist,
            myMinDistSq: f64::MAX,
            myAcceptedCount: 0,
            myClosestIndex: -1,
        }
    }
    fn AcceptedCount(&self) -> i32 {
        self.myAcceptedCount
    }
    fn ClosestIndex(&self) -> i32 {
        self.myClosestIndex
    }
    fn MinDistance(&self) -> f64 {
        self.myMinDistSq.sqrt()
    }
    fn PointBoxSquareDistance(thePoint: &BvhVec3d, theMin: &BvhVec3d, theMax: &BvhVec3d) -> f64 {
        let mut aDist = 0.0;
        for i in 0..3 {
            if thePoint[i] < theMin[i] {
                let d = theMin[i] - thePoint[i];
                aDist += d * d;
            } else if thePoint[i] > theMax[i] {
                let d = thePoint[i] - theMax[i];
                aDist += d * d;
            }
        }
        aDist
    }
}
impl BvhBaseTraverse<f64> for BVH_DistanceSelector {
    fn is_metric_better(&self, theLeft: &f64, theRight: &f64) -> bool {
        theLeft < theRight // Closer is better
    }
    fn reject_metric(&self, theMetric: &f64) -> bool {
        *theMetric > self.myMaxDistSq
    }
}
impl BvhTraverse<BvhVec3d, f64> for BVH_DistanceSelector {
    fn reject_node(&self, theMin: &BvhVec3d, theMax: &BvhVec3d, theMetric: &mut f64) -> bool {
        // Compute squared distance from point to box
        *theMetric = Self::PointBoxSquareDistance(&self.myPoint, theMin, theMax);
        *theMetric > self.myMaxDistSq
    }
    fn accept(&mut self, theIndex: i32, theMetric: &f64) -> Result<bool, Failure> {
        self.myAcceptedCount += 1;
        if *theMetric < self.myMinDistSq {
            self.myMinDistSq = *theMetric;
            self.myClosestIndex = theIndex;
        }
        Ok(true)
    }
}

/// Traverse with early stopping after finding N elements
struct BVH_LimitedSelector {
    myMaxCount: i32,
    myAcceptedCount: i32,
}
impl BVH_LimitedSelector {
    fn new(theMaxCount: i32) -> Self {
        Self {
            myMaxCount: theMaxCount,
            myAcceptedCount: 0,
        }
    }
    fn AcceptedCount(&self) -> i32 {
        self.myAcceptedCount
    }
}
impl BvhBaseTraverse<f64> for BVH_LimitedSelector {
    fn stop(&self) -> bool {
        self.myAcceptedCount >= self.myMaxCount
    }
}
impl BvhTraverse<BvhVec3d, f64> for BVH_LimitedSelector {
    fn reject_node(&self, _: &BvhVec3d, _: &BvhVec3d, theMetric: &mut f64) -> bool {
        *theMetric = 0.0;
        false
    }
    fn accept(&mut self, _: i32, _: &f64) -> Result<bool, Failure> {
        self.myAcceptedCount += 1;
        Ok(true)
    }
}

// =======================================================================================
// Test implementations of BVH_PairTraverse
// =======================================================================================

/// Simple pair traverse that counts all pairs
#[derive(Default)]
struct BVH_CountAllPairs {
    myAcceptedCount: i32,
}
impl BVH_CountAllPairs {
    fn AcceptedCount(&self) -> i32 {
        self.myAcceptedCount
    }
}
impl BvhBaseTraverse<f64> for BVH_CountAllPairs {}
impl BvhPairTraverse<BvhVec3d, f64> for BVH_CountAllPairs {
    fn reject_node(
        &self,
        _: &BvhVec3d,
        _: &BvhVec3d,
        _: &BvhVec3d,
        _: &BvhVec3d,
        theMetric: &mut f64,
    ) -> bool {
        *theMetric = 0.0;
        false // Never reject
    }
    fn accept(&mut self, _: i32, _: i32) -> Result<bool, Failure> {
        self.myAcceptedCount += 1;
        Ok(true)
    }
}

/// Pair traverse that only accepts overlapping boxes
#[derive(Default)]
struct BVH_OverlapDetector {
    myOverlapCount: i32,
    // `mutable int myRejectCount` written by the const RejectNode.
    myRejectCount: Cell<i32>,
}
impl BVH_OverlapDetector {
    fn OverlapCount(&self) -> i32 {
        self.myOverlapCount
    }
    fn RejectCount(&self) -> i32 {
        self.myRejectCount.get()
    }
}
impl BvhBaseTraverse<f64> for BVH_OverlapDetector {}
impl BvhPairTraverse<BvhVec3d, f64> for BVH_OverlapDetector {
    fn reject_node(
        &self,
        theMin1: &BvhVec3d,
        theMax1: &BvhVec3d,
        theMin2: &BvhVec3d,
        theMax2: &BvhVec3d,
        theMetric: &mut f64,
    ) -> bool {
        self.myRejectCount.set(self.myRejectCount.get() + 1);
        *theMetric = 0.0;
        // Reject if boxes don't overlap
        let aBox1 = BvhBox::from_corners(*theMin1, *theMax1);
        aBox1.is_out_corners(theMin2, theMax2)
    }
    fn accept(&mut self, _: i32, _: i32) -> Result<bool, Failure> {
        // For this test, if we reach Accept, it means the bounding boxes overlap.
        // In a real implementation, you would check actual triangle-triangle intersection here.
        // For testing purposes, we count all pairs whose bounding boxes overlap.
        self.myOverlapCount += 1;
        Ok(true)
    }
}

// =======================================================================================
// Helper functions
// =======================================================================================

/// Creates a simple triangulation for testing
fn CreateSimpleTriangulationBVH(theNumTriangles: i32) -> BvhTree<BvhVec3d> {
    let mut aTriangulation = BvhTriangulation::<BvhVec3d>::default();

    for i in 0..theNumTriangles {
        let x = (i * 2) as f64;
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(x, 0.0, 0.0));
        array::append(
            &mut aTriangulation.vertices,
            BvhVec3d::new(x + 1.0, 1.0, 0.0),
        );
        array::append(
            &mut aTriangulation.vertices,
            BvhVec3d::new(x + 2.0, 0.0, 0.0),
        );
        array::append(
            &mut aTriangulation.elements,
            BvhVec4i::new(i * 3, i * 3 + 1, i * 3 + 2, 0),
        );
    }

    let mut aBVH = BvhTree::<BvhVec3d>::default();
    let aBuilder = BvhBinnedBuilder::<BvhVec3d>::default();
    let aBox = aTriangulation.bounding_box().unwrap();
    aBuilder
        .build(&mut aTriangulation, &mut aBVH, &aBox)
        .unwrap();

    aBVH
}

/// The two 3-triangle triangulations of the pair tests: x = 0..3 and x = 100..103.
fn SeparatedTriangulations() -> (BvhTriangulation<BvhVec3d>, BvhTriangulation<BvhVec3d>) {
    let mut aTri1 = BvhTriangulation::<BvhVec3d>::default();
    let mut aTri2 = BvhTriangulation::<BvhVec3d>::default();

    for (aTri, anOrigin) in [(&mut aTri1, 0.0), (&mut aTri2, 100.0)] {
        for i in 0..3 {
            let x = anOrigin + i as f64;
            array::append(&mut aTri.vertices, BvhVec3d::new(x, 0.0, 0.0));
            array::append(&mut aTri.vertices, BvhVec3d::new(x + 0.5, 1.0, 0.0));
            array::append(&mut aTri.vertices, BvhVec3d::new(x + 1.0, 0.0, 0.0));
            array::append(
                &mut aTri.elements,
                BvhVec4i::new(i * 3, i * 3 + 1, i * 3 + 2, 0),
            );
        }
    }
    (aTri1, aTri2)
}

include!("bvh_traverse/suites_0.rs");
include!("bvh_traverse/suites_1.rs");
