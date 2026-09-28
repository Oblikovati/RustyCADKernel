//! Conversion context, byte encoding, temperature shifts and evaluator mutation.
#[path = "units_support/mod.rs"]
mod units_support;
use rustycad_tkernel::units::UnitSentence;
use rustycad_tkernel::units_api::SystemUnits as UnitSystem;
use units_support::api;
#[test]
fn isolated_context_selection_and_default_preserve_native_local_units() {
    let mut units = api();
    assert_eq!(units.local_system(), UnitSystem::Si);
    assert_eq!(units.any_to_ls(2.0, b"m").unwrap(), 2.0);
    units.set_local_system(UnitSystem::Mdtv).unwrap();
    assert_eq!(units.any_to_ls(2.0, b"m").unwrap(), 2000.0);
    units.set_local_system(UnitSystem::Default).unwrap();
    assert_eq!(units.local_system(), UnitSystem::Mdtv);
    assert_eq!(api().any_to_ls(2.0, b"m").unwrap(), 2.0);
    units.set_local_system(UnitSystem::Si).unwrap();
    assert_eq!(units.any_to_ls(2.0, b"m").unwrap(), 2.0);
}
#[test]
fn shifted_temperature_conversion_and_legacy_byte_symbols() {
    let mut units = api();
    assert_eq!(units.any_to_si(0.0, b"deg.C").unwrap(), 273.15);
    assert_eq!(units.any_from_si(273.15, b"deg.C").unwrap(), 0.0);
    // The native Fahrenheit offset is 1379/3, yielding this small nonzero Celsius result.
    assert_eq!(
        units
            .any_to_any(32.0, b"deg.F", b"deg.C")
            .unwrap()
            .to_bits(),
        13789555019046977536
    );
    assert_eq!(units.any_to_si(1.0, b"mm\xb2").unwrap(), 1e-6);
    assert_eq!(units.any_to_any(2.0, b"m", b"s").unwrap(), 2.0);
    assert_eq!(units.any_to_si(5.0, b"unrecognized").unwrap(), 0.0);
    assert_eq!(units.any_from_si(5.0, b"unrecognized").unwrap(), 0.0);
}
#[test]
fn dictionary_aliases_dimensions_and_destructive_evaluation_are_preserved() {
    let units = api();
    let context = units.units();
    let dictionary = context.dictionary_of_units(false).unwrap();
    let quantities = dictionary.get().unwrap().sequence();
    let names: Vec<Vec<u8>> = quantities.get().unwrap().sequence(|items| {
        items
            .iter()
            .map(|quantity| quantity.get().unwrap().name().as_bytes().to_vec())
            .collect()
    });
    assert_eq!(
        names.first().map(Vec::as_slice),
        Some(b"DIMENSIONLESS".as_slice())
    );
    assert_eq!(
        names.last().map(Vec::as_slice),
        Some(b"ROTATION STIFFNESS".as_slice())
    );
    let mut sentence = UnitSentence::new(context, b"kg/m/s").unwrap();
    let first = sentence.evaluate().unwrap();
    let second = sentence.evaluate().unwrap();
    let (first, second) = (first.get().unwrap(), second.get().unwrap());
    assert_eq!(first.word(), "(((k)*(g))/(m))/(s)");
    assert_eq!(first.word().as_bytes(), second.word().as_bytes());
    assert_eq!(first.value(), 1.0);
    assert_eq!(
        first.dimensions().get().unwrap().exponents(),
        &[1.0, -1.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
    );
    let mut unknown = UnitSentence::new(context, b"m m").unwrap();
    assert!(!unknown.is_done().unwrap());
    assert!(unknown.evaluate().unwrap().is_null());
}
