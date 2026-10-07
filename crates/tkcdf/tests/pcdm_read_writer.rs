//! 1:1 translation of src/ApplicationFramework/TKCDF/GTests/PCDM_ReadWriter_Test.cxx.
#![allow(non_snake_case)]
use rustycad_tkcdf::pcdm::PcdmReadWriterBase;
use rustycad_tkernel::tcollection::ExtendedString;

#[test]
fn PCDM_ReadWriterTest__ParseReference() {
    let mut identifier = 0;
    let mut version = 0;
    let mut file_name = ExtendedString::default();
    assert!(PcdmReadWriterBase::parse_reference(
        &ExtendedString::new("12 3 relative/path.ext"),
        &mut identifier,
        &mut version,
        &mut file_name
    ));
    assert_eq!(identifier, 12);
    assert_eq!(version, 3);
    assert_eq!(file_name, ExtendedString::new("relative/path.ext"));
}

#[test]
fn PCDM_ReadWriterTest__ParseReferenceFileNameWithSpaces() {
    let mut identifier = 0;
    let mut version = 0;
    let mut file_name = ExtendedString::default();
    assert!(PcdmReadWriterBase::parse_reference(
        &ExtendedString::new("7 2 folder/file name.ext"),
        &mut identifier,
        &mut version,
        &mut file_name
    ));
    assert_eq!(identifier, 7);
    assert_eq!(version, 2);
    assert_eq!(file_name, ExtendedString::new("folder/file name.ext"));
}

#[test]
fn PCDM_ReadWriterTest__RejectMalformedReferenceWithoutChangingOutputs() {
    let malformed = [
        "",
        "1",
        "1 2",
        "identifier 2 file.ext",
        "1 version file.ext",
        "1 2 ",
    ];
    for reference in malformed {
        let mut identifier = 10;
        let mut version = 20;
        let mut file_name = ExtendedString::new("unchanged");
        assert!(!PcdmReadWriterBase::parse_reference(
            &ExtendedString::new(reference),
            &mut identifier,
            &mut version,
            &mut file_name
        ));
        assert_eq!(identifier, 10);
        assert_eq!(version, 20);
        assert_eq!(file_name, ExtendedString::new("unchanged"));
    }
}
