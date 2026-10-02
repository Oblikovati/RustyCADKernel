//! Regression coverage for the BOPDS solid interference subtype records.
use rustycad_tkbo::bop_ds::{InterfEZ, InterfFZ, InterfVZ, InterfZZ, Interference};

#[test]
fn solid_interference_records_share_native_base_index_state() {
    check_default_and_base_methods(&mut InterfVZ::default());
    check_default_and_base_methods(&mut InterfEZ::default());
    check_default_and_base_methods(&mut InterfFZ::default());
    check_default_and_base_methods(&mut InterfZZ::default());
}

fn check_default_and_base_methods<T: std::ops::DerefMut<Target = Interference>>(record: &mut T) {
    assert_eq!(record.indices(), (-1, -1));
    assert_eq!(record.index_new(), -1);
    assert!(!record.has_index_new());
    record.set_indices(4, 9);
    record.set_index_new(13);
    check_indices(record);
    check_new_index(record);
}

fn check_indices<T: std::ops::DerefMut<Target = Interference>>(record: &T) {
    assert_eq!(record.indices(), (4, 9));
    assert_eq!(record.opposite_index(4), 9);
    assert_eq!(record.opposite_index(9), 4);
    assert_eq!(record.opposite_index(27), -1);
    assert!(record.contains(4));
    assert!(record.contains(9));
    assert!(!record.contains(27));
}

fn check_new_index<T: std::ops::DerefMut<Target = Interference>>(record: &mut T) {
    assert!(record.has_index_new());
    let mut output = 77;
    assert!(record.has_index_new_into(&mut output));
    assert_eq!(output, 13);
    record.set_index_new(-1);
    assert!(!record.has_index_new());
}
