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

//! Full UBTree/EBTree legacy suite, using the original one-dimensional bounds.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::{BoundingVolume, EBTree, TreeSelector, UBTree};
include!("ncollection_ubtree/support.rs");

#[test]
fn NCollection_UBTreeTest__EmptyTree() {
    let aTree: UBTree<i32, TestBnd1D> = UBTree::default();

    assert!(aTree.is_empty());

    let mut aSelector = TestSelector1D::new(TestBnd1D::new(0.0, 10.0));
    let nSel: i32 = aTree.select(&mut aSelector);
    assert_eq!(0, nSel);
}

#[test]
fn NCollection_UBTreeTest__AddAndSelect_SingleObject() {
    let mut aTree: UBTree<i32, TestBnd1D> = UBTree::default();

    aTree.add(1, TestBnd1D::new(0.0, 5.0));
    assert!(!aTree.is_empty());

    // Query overlapping
    let mut aSelector = TestSelector1D::new(TestBnd1D::new(3.0, 7.0));
    let nSel: i32 = aTree.select(&mut aSelector);
    assert_eq!(1, nSel);
    assert_eq!(1, aSelector.results.len());
    assert_eq!(1, aSelector.results[0]);

    // Query not overlapping
    let mut aSelector2 = TestSelector1D::new(TestBnd1D::new(6.0, 10.0));
    let nSel2: i32 = aTree.select(&mut aSelector2);
    assert_eq!(0, nSel2);
}

#[test]
fn NCollection_UBTreeTest__AddAndSelect_MultipleObjects() {
    let mut aTree: UBTree<i32, TestBnd1D> = UBTree::default();

    // Add intervals: [0,2], [3,5], [4,6], [8,10]
    aTree.add(1, TestBnd1D::new(0.0, 2.0));
    aTree.add(2, TestBnd1D::new(3.0, 5.0));
    aTree.add(3, TestBnd1D::new(4.0, 6.0));
    aTree.add(4, TestBnd1D::new(8.0, 10.0));

    // Query [3.5, 5.5] should overlap with objects 2 and 3
    let mut aSelector = TestSelector1D::new(TestBnd1D::new(3.5, 5.5));
    let nSel: i32 = aTree.select(&mut aSelector);
    assert_eq!(2, nSel);

    let mut aResults: Vec<i32> = aSelector.results.clone();
    aResults.sort();
    assert_eq!(2, aResults.len());
    assert_eq!(2, aResults[0]);
    assert_eq!(3, aResults[1]);

    // Query [0, 10] should overlap all
    let mut aSelector2 = TestSelector1D::new(TestBnd1D::new(0.0, 10.0));
    let nSel2: i32 = aTree.select(&mut aSelector2);
    assert_eq!(4, nSel2);
}

#[test]
fn NCollection_UBTreeTest__StopSelector() {
    let mut aTree: UBTree<i32, TestBnd1D> = UBTree::default();

    for i in 0..10 {
        aTree.add(i, TestBnd1D::new(f64::from(i), f64::from(i + 1)));
    }

    // Query overlaps all, but stop after 3
    let mut aSelector = TestSelector1D::stopping(TestBnd1D::new(0.0, 11.0), 3);
    let nSel: i32 = aTree.select(&mut aSelector);
    assert_eq!(3, nSel);
    assert_eq!(3, aSelector.results.len());
}

#[test]
fn NCollection_UBTreeTest__Clear() {
    let mut aTree: UBTree<i32, TestBnd1D> = UBTree::default();

    aTree.add(1, TestBnd1D::new(0.0, 5.0));
    aTree.add(2, TestBnd1D::new(3.0, 7.0));

    assert!(!aTree.is_empty());

    aTree.clear();
    assert!(aTree.is_empty());

    let mut aSelector = TestSelector1D::new(TestBnd1D::new(0.0, 10.0));
    assert_eq!(0, aTree.select(&mut aSelector));
}

