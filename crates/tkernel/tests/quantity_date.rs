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

//! 1:1 translation of TKernel/GTests/Quantity_Date_Test.cxx.
// Unused out-parameters and original test/local names are retained for review.
#![allow(non_snake_case, unused_variables)]
use rustycad_tkernel::quantity::{Date, Period};
#[test]
fn Quantity_DateTest__BasicConstruction() {
    // Default constructor creates January 1, 1979, 00:00:00
    let aDate1 = Date::default();

    let (mm, dd, yy, hh, mn, ss, mis, mics) = aDate1.values();

    assert_eq!(1, mm); // January
    assert_eq!(1, dd); // 1st
    assert_eq!(1979, yy); // 1979
    assert_eq!(0, hh); // 00 hours
    assert_eq!(0, mn); // 00 minutes
    assert_eq!(0, ss); // 00 seconds

    // Parameterized constructor
    let aDate2 = Date::new(6, 15, 2025, 14, 30, 45, 123, 456).unwrap();
    let (mm, dd, yy, hh, mn, ss, mis, mics) = aDate2.values();

    assert_eq!(6, mm);
    assert_eq!(15, dd);
    assert_eq!(2025, yy);
    assert_eq!(14, hh);
    assert_eq!(30, mn);
    assert_eq!(45, ss);
    assert_eq!(123, mis);
    assert_eq!(456, mics);
}

#[test]
fn Quantity_DateTest__ConstexprComparisons() {
    let aDate1 = Date::new(1, 1, 2020, 0, 0, 0, 0, 0).unwrap();
    let aDate2 = Date::new(1, 1, 2020, 0, 0, 0, 0, 0).unwrap();
    let aDate3 = Date::new(1, 2, 2020, 0, 0, 0, 0, 0).unwrap();

    // Test IsEqual (constexpr)
    assert!(aDate1.is_equal(aDate2));
    assert!(!(aDate1.is_equal(aDate3)));

    // Test IsEarlier (constexpr)
    assert!(aDate1.is_earlier(aDate3));
    assert!(!(aDate3.is_earlier(aDate1)));
    assert!(!(aDate1.is_earlier(aDate2)));

    // Test IsLater (constexpr)
    assert!(aDate3.is_later(aDate1));
    assert!(!(aDate1.is_later(aDate3)));
    assert!(!(aDate1.is_later(aDate2)));
}

#[test]
fn Quantity_DateTest__LeapYearDetection() {
    // Leap years
    assert!(Date::is_leap(2000)); // Divisible by 400
    assert!(Date::is_leap(2004)); // Divisible by 4, not by 100
    assert!(Date::is_leap(2020));

    // Non-leap years
    assert!(!(Date::is_leap(1900))); // Divisible by 100 but not 400
    assert!(!(Date::is_leap(2001))); // Not divisible by 4
    assert!(!(Date::is_leap(2100))); // Divisible by 100 but not 400
}

#[test]
fn Quantity_DateTest__ValidationWithLeapYear() {
    // Valid date in leap year (February 29)
    assert!(Date::is_valid(2, 29, 2020, 0, 0, 0, 0, 0));

    // Invalid date in non-leap year (February 29)
    assert!(!(Date::is_valid(2, 29, 2019, 0, 0, 0, 0, 0)));

    // Valid February 28 in any year
    assert!(Date::is_valid(2, 28, 2019, 0, 0, 0, 0, 0));
    assert!(Date::is_valid(2, 28, 2020, 0, 0, 0, 0, 0));
}

#[test]
fn Quantity_DateTest__MonthBoundaries() {
    // January has 31 days
    assert!(Date::is_valid(1, 31, 2020, 0, 0, 0, 0, 0));
    assert!(!(Date::is_valid(1, 32, 2020, 0, 0, 0, 0, 0)));

    // April has 30 days
    assert!(Date::is_valid(4, 30, 2020, 0, 0, 0, 0, 0));
    assert!(!(Date::is_valid(4, 31, 2020, 0, 0, 0, 0, 0)));

    // December has 31 days
    assert!(Date::is_valid(12, 31, 2020, 0, 0, 0, 0, 0));
    assert!(!(Date::is_valid(12, 32, 2020, 0, 0, 0, 0, 0)));
}

#[test]
fn Quantity_DateTest__TimeValidation() {
    // Valid times
    assert!(Date::is_valid(1, 1, 2020, 23, 59, 59, 999, 999));
    assert!(Date::is_valid(1, 1, 2020, 0, 0, 0, 0, 0));

    // Invalid hours
    assert!(!(Date::is_valid(1, 1, 2020, 24, 0, 0, 0, 0)));
    assert!(!(Date::is_valid(1, 1, 2020, -1, 0, 0, 0, 0)));

    // Invalid minutes
    assert!(!(Date::is_valid(1, 1, 2020, 0, 60, 0, 0, 0)));
    assert!(!(Date::is_valid(1, 1, 2020, 0, -1, 0, 0, 0)));

    // Invalid seconds
    assert!(!(Date::is_valid(1, 1, 2020, 0, 0, 60, 0, 0)));
    assert!(!(Date::is_valid(1, 1, 2020, 0, 0, -1, 0, 0)));

    // Invalid milliseconds
    assert!(!(Date::is_valid(1, 1, 2020, 0, 0, 0, 1000, 0)));
    assert!(!(Date::is_valid(1, 1, 2020, 0, 0, 0, -1, 0)));

    // Invalid microseconds
    assert!(!(Date::is_valid(1, 1, 2020, 0, 0, 0, 0, 1000)));
    assert!(!(Date::is_valid(1, 1, 2020, 0, 0, 0, 0, -1)));
}

