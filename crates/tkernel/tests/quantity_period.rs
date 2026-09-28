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

//! 1:1 translation of TKernel/GTests/Quantity_Period_Test.cxx.
// Unused out-parameters and original test/local names are retained for review.
#![allow(non_snake_case, unused_variables)]
use rustycad_tkernel::quantity::Period;
#[test]
fn Quantity_PeriodTest__BasicConstruction() {
    // Constructor with days, hours, minutes, seconds, milliseconds, microseconds
    let aPeriod1 = Period::from_components(1, 2, 3, 4, 5, 6).unwrap();

    let (dd, hh, mn, ss, mis, mics) = aPeriod1.components();

    assert_eq!(1, dd);
    assert_eq!(2, hh);
    assert_eq!(3, mn);
    assert_eq!(4, ss);
    assert_eq!(5, mis);
    assert_eq!(6, mics);

    // Constructor with seconds and microseconds
    let aPeriod2 = Period::new(3600, 500000).unwrap(); // 1 hour, 500000 microseconds

    let (ss2, mics2) = aPeriod2.values();

    assert_eq!(3600, ss2); // SECONDS_PER_HOUR
    assert_eq!(500000, mics2);
}

#[test]
fn Quantity_PeriodTest__ConstexprComparisons() {
    let aPeriod1 = Period::from_components(0, 0, 0, 100, 0, 0).unwrap();
    let aPeriod2 = Period::from_components(0, 0, 0, 100, 0, 0).unwrap();
    let aPeriod3 = Period::from_components(0, 0, 0, 200, 0, 0).unwrap();

    // Test IsEqual (constexpr)
    assert!(aPeriod1.is_equal(aPeriod2));
    assert!(!(aPeriod1.is_equal(aPeriod3)));

    // Test IsShorter (constexpr)
    assert!(aPeriod1.is_shorter(aPeriod3));
    assert!(!(aPeriod3.is_shorter(aPeriod1)));
    assert!(!(aPeriod1.is_shorter(aPeriod2)));

    // Test IsLonger (constexpr)
    assert!(aPeriod3.is_longer(aPeriod1));
    assert!(!(aPeriod1.is_longer(aPeriod3)));
    assert!(!(aPeriod1.is_longer(aPeriod2)));
}

#[test]
fn Quantity_PeriodTest__Validation() {
    // Valid periods
    assert!(Period::is_valid_components(1, 2, 3, 4, 5, 6));
    assert!(Period::is_valid_components(0, 0, 0, 0, 0, 0));

    // Invalid periods (negative values)
    assert!(!(Period::is_valid_components(-1, 0, 0, 0, 0, 0)));
    assert!(!(Period::is_valid_components(0, -1, 0, 0, 0, 0)));
    assert!(!(Period::is_valid_components(0, 0, -1, 0, 0, 0)));
    assert!(!(Period::is_valid_components(0, 0, 0, -1, 0, 0)));
    assert!(!(Period::is_valid_components(0, 0, 0, 0, -1, 0)));
    assert!(!(Period::is_valid_components(0, 0, 0, 0, 0, -1)));

    // Validation with seconds and microseconds only
    assert!(Period::is_valid(100, 500));
    assert!(!(Period::is_valid(-1, 500)));
    assert!(!(Period::is_valid(100, -1)));
}

#[test]
fn Quantity_PeriodTest__SetValuesRoundTrip() {
    let mut aPeriod = Period::new(0, 0).unwrap(); // Initialize with zero period
    aPeriod.set_components(2, 3, 4, 5, 6, 7).unwrap();

    let (dd, hh, mn, ss, mis, mics) = aPeriod.components();

    assert_eq!(2, dd);
    assert_eq!(3, hh);
    assert_eq!(4, mn);
    assert_eq!(5, ss);
    assert_eq!(6, mis);
    assert_eq!(7, mics);
}

#[test]
fn Quantity_PeriodTest__FormatConversion() {
    // 1 day should equal 86400 seconds (SECONDS_PER_DAY)
    let aPeriod1 = Period::from_components(1, 0, 0, 0, 0, 0).unwrap();

    let (ss, mics) = aPeriod1.values();

    assert_eq!(86400, ss); // SECONDS_PER_DAY = 24 * 3600

    // 1 hour should equal 3600 seconds (SECONDS_PER_HOUR)
    let aPeriod2 = Period::from_components(0, 1, 0, 0, 0, 0).unwrap();
    let (ss, mics) = aPeriod2.values();

    assert_eq!(3600, ss); // SECONDS_PER_HOUR

    // 1 minute should equal 60 seconds (SECONDS_PER_MINUTE)
    let aPeriod3 = Period::from_components(0, 0, 1, 0, 0, 0).unwrap();
    let (ss, mics) = aPeriod3.values();

    assert_eq!(60, ss); // SECONDS_PER_MINUTE
}