#[test]
fn NCollection_UBTreeTest__ManyObjects() {
    let mut aTree: UBTree<i32, TestBnd1D> = UBTree::default();

    let n: i32 = 100;
    for i in 0..n {
        aTree.add(i, TestBnd1D::new(f64::from(i), f64::from(i + 1)));
    }

    // Query that overlaps all
    let mut aSelector = TestSelector1D::new(TestBnd1D::new(0.0, f64::from(n + 1)));
    let nSel: i32 = aTree.select(&mut aSelector);
    assert_eq!(n, nSel);
}

#[test]
fn NCollection_UBTreeTest__DeepUnbalancedTree() {
    // Insert sorted non-overlapping intervals to create a deeply unbalanced tree.
    // Each new interval is "out" of all previous bounds, forcing the tree
    // to grow on one side -- producing depth ~ n.
    let mut aTree: UBTree<i32, TestBnd1D> = UBTree::default();

    let n: i32 = 200; // well over the initial stack size of 64
    for i in 0..n {
        let aLo: f64 = f64::from(i) * 10.0;
        aTree.add(i, TestBnd1D::new(aLo, aLo + 1.0));
    }

    // Select all -- should find all n objects despite deep recursion
    let mut aSelector = TestSelector1D::new(TestBnd1D::new(0.0, f64::from(n) * 10.0));
    let nSel: i32 = aTree.select(&mut aSelector);
    assert_eq!(n, nSel);
}

#[test]
fn NCollection_UBTreeTest__MoveConstructor() {
    let mut aSrc: UBTree<i32, TestBnd1D> = UBTree::default();
    aSrc.add(1, TestBnd1D::new(0.0, 2.0));
    aSrc.add(2, TestBnd1D::new(3.0, 5.0));
    aSrc.add(3, TestBnd1D::new(4.0, 6.0));

    let aDst = UBTree::move_from(&mut aSrc);

    // Source is empty
    assert!(aSrc.is_empty());

    // Destination has all objects
    assert!(!aDst.is_empty());
    let mut aSelector = TestSelector1D::new(TestBnd1D::new(0.0, 10.0));
    let nSel: i32 = aDst.select(&mut aSelector);
    assert_eq!(3, nSel);
}

#[test]
fn NCollection_UBTreeTest__MoveAssignment() {
    let mut aSrc: UBTree<i32, TestBnd1D> = UBTree::default();
    aSrc.add(1, TestBnd1D::new(0.0, 2.0));
    aSrc.add(2, TestBnd1D::new(3.0, 5.0));

    let mut aDst: UBTree<i32, TestBnd1D> = UBTree::default();
    aDst.add(10, TestBnd1D::new(100.0, 200.0));

    aDst.move_assign(&mut aSrc);

    assert!(aSrc.is_empty());
    assert!(!aDst.is_empty());

    let mut aSelector = TestSelector1D::new(TestBnd1D::new(0.0, 10.0));
    let nSel: i32 = aDst.select(&mut aSelector);
    assert_eq!(2, nSel);
}

#[test]
fn NCollection_EBTreeTest__AddContainsRemove() {
    let mut aTree: EBTree<i32, TestBnd1D> = EBTree::default();

    assert!(aTree.is_empty());

    aTree.add(1, TestBnd1D::new(0.0, 2.0));
    aTree.add(2, TestBnd1D::new(3.0, 5.0));
    aTree.add(3, TestBnd1D::new(4.0, 6.0));

    assert!(aTree.contains(&1));
    assert!(aTree.contains(&2));
    assert!(aTree.contains(&3));
    assert!(!aTree.contains(&4));

    // Duplicate add should return false
    assert!(!aTree.add(1, TestBnd1D::new(0.0, 2.0)));

    // Remove object 2
    assert!(aTree.remove(&2));
    assert!(!aTree.contains(&2));
    assert!(aTree.contains(&1));
    assert!(aTree.contains(&3));

    // Remove non-existent
    assert!(!aTree.remove(&4));
}

