//! OCC23497: preserve the observed kernel failure, per the user's explicit decision.
//! Upstream GTEST_SKIP remains provenance; this is a separate behavioral regression.
#[path = "units_support/mod.rs"]
mod units_support;
use rustycad_tkernel::units::UnitSentence;
use units_support::api;
#[test]
fn occ23497_current_composite_prefix_failure() {
    let mut units = api();
    let mut sentence = UnitSentence::new(units.units(), b"sq.km").unwrap();
    assert!(sentence.is_done().unwrap());
    let failure = sentence.evaluate().err().unwrap();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    assert_eq!(failure.message(), "NCollection_Sequence::Value");
    let failure = units.any_to_si(1.0, b"sq.km").err().unwrap();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    assert_eq!(failure.message(), "NCollection_Sequence::Value");
}