#[test]
fn Quantity_PeriodTest__MillisecondConversion() {
    // 1 millisecond = 1000 microseconds (USECS_PER_MSEC)
    let aPeriod = Period::from_components(0, 0, 0, 0, 1, 0).unwrap();

    let (ss, mics) = aPeriod.values();

    assert_eq!(0, ss);
    assert_eq!(1000, mics); // USECS_PER_MSEC

    // Mixed milliseconds and microseconds
    let aPeriod2 = Period::from_components(0, 0, 0, 0, 5, 250).unwrap();
    let (ss, mics) = aPeriod2.values();

    assert_eq!(0, ss);
    assert_eq!(5250, mics); // 5 * 1000 + 250
}

#[test]
fn Quantity_PeriodTest__MicrosecondOverflow() {
    // Create period with > 1,000,000 microseconds
    let mut aPeriod = Period::new(0, 0).unwrap(); // Initialize with zero period
    aPeriod.set_values(0, 1500000).unwrap(); // 1.5 seconds in microseconds

    let (ss, mics) = aPeriod.values();

    assert_eq!(1, ss); // 1 second
    assert_eq!(500000, mics); // 0.5 second = 500000 microseconds
}

#[test]
fn Quantity_PeriodTest__AddPeriods() {
    let aPeriod1 = Period::from_components(0, 1, 0, 0, 0, 0).unwrap(); // 1 hour
    let aPeriod2 = Period::from_components(0, 2, 0, 0, 0, 0).unwrap(); // 2 hours

    let aResult = aPeriod1.add(aPeriod2).unwrap();

    let (ss, mics) = aResult.values();

    assert_eq!(10800, ss); // 3 hours = 3 * 3600 = 10800 seconds
    assert_eq!(0, mics);
}

#[test]
fn Quantity_PeriodTest__AddWithMicrosecondOverflow() {
    let aPeriod1 = Period::from_components(0, 0, 0, 0, 0, 600000).unwrap(); // 0.6 seconds
    let aPeriod2 = Period::from_components(0, 0, 0, 0, 0, 600000).unwrap(); // 0.6 seconds

    let aResult = aPeriod1.add(aPeriod2).unwrap();

    let (ss, mics) = aResult.values();

    assert_eq!(1, ss); // Should overflow to 1 second
    assert_eq!(200000, mics); // Remaining 0.2 seconds
}

#[test]
fn Quantity_PeriodTest__SubtractPeriods() {
    let aPeriod1 = Period::from_components(0, 3, 0, 0, 0, 0).unwrap(); // 3 hours
    let aPeriod2 = Period::from_components(0, 1, 0, 0, 0, 0).unwrap(); // 1 hour

    let aResult = aPeriod1.subtract(aPeriod2).unwrap();

    let (ss, mics) = aResult.values();

    assert_eq!(7200, ss); // 2 hours = 2 * 3600
    assert_eq!(0, mics);
}

#[test]
fn Quantity_PeriodTest__SubtractWithMicrosecondUnderflow() {
    let aPeriod1 = Period::from_components(0, 0, 0, 1, 0, 200000).unwrap(); // 1.2 seconds
    let aPeriod2 = Period::from_components(0, 0, 0, 0, 0, 500000).unwrap(); // 0.5 seconds

    let aResult = aPeriod1.subtract(aPeriod2).unwrap();

    let (ss, mics) = aResult.values();

    assert_eq!(0, ss);
    assert_eq!(700000, mics); // 1.2 - 0.5 = 0.7 seconds
}

#[test]
fn Quantity_PeriodTest__SubtractNegative() {
    let aPeriod1 = Period::from_components(0, 1, 0, 0, 0, 0).unwrap(); // 1 hour
    let aPeriod2 = Period::from_components(0, 3, 0, 0, 0, 0).unwrap(); // 3 hours

    let aResult = aPeriod1.subtract(aPeriod2).unwrap();

    let (ss, mics) = aResult.values();

    // Result should be absolute value (2 hours)
    assert_eq!(7200, ss); // 2 * 3600
    assert_eq!(0, mics);
}

#[test]
fn Quantity_PeriodTest__ComplexCalculations() {
    // 1 day, 2 hours, 30 minutes, 45 seconds, 500 milliseconds, 250 microseconds
    let aPeriod = Period::from_components(1, 2, 30, 45, 500, 250).unwrap();

    let (ss, mics) = aPeriod.values();

    // Calculate expected seconds:
    // 1 day = 86400 seconds
    // 2 hours = 7200 seconds
    // 30 minutes = 1800 seconds
    // 45 seconds = 45 seconds
    // Total = 95445 seconds
    assert_eq!(95445, ss);

    // Calculate expected microseconds:
    // 500 milliseconds = 500000 microseconds
    // 250 microseconds = 250 microseconds
    // Total = 500250 microseconds
    assert_eq!(500250, mics);
}

