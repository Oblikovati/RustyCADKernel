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

//! Direct translation of math_DoubleTab_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case, unused_mut)]
use rustycad_tkmath::math::DoubleTab;

// Tests for constructors
#[test]
fn MathDoubleTabTest__DefaultConstructor() {
    // Test standard constructor with ranges
    let mut aTab = DoubleTab::with_bounds(1, 3, 1, 3).unwrap();

    // Initialize with test values
    aTab.init(5.0);

    // Check all values are set to 5.0
    for anI in 1..=3 {
        for aJ in 1..=3 {
            assert_eq!(*aTab.value(anI, aJ).unwrap(), 5.0);
            assert_eq!(aTab[(anI, aJ)], 5.0); // Test operator()
        }
    }
}

#[test]
fn MathDoubleTabTest__ExternalArrayConstructor() {
    // Create external array
    let mut anArray = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];

    let expected = anArray;

    // Create math_DoubleTab from external array
    let mut aTab = DoubleTab::from_buffer(&mut anArray, 1, 3, 1, 3).unwrap();

    // Check values are correctly accessed
    let mut anIndex: usize = 0;
    for anI in 1..=3 {
        for aJ in 1..=3 {
            assert_eq!(*aTab.value(anI, aJ).unwrap(), expected[anIndex]);
            anIndex += 1;
        }
    }
}

#[test]
fn MathDoubleTabTest__CopyConstructor() {
    // Create original array
    let mut aOriginal = DoubleTab::with_bounds(1, 3, 1, 3).unwrap();
    aOriginal.init(10.0);

    // Set some specific values
    *aOriginal.change_value(1, 1).unwrap() = 1.5;
    *aOriginal.change_value(2, 2).unwrap() = 2.5;
    *aOriginal.change_value(3, 3).unwrap() = 3.5;

    // Create copy
    let mut aCopy = aOriginal.clone();

    // Check values are copied correctly
    assert_eq!(*aCopy.value(1, 1).unwrap(), 1.5);
    assert_eq!(*aCopy.value(2, 2).unwrap(), 2.5);
    assert_eq!(*aCopy.value(3, 3).unwrap(), 3.5);
    assert_eq!(*aCopy.value(1, 2).unwrap(), 10.0); // Check other values too
}

// Tests for basic operations
#[test]
fn MathDoubleTabTest__InitOperation() {
    let mut aTab = DoubleTab::with_bounds(0, 2, 0, 2).unwrap();

    // Initialize with specific value
    aTab.init(7.5);

    // Check all elements are set
    for anI in 0..=2 {
        for aJ in 0..=2 {
            assert_eq!(*aTab.value(anI, aJ).unwrap(), 7.5);
        }
    }
}

#[test]
fn MathDoubleTabTest__ValueAccess() {
    let mut aTab = DoubleTab::with_bounds(1, 2, 1, 2).unwrap();

    // Set values using Value method
    *aTab.change_value(1, 1).unwrap() = 11.0;
    *aTab.change_value(1, 2).unwrap() = 12.0;
    *aTab.change_value(2, 1).unwrap() = 21.0;
    *aTab.change_value(2, 2).unwrap() = 22.0;

    // Check values using operator()
    assert_eq!(aTab[(1, 1)], 11.0);
    assert_eq!(aTab[(1, 2)], 12.0);
    assert_eq!(aTab[(2, 1)], 21.0);
    assert_eq!(aTab[(2, 2)], 22.0);
}