#[test]
fn Quantity_DateTest__SetValuesRoundTrip() {
    let mut aDate = Date::default();
    aDate.set_values(3, 15, 2021, 10, 25, 30, 500, 750).unwrap();

    let (mm, dd, yy, hh, mn, ss, mis, mics) = aDate.values();

    assert_eq!(3, mm);
    assert_eq!(15, dd);
    assert_eq!(2021, yy);
    assert_eq!(10, hh);
    assert_eq!(25, mn);
    assert_eq!(30, ss);
    assert_eq!(500, mis);
    assert_eq!(750, mics);
}

#[test]
fn Quantity_DateTest__IndividualGetters() {
    // Test with a simple date first: January 2, 1979
    let aDate1 = Date::new(1, 2, 1979, 0, 0, 0, 0, 0).unwrap();
    assert_eq!(1, aDate1.month());
    assert_eq!(2, aDate1.day());
    assert_eq!(1979, aDate1.year());

    // Test with July 20, 2024 at midnight
    let aDate2 = Date::new(7, 20, 2024, 0, 0, 0, 0, 0).unwrap();
    assert_eq!(7, aDate2.month());
    assert_eq!(20, aDate2.day());
    assert_eq!(2024, aDate2.year());

    // Test with July 20, 2024 with time components
    let aDate3 = Date::new(7, 20, 2024, 15, 45, 30, 123, 456).unwrap();
    assert_eq!(7, aDate3.month());
    assert_eq!(20, aDate3.day());
    assert_eq!(2024, aDate3.year());
    assert_eq!(15, aDate3.hour());
    assert_eq!(45, aDate3.minute());
    assert_eq!(30, aDate3.second());
    assert_eq!(123, aDate3.millisecond());
    assert_eq!(456, aDate3.microsecond());
}

#[test]
fn Quantity_DateTest__DateDifference() {
    let aDate1 = Date::new(1, 1, 2020, 0, 0, 0, 0, 0).unwrap();
    let aDate2 = Date::new(1, 1, 2020, 1, 0, 0, 0, 0).unwrap(); // 1 hour later

    let aPeriod = aDate2.difference(aDate1);

    let (ss, mics) = aPeriod.values();

    assert_eq!(3600, ss); // 1 hour = 3600 seconds (SECONDS_PER_HOUR)
    assert_eq!(0, mics);
}

#[test]
fn Quantity_DateTest__AddPeriod() {
    let aDate = Date::new(1, 1, 2020, 0, 0, 0, 0, 0).unwrap();
    let aPeriod = Period::from_components(1, 0, 0, 0, 0, 0).unwrap(); // 1 day

    let aNewDate = aDate.add(aPeriod).unwrap();

    let (mm, dd, yy, hh, mn, ss, mis, mics) = aNewDate.values();

    assert_eq!(1, mm);
    assert_eq!(2, dd); // Next day
    assert_eq!(2020, yy);
}

#[test]
fn Quantity_DateTest__SubtractPeriod() {
    let aDate = Date::new(1, 2, 2020, 0, 0, 0, 0, 0).unwrap(); // January 2
    let aPeriod = Period::from_components(1, 0, 0, 0, 0, 0).unwrap(); // 1 day

    let aNewDate = aDate.subtract(aPeriod).unwrap();

    let (mm, dd, yy, hh, mn, ss, mis, mics) = aNewDate.values();

    assert_eq!(1, mm);
    assert_eq!(1, dd); // Previous day
    assert_eq!(2020, yy);
}

#[test]
fn Quantity_DateTest__YearBoundary() {
    let aDate = Date::new(12, 31, 2020, 23, 59, 59, 0, 0).unwrap();
    let aPeriod = Period::from_components(0, 0, 0, 1, 0, 0).unwrap(); // 1 second

    let aNewDate = aDate.add(aPeriod).unwrap();

    let (mm, dd, yy, hh, mn, ss, mis, mics) = aNewDate.values();

    assert_eq!(1, mm); // January
    assert_eq!(1, dd); // 1st
    assert_eq!(2021, yy); // Next year
    assert_eq!(0, hh);
    assert_eq!(0, mn);
    assert_eq!(0, ss);
}