#[test]
fn NCollection_EBTreeTest__RemoveRoot() {
    let mut aTree: EBTree<i32, TestBnd1D> = EBTree::default();

    aTree.add(1, TestBnd1D::new(0.0, 5.0));
    assert!(aTree.contains(&1));

    // Removing the only element (root) should clear the tree
    assert!(aTree.remove(&1));
    assert!(aTree.is_empty());
}

#[test]
fn NCollection_EBTreeTest__SelectAfterRemove() {
    let mut aTree: EBTree<i32, TestBnd1D> = EBTree::default();

    aTree.add(1, TestBnd1D::new(0.0, 2.0));
    aTree.add(2, TestBnd1D::new(3.0, 5.0));
    aTree.add(3, TestBnd1D::new(4.0, 6.0));

    // Remove object 2
    aTree.remove(&2);

    // Select overlapping [3.5, 5.5] -- only object 3 should remain
    let mut aSelector = TestSelector1D::new(TestBnd1D::new(3.5, 5.5));
    let nSel: i32 = aTree.select(&mut aSelector);
    assert_eq!(1, nSel);
    assert_eq!(1, aSelector.results.len());
    assert_eq!(3, aSelector.results[0]);
}

#[test]
fn NCollection_EBTreeTest__FindNode() {
    let mut aTree: EBTree<i32, TestBnd1D> = EBTree::default();

    aTree.add(1, TestBnd1D::new(0.0, 2.0));
    aTree.add(2, TestBnd1D::new(3.0, 5.0));

    let aNode = aTree.find_node(&1).unwrap();
    assert!(aNode.is_leaf());
    assert_eq!(1, *aNode.object());
}

#[test]
fn NCollection_EBTreeTest__RemoveAll() {
    let mut aTree: EBTree<i32, TestBnd1D> = EBTree::default();

    aTree.add(1, TestBnd1D::new(0.0, 2.0));
    aTree.add(2, TestBnd1D::new(3.0, 5.0));
    aTree.add(3, TestBnd1D::new(4.0, 6.0));

    assert!(aTree.remove(&1));
    assert!(aTree.remove(&2));
    assert!(aTree.remove(&3));
    assert!(aTree.is_empty());
}

#[test]
fn NCollection_EBTreeTest__MoveConstructor() {
    let mut aSrc: EBTree<i32, TestBnd1D> = EBTree::default();
    aSrc.add(1, TestBnd1D::new(0.0, 2.0));
    aSrc.add(2, TestBnd1D::new(3.0, 5.0));
    aSrc.add(3, TestBnd1D::new(4.0, 6.0));

    let aDst = EBTree::move_from(&mut aSrc);

    // Source is empty
    assert!(aSrc.is_empty());
    assert!(!aSrc.contains(&1));

    // Destination has all objects
    assert!(!aDst.is_empty());
    assert!(aDst.contains(&1));
    assert!(aDst.contains(&2));
    assert!(aDst.contains(&3));

    let mut aSelector = TestSelector1D::new(TestBnd1D::new(0.0, 10.0));
    let nSel: i32 = aDst.select(&mut aSelector);
    assert_eq!(3, nSel);
}

#[test]
fn NCollection_EBTreeTest__MoveAssignment() {
    let mut aSrc: EBTree<i32, TestBnd1D> = EBTree::default();
    aSrc.add(1, TestBnd1D::new(0.0, 2.0));
    aSrc.add(2, TestBnd1D::new(3.0, 5.0));

    let mut aDst: EBTree<i32, TestBnd1D> = EBTree::default();
    aDst.add(10, TestBnd1D::new(100.0, 200.0));

    aDst.move_assign(&mut aSrc);

    assert!(aSrc.is_empty());
    assert!(!aDst.is_empty());
    assert!(aDst.contains(&1));
    assert!(aDst.contains(&2));
    assert!(!aDst.contains(&10));

    let mut aSelector = TestSelector1D::new(TestBnd1D::new(0.0, 10.0));
    let nSel: i32 = aDst.select(&mut aSelector);
    assert_eq!(2, nSel);
}