#[test]
fn MathDoubleTabTest__OperatorAccess() {
    let mut aTab = DoubleTab::with_bounds(-1, 1, -1, 1).unwrap();

    // Set values using operator()
    aTab[(-1, -1)] = -11.0;
    aTab[(-1, 0)] = -10.0;
    aTab[(-1, 1)] = -9.0;
    aTab[(0, -1)] = -1.0;
    aTab[(0, 0)] = 0.0;
    aTab[(0, 1)] = 1.0;
    aTab[(1, -1)] = 9.0;
    aTab[(1, 0)] = 10.0;
    aTab[(1, 1)] = 11.0;

    // Check values using Value method
    assert_eq!(*aTab.value(-1, -1).unwrap(), -11.0);
    assert_eq!(*aTab.value(-1, 0).unwrap(), -10.0);
    assert_eq!(*aTab.value(-1, 1).unwrap(), -9.0);
    assert_eq!(*aTab.value(0, -1).unwrap(), -1.0);
    assert_eq!(*aTab.value(0, 0).unwrap(), 0.0);
    assert_eq!(*aTab.value(0, 1).unwrap(), 1.0);
    assert_eq!(*aTab.value(1, -1).unwrap(), 9.0);
    assert_eq!(*aTab.value(1, 0).unwrap(), 10.0);
    assert_eq!(*aTab.value(1, 1).unwrap(), 11.0);
}

#[test]
fn MathDoubleTabTest__CopyMethod() {
    // Create source array
    let mut aSource = DoubleTab::with_bounds(1, 2, 1, 2).unwrap();
    *aSource.change_value(1, 1).unwrap() = 100.0;
    *aSource.change_value(1, 2).unwrap() = 200.0;
    *aSource.change_value(2, 1).unwrap() = 300.0;
    *aSource.change_value(2, 2).unwrap() = 400.0;

    // Create destination array
    let mut aDest = DoubleTab::with_bounds(1, 2, 1, 2).unwrap();
    aDest.init(0.0);

    // Copy from source to destination
    aSource.copy_to(&mut aDest).unwrap();

    // Check values are copied
    assert_eq!(*aDest.value(1, 1).unwrap(), 100.0);
    assert_eq!(*aDest.value(1, 2).unwrap(), 200.0);
    assert_eq!(*aDest.value(2, 1).unwrap(), 300.0);
    assert_eq!(*aDest.value(2, 2).unwrap(), 400.0);
}

// Tests for buffer optimization
#[test]
fn MathDoubleTabTest__SmallArrayOptimization() {
    // Test that small arrays (<=16 elements) use buffer optimization
    // 4x4 = 16 elements, should use buffer
    let mut aSmallTab = DoubleTab::with_bounds(1, 4, 1, 4).unwrap();
    aSmallTab.init(42.0);

    // Set and read values to ensure it works correctly
    *aSmallTab.change_value(2, 3).unwrap() = 123.45;
    assert_eq!(*aSmallTab.value(2, 3).unwrap(), 123.45);
    assert_eq!(*aSmallTab.value(1, 1).unwrap(), 42.0);
}

#[test]
fn MathDoubleTabTest__LargeArrayAllocation() {
    // Test that large arrays (>16 elements) allocate memory
    // 5x5 = 25 elements, should allocate
    let mut aLargeTab = DoubleTab::with_bounds(1, 5, 1, 5).unwrap();
    aLargeTab.init(99.99);

    // Set and read values to ensure it works correctly
    *aLargeTab.change_value(3, 4).unwrap() = 987.65;
    assert_eq!(*aLargeTab.value(3, 4).unwrap(), 987.65);
    assert_eq!(*aLargeTab.value(1, 1).unwrap(), 99.99);
}

// Tests for edge cases
#[test]
fn MathDoubleTabTest__SingleElement() {
    // Test with single element array
    let mut aSingle = DoubleTab::with_bounds(5, 5, 10, 10).unwrap();
    *aSingle.change_value(5, 10).unwrap() = 777.0;

    assert_eq!(*aSingle.value(5, 10).unwrap(), 777.0);
    assert_eq!(aSingle[(5, 10)], 777.0);
}

#[test]
fn MathDoubleTabTest__NegativeIndices() {
    // Test with negative indices
    let mut aNegTab = DoubleTab::with_bounds(-5, -1, -3, -1).unwrap();
    aNegTab.init(-1.0);

    *aNegTab.change_value(-3, -2).unwrap() = -999.0;
    assert_eq!(*aNegTab.value(-3, -2).unwrap(), -999.0);
    assert_eq!(*aNegTab.value(-5, -3).unwrap(), -1.0);
}
