//! BOPTest_Utilities.pxx: the BRepAlgoAPI_TestBase and BOPAlgo_TestBase fixture methods.

#![allow(dead_code, non_snake_case)]

use super::*;

fn expect_near(actual: f64, expected: f64, tolerance: f64, what: &str) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{what}: {actual} differs from {expected} by more than {tolerance}"
    );
}

/// Validate result properties against expected values (`ValidateResult` of both fixtures).
pub fn ValidateResult(result: &Shape, surface_area: f64, volume: f64, empty: bool) {
    let tolerance = DefaultTolerance();
    if empty {
        assert!(IsEmpty(result, tolerance), "Result should be empty");
        return;
    }
    assert!(!result.is_null(), "Result shape should not be null");
    if surface_area >= 0.0 {
        expect_near(
            GetSurfaceArea(result),
            surface_area,
            5000.0,
            "Surface area mismatch",
        );
    }
    if volume >= 0.0 {
        expect_near(GetVolume(result), volume, tolerance, "Volume mismatch");
    }
}

/// `BRepAlgoAPI_TestBase::PerformCut`.
pub fn PerformCut(object: &Shape, tool: &Shape) -> Shape {
    let cutter = BRepAlgoAPICut::from_shapes(object, tool).unwrap();
    assert!(cutter.is_done(), "BRepAlgoAPI_Cut operation failed");
    cutter.shape().clone()
}

/// `BRepAlgoAPI_TestBase::PerformFuse`.
pub fn PerformFuse(first: &Shape, second: &Shape) -> Shape {
    let fuser = BRepAlgoAPIFuse::from_shapes(first, second).unwrap();
    assert!(fuser.is_done(), "BRepAlgoAPI_Fuse operation failed");
    fuser.shape().clone()
}

/// `BRepAlgoAPI_TestBase::PerformCommon`.
pub fn PerformCommon(first: &Shape, second: &Shape) -> Shape {
    let common = BRepAlgoAPICommon::from_shapes(first, second).unwrap();
    assert!(common.is_done(), "BRepAlgoAPI_Common operation failed");
    common.shape().clone()
}

/// `BOPAlgo_TestBase::PerformDirectBOP`.
pub fn PerformDirectBOP(first: &Shape, second: &Shape, operation: Operation) -> Shape {
    let mut bop = BOPAlgoBOP::new();
    bop.add_argument(first);
    bop.add_tool(second);
    bop.set_operation(operation);
    bop.options_mut().set_fuzzy_value(DefaultFuzzyValue());
    bop.options_mut().set_run_parallel(false);
    bop.set_non_destructive(false);
    bop.perform();
    assert!(!bop.options().has_errors(), "Direct BOP operation failed");
    bop.shape().clone()
}

/// `BOPAlgo_TestBase::PreparePaveFiller`.
pub fn PreparePaveFiller(first: &Shape, second: &Shape) -> BOPAlgoPaveFiller {
    let arguments: List<Shape> = [first.clone(), second.clone()].into_iter().collect();
    let mut filler = BOPAlgoPaveFiller::new();
    filler.set_arguments(&arguments);
    filler.options_mut().set_fuzzy_value(DefaultFuzzyValue());
    filler.options_mut().set_run_parallel(false);
    filler.set_non_destructive(false);
    filler.perform();
    assert!(
        !filler.options().has_errors(),
        "PaveFiller preparation failed"
    );
    filler
}

/// `BOPAlgo_TestBase::PerformBOPWithPaveFiller`.
pub fn PerformBOPWithPaveFiller(filler: &mut BOPAlgoPaveFiller, operation: Operation) -> Shape {
    let arguments: Vec<Shape> = filler.arguments().iter().cloned().collect();
    assert_eq!(arguments.len(), 2, "Wrong number of arguments");
    let mut bop = BOPAlgoBOP::new();
    bop.add_argument(&arguments[0]);
    bop.add_tool(&arguments[1]);
    bop.set_operation(operation);
    bop.options_mut().set_run_parallel(false);
    bop.perform_with_filler(filler);
    assert!(
        !bop.options().has_errors(),
        "BOP operation with PaveFiller failed"
    );
    bop.shape().clone()
}

/// `BOPAlgo_TestBase::PerformTwoStepBOP`.
pub fn PerformTwoStepBOP(first: &Shape, second: &Shape, operation: Operation) -> Shape {
    let mut filler = PreparePaveFiller(first, second);
    PerformBOPWithPaveFiller(&mut filler, operation)
}