#[test]
fn Quantity_PeriodTest__ComponentExtraction() {
    let aPeriod = Period::from_components(2, 3, 45, 30, 123, 456).unwrap();

    let (dd, hh, mn, ss, mis, mics) = aPeriod.components();

    assert_eq!(2, dd);
    assert_eq!(3, hh);
    assert_eq!(45, mn);
    assert_eq!(30, ss);
    assert_eq!(123, mis);
    assert_eq!(456, mics);
}

#[test]
fn Quantity_PeriodTest__ZeroPeriod() {
    let aPeriod = Period::from_components(0, 0, 0, 0, 0, 0).unwrap();

    let (dd, hh, mn, ss, mis, mics) = aPeriod.components();

    assert_eq!(0, dd);
    assert_eq!(0, hh);
    assert_eq!(0, mn);
    assert_eq!(0, ss);
    assert_eq!(0, mis);
    assert_eq!(0, mics);

    // Zero period should be equal to itself
    let aZero2 = Period::from_components(0, 0, 0, 0, 0, 0).unwrap();
    assert!(aPeriod.is_equal(aZero2));
}

#[test]
fn Quantity_PeriodTest__LargeValues() {
    // 100 days
    let aPeriod = Period::from_components(100, 0, 0, 0, 0, 0).unwrap();

    let (ss, mics) = aPeriod.values();

    assert_eq!(8640000, ss); // 100 * 86400
    assert_eq!(0, mics);
}

#[test]
fn Quantity_PeriodTest__TimeConstantValues() {
    // Test that 24 hours = 1 day
    let aPeriod24h = Period::from_components(0, 24, 0, 0, 0, 0).unwrap();
    let aPeriod1d = Period::from_components(1, 0, 0, 0, 0, 0).unwrap();

    let (ss1, mics1) = aPeriod24h.values();
    let (ss2, mics2) = aPeriod1d.values();

    assert_eq!(ss1, ss2); // Should both be 86400 seconds

    // Test that 60 seconds = 1 minute
    let aPeriod60s = Period::from_components(0, 0, 0, 60, 0, 0).unwrap();
    let aPeriod1m = Period::from_components(0, 0, 1, 0, 0, 0).unwrap();

    let (ss1, mics1) = aPeriod60s.values();
    let (ss2, mics2) = aPeriod1m.values();

    assert_eq!(ss1, ss2); // Should both be 60 seconds

    // Test that 60 minutes = 1 hour
    let aPeriod60m = Period::from_components(0, 0, 60, 0, 0, 0).unwrap();
    let aPeriod1h = Period::from_components(0, 1, 0, 0, 0, 0).unwrap();

    let (ss1, mics1) = aPeriod60m.values();
    let (ss2, mics2) = aPeriod1h.values();

    assert_eq!(ss1, ss2); // Should both be 3600 seconds
}

#[test]
fn Quantity_PeriodTest__MillisecondToSecondConversion() {
    let aPeriod = Period::from_components(0, 0, 0, 0, 1000, 0).unwrap();

    let (ss, mics) = aPeriod.values();

    assert_eq!(1, ss); // Should overflow to 1 second
    assert_eq!(0, mics); // No remaining microseconds
}

#[test]
fn Quantity_PeriodTest__SubtractLargeMicrosecondUnderflow() {
    let aPeriod1 = Period::from_components(0, 0, 0, 10, 0, 200000).unwrap(); // 10.2 seconds
    let aPeriod2 = Period::from_components(0, 0, 0, 0, 0, 2700000).unwrap(); // 2.7 seconds

    let aResult = aPeriod1.subtract(aPeriod2).unwrap();

    let (ss, mics) = aResult.values();

    assert_eq!(7, ss); // 10.2 - 2.7 = 7.5 seconds
    assert_eq!(500000, mics);
}

#[test]
fn Quantity_PeriodTest__SubtractNegativeWithMicroseconds() {
    let aPeriod1 = Period::from_components(0, 0, 0, 5, 0, 300000).unwrap(); // 5.3 seconds
    let aPeriod2 = Period::from_components(0, 0, 0, 8, 0, 100000).unwrap(); // 8.1 seconds

    let aResult = aPeriod1.subtract(aPeriod2).unwrap();

    let (ss, mics) = aResult.values();

    // Absolute value: 8.1 - 5.3 = 2.8 seconds
    assert_eq!(2, ss);
    assert_eq!(800000, mics);
}
