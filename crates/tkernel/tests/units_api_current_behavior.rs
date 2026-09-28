//! OCC23497: preserve the observed kernel failure, per the user's explicit decision.
//! Upstream GTEST_SKIP remains provenance; this is a separate behavioral regression.
use rustycad_tkernel::units::{UnitSentence, UnitsApi, UnitsDictionary};
#[test]
fn occ23497_current_composite_prefix_failure() {
    let dictionary = UnitsDictionary::builtin().unwrap();
    let mut sentence = UnitSentence::new(&dictionary, b"sq.km").unwrap();
    assert!(sentence.is_done());
    let failure = sentence.evaluate().unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    assert_eq!(failure.message(), "NCollection_Sequence::Value");
    let failure = UnitsApi::new(dictionary)
        .any_to_si(1.0, "sq.km")
        .unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    assert_eq!(failure.message(), "NCollection_Sequence::Value");
}
