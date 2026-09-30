//! Original UnitsAPI assertions; independent injected unit contexts replace global mutable state.
#![allow(non_snake_case)]
#[path = "units_support/mod.rs"]
mod units_support;
use rustycad_tkernel::units::UnitSentence;
use rustycad_tkernel::units_api::SystemUnits as UnitSystem;
use units_support::api;
#[test]
fn UnitsAPI_Test__BUC60727_AnyToLS_Conversion() {
    let mut units = api();
    units.set_local_system(UnitSystem::Mdtv).unwrap();
    assert_eq!(units.any_to_ls(3.0, b"mm").unwrap(), 3.0);
}
#[test]
fn UnitsAPI_Test__AnyToAny_UnknownUnit() {
    let mut units = api();
    assert_eq!(units.any_to_any(1.0, b"unknown_unit", b"mm").unwrap(), 1.0);
    assert_eq!(units.any_to_any(1.0, b"mm", b"unknown_unit").unwrap(), 1.0);
}
#[test]
fn UnitsAPI_Test__FClassesBug_30800_PoiseConversion() {
    let mut units = api();
    assert!((units.any_to_si(1.0, b"Po").unwrap() - 0.1).abs() <= 1e-12);
    assert!((units.any_to_any(1.0, b"Po", b"kg/cm/s").unwrap() - 0.001).abs() <= 1e-12);
}
#[test]
fn UnitsAPI_Test__FClassesBug_11568_2_MinuteAlias() {
    assert_eq!(api().any_to_si(1.0, b"mn").unwrap(), 60.0);
}
#[test]
fn UnitsAPI_Test__FClassesBug_11568_4_MinuteSymbol() {
    assert_eq!(api().any_to_si(1.0, b"min").unwrap(), 60.0);
}
fn parsed_word(expression: &str) -> String {
    let mut units = api();
    units.set_local_system(UnitSystem::Si).unwrap();
    let mut sentence = UnitSentence::new(units.units(), expression.as_bytes()).unwrap();
    assert!(sentence.is_done().unwrap());
    let token = sentence.evaluate().unwrap();
    assert!(token.is_some());
    String::from_utf8(token.get().unwrap().word().as_bytes().to_vec()).unwrap()
}
#[test]
fn UnitsAPI_Test__FClassesBug_11568_1_MinuteAliasParsing() {
    assert_eq!(parsed_word("mn"), "min");
}
#[test]
fn UnitsAPI_Test__FClassesBug_11568_3_MinuteSymbolParsing() {
    assert_eq!(parsed_word("min"), "min");
}
#[test]
fn UnitsAPI_Test__FClassesBug_9848_1_InvalidUnitSentence() {
    let mut units = api();
    units.set_local_system(UnitSystem::Si).unwrap();
    let sentence = UnitSentence::new(units.units(), b"tonne/(mm*s**2)").unwrap();
    assert!(!sentence.is_done().unwrap());
}
#[test]
fn UnitsAPI_Test__FClassesBug_9848_2_ValidCompositeUnitSentence() {
    assert_eq!(parsed_word("ton/(m*s**2)"), "(ton)/((m)*((s)**(2)))");
}
