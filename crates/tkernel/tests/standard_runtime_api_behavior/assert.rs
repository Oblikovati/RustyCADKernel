//! Standard_Assert report, raise and static assert.
use rustycad_tkernel::standard::assert::{
    StandardStaticAssert, standard_assert_do_nothing, standard_assert_raise, standard_assert_report,
};

#[test]
fn report_writes_the_posix_debug_text() {
    let mut stream: Vec<u8> = Vec::new();
    assert!(standard_assert_report(&mut stream, "a.cxx", 12, "x > 0", "positive").unwrap());
    assert_eq!(
        "ERROR: statement 'x > 0' is not TRUE!\n\nFile: 'a.cxx'\nLine: 12\nDescription: positive\n",
        String::from_utf8(stream).unwrap()
    );
}
#[test]
fn report_omits_an_empty_description() {
    let mut stream: Vec<u8> = Vec::new();
    assert!(standard_assert_report(&mut stream, "b.cxx", -1, "ok", "").unwrap());
    assert_eq!(
        "ERROR: statement 'ok' is not TRUE!\n\nFile: 'b.cxx'\nLine: -1\n",
        String::from_utf8(stream).unwrap()
    );
}
#[test]
fn raise_builds_the_program_error_message() {
    assert!(standard_assert_raise(true, "f.cxx", "x", "never").is_ok());
    let failure = standard_assert_raise(false, "f.cxx", "x == 1", "bad x").unwrap_err();
    assert_eq!("Standard_ProgramError", failure.exception_type());
    assert_eq!(
        "*** ERROR: ASSERT in file 'f.cxx': \nbad x (x == 1)",
        failure.message()
    );
}
#[test]
fn do_nothing_and_static_assert_compile_to_nothing() {
    standard_assert_do_nothing();
    StandardStaticAssert::<{ 1 + 1 == 2 }>::assert_ok();
}