#[test]
fn Quantity_DateTest__LeapYearBoundary() {
    // Leap year: Feb 28 + 1 day = Feb 29
    let aDate1 = Date::new(2, 28, 2020, 0, 0, 0, 0, 0).unwrap();
    let aPeriod1 = Period::from_components(1, 0, 0, 0, 0, 0).unwrap();
    let aNewDate1 = aDate1.add(aPeriod1).unwrap();

    let (mm, dd, yy, hh, mn, ss, mis, mics) = aNewDate1.values();

    assert_eq!(2, mm);
    assert_eq!(29, dd); // Feb 29 exists in 2020
    assert_eq!(2020, yy);

    // Non-leap year: Feb 28 + 1 day = Mar 1
    let aDate2 = Date::new(2, 28, 2021, 0, 0, 0, 0, 0).unwrap();
    let aPeriod2 = Period::from_components(1, 0, 0, 0, 0, 0).unwrap();
    let aNewDate2 = aDate2.add(aPeriod2).unwrap();

    let (mm, dd, yy, hh, mn, ss, mis, mics) = aNewDate2.values();

    assert_eq!(3, mm); // March
    assert_eq!(1, dd); // 1st
    assert_eq!(2021, yy);
}

#[test]
fn Quantity_DateTest__MicrosecondOverflow() {
    let aDate = Date::new(1, 1, 2020, 0, 0, 0, 999, 999).unwrap();
    let aPeriod = Period::from_components(0, 0, 0, 0, 0, 1).unwrap(); // 1 microsecond

    let aNewDate = aDate.add(aPeriod).unwrap();

    let (mm, dd, yy, hh, mn, ss, mis, mics) = aNewDate.values();

    assert_eq!(1, mm);
    assert_eq!(1, dd);
    assert_eq!(2020, yy);
    assert_eq!(0, hh);
    assert_eq!(0, mn);
    assert_eq!(1, ss); // Should overflow to next second
    assert_eq!(0, mis);
    assert_eq!(0, mics);
}

#[test]
fn Quantity_DateTest__SpecificDateCalculations() {
    // Test that 24 hours = 1 day (SECONDS_PER_DAY = 86400)
    let aDate1 = Date::new(1, 1, 2020, 0, 0, 0, 0, 0).unwrap();
    let aPeriod24h = Period::from_components(0, 24, 0, 0, 0, 0).unwrap(); // 24 hours

    let aDate2 = aDate1.add(aPeriod24h).unwrap();

    let (mm, dd, yy, hh, mn, ss, mis, mics) = aDate2.values();

    assert_eq!(1, mm);
    assert_eq!(2, dd); // Next day
    assert_eq!(2020, yy);
    assert_eq!(0, hh);
}

#[test]
fn Quantity_DateTest__MinimumDate() {
    assert!(Date::is_valid(1, 1, 1979, 0, 0, 0, 0, 0));
    assert!(!(Date::is_valid(12, 31, 1978, 23, 59, 59, 999, 999)));
}

#[test]
fn Quantity_DateTest__MultipleLeapYearChecks() {
    for year in 2000..=2024 {
        for month in 1..=12 {
            let max_day = if month == 2 {
                if Date::is_leap(year) { 29 } else { 28 }
            } else if month == 4 || month == 6 || month == 9 || month == 11 {
                30
            } else {
                31
            };
            assert!(Date::is_valid(month, max_day, year, 0, 0, 0, 0, 0));
            assert!(!Date::is_valid(month, max_day + 1, year, 0, 0, 0, 0, 0));
        }
    }
}

#[test]
fn Quantity_DateTest__DifferenceFromEpoch() {
    let aEpoch = Date::default(); // Defaults to Jan 1, 1979 00:00
    let aDate = Date::new(1, 2, 1979, 0, 0, 0, 0, 0).unwrap(); // Jan 2, 1979

    let aPeriod = aEpoch.difference(aDate);

    let (ss, mics) = aPeriod.values();

    assert_eq!(86400, ss); // 1 day = 86400 seconds
    assert_eq!(0, mics);
}

#[test]
fn Quantity_DateTest__DifferenceWithMicrosecondUnderflow() {
    let aDate1 = Date::new(1, 1, 2020, 0, 0, 1, 200, 0).unwrap(); // 1.2 seconds
    let aDate2 = Date::new(1, 1, 2020, 0, 0, 0, 500, 0).unwrap(); // 0.5 seconds

    let aPeriod = aDate1.difference(aDate2);

    let (ss, mics) = aPeriod.values();

    assert_eq!(0, ss);
    assert_eq!(700000, mics); // 1.2 - 0.5 = 0.7 seconds = 700000 microseconds
}

#[test]
fn Quantity_DateTest__DifferenceReversed() {
    let aDate1 = Date::new(1, 1, 2020, 1, 0, 0, 0, 0).unwrap(); // 1 hour
    let aDate2 = Date::new(1, 1, 2020, 3, 0, 0, 0, 0).unwrap(); // 3 hours

    // Difference should be absolute value
    let aPeriod = aDate1.difference(aDate2);

    let (ss, mics) = aPeriod.values();

    assert_eq!(7200, ss); // 2 hours = 7200 seconds
    assert_eq!(0, mics);
}
