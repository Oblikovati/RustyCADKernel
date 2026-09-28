// Copyright (c) 2026 OPEN CASCADE SAS
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

//! Concurrent decimal-conversion regressions with all upstream inputs and iteration counts.
#![allow(non_snake_case, clippy::approx_constant, clippy::excessive_precision)]
use rustycad_tkernel::{ncollection::Array1, osd::Parallel, standard::strtod};
const CASES: [(&str, f64); 15] = [
    ("0.0", 0.0),
    ("1.0", 1.0),
    ("3.141592653589793", 3.141592653589793),
    ("-273.15", -273.15),
    ("1e10", 1e10),
    ("1.23456789e-15", 1.23456789e-15),
    ("123456789.987654321", 123456789.987654321),
    ("1.0e20", 1e20),
    ("1.0e-20", 1e-20),
    ("9.999999999999999e300", 9.999999999999999e300),
    ("5e-324", 5e-324),
    ("2.2250738585072014e-308", 2.2250738585072014e-308),
    ("17976931348623157.0", 17976931348623157.0),
    ("0.000001", 0.000001),
    ("999999999999999.0", 999999999999999.0),
];
#[test]
fn Standard_StrtodTest__SingleThread_Correctness() {
    for (text, expected) in CASES {
        let (actual, end) = strtod(text);
        assert_ne!(end, 0, "{text}");
        assert_eq!(actual, expected, "{text}");
    }
}
#[test]
fn Standard_StrtodTest__Parallel_ConsistentResults() {
    let mut results = Array1::with_size(200);
    results.init(-1.0);
    let text = "3.14159265358979323846264338327950288419716939937510";
    Parallel::for_each_mut(results.as_mut_slice(), |_, slot| *slot = strtod(text).0);
    let reference = strtod(text).0;
    for &actual in &results {
        assert_eq!(actual, reference);
    }
}
#[test]
fn Standard_StrtodTest__Parallel_RepeatedRuns() {
    let text = "2.71828182845904523536028747135266249775724709369995";
    let reference = strtod(text).0;
    for run in 0..10 {
        let mut results = Array1::with_size(100);
        Parallel::for_each_mut(results.as_mut_slice(), |_, slot| *slot = strtod(text).0);
        for (index, actual) in results.iter().enumerate() {
            assert_eq!(*actual, reference, "run {run}, thread {index}");
        }
    }
}
#[test]
fn Standard_StrtodTest__Parallel_ManyDistinctInputs() {
    let mut expected = Array1::with_size(100);
    let mut results = Array1::with_size(100);
    for (index, value) in expected.iter_mut().enumerate() {
        *value = strtod(CASES[index % CASES.len()].0).0;
    }
    Parallel::for_each_mut(results.as_mut_slice(), |index, value| {
        *value = strtod(CASES[index % CASES.len()].0).0
    });
    for index in 0..100 {
        assert_eq!(results[index], expected[index], "index {index}");
    }
}
