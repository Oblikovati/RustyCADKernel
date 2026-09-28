//! Conversion context, byte encoding, temperature shifts and evaluator mutation.
use rustycad_tkernel::units::{UnitSentence, UnitSystem, UnitsApi, UnitsDictionary};
#[test]
fn isolated_context_selection_and_default_preserve_native_local_units() {
    let dictionary = UnitsDictionary::builtin().unwrap();
    let mut units = UnitsApi::new(dictionary.clone());
    assert_eq!(units.local_system(), UnitSystem::Si);
    assert_eq!(units.any_to_ls(2.0, "m").unwrap(), 2.0);
    units.set_local_system(UnitSystem::Mdtv).unwrap();
    assert_eq!(units.any_to_ls(2.0, "m").unwrap(), 2000.0);
    units.set_local_system(UnitSystem::Default).unwrap();
    assert_eq!(units.local_system(), UnitSystem::Mdtv);
    assert_eq!(UnitsApi::new(dictionary).any_to_ls(2.0, "m").unwrap(), 2.0);
    units.set_local_system(UnitSystem::Si).unwrap();
    assert_eq!(units.any_to_ls(2.0, "m").unwrap(), 2.0);
}
#[test]
fn shifted_temperature_conversion_and_legacy_byte_symbols() {
    let units = UnitsApi::new(UnitsDictionary::builtin().unwrap());
    assert_eq!(units.any_to_si(0.0, "deg.C").unwrap(), 273.15);
    assert_eq!(units.any_from_si(273.15, "deg.C").unwrap(), 0.0);
    // The native Fahrenheit offset is 1379/3, yielding this small nonzero Celsius result.
    assert_eq!(
        units.any_to_any(32.0, "deg.F", "deg.C").unwrap().to_bits(),
        13789555019046977536
    );
    assert_eq!(units.any_to_si(1.0, b"mm\xb2").unwrap(), 1e-6);
    assert_eq!(units.any_to_any(2.0, "m", "s").unwrap(), 2.0);
    assert_eq!(units.any_to_si(5.0, "unrecognized").unwrap(), 0.0);
    assert_eq!(units.any_from_si(5.0, "unrecognized").unwrap(), 0.0);
}
#[test]
fn dictionary_aliases_dimensions_and_destructive_evaluation_are_preserved() {
    let dictionary = UnitsDictionary::builtin().unwrap();
    assert_eq!(
        dictionary.quantity_names().next(),
        Some(b"DIMENSIONLESS".as_slice())
    );
    assert_eq!(
        dictionary.quantity_names().last(),
        Some(b"ROTATION STIFFNESS".as_slice())
    );
    let mut sentence = UnitSentence::new(&dictionary, b"kg/m/s").unwrap();
    let first = sentence.evaluate().unwrap().unwrap();
    let second = sentence.evaluate().unwrap().unwrap();
    assert_eq!(first.word(), b"(((k)*(g))/(m))/(s)");
    assert_eq!(first.word(), second.word());
    assert_eq!(first.value(), 1.0);
    assert_eq!(
        first.dimensions(),
        &[1.0, -1.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
    );
    let mut unknown = UnitSentence::new(&dictionary, b"m m").unwrap();
    assert!(!unknown.is_done());
    assert!(unknown.evaluate().unwrap().is_none());
}
