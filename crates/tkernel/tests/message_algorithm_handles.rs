//! Shared-handle regression for the real Message_Algorithm base used by mesh builders.
use rustycad_tkernel::{
    impl_transient,
    message::{Algorithm, Status},
    standard::{Handle, RuntimeTyped, StandardTransient},
};

struct DerivedMessageAlgorithm {
    parent: Algorithm,
}
impl DerivedMessageAlgorithm {
    fn new() -> Self {
        Self {
            parent: Algorithm::with_runtime_type(Self::static_type()),
        }
    }
}
impl_transient!(
    DerivedMessageAlgorithm,
    "Probe_MessageAlgorithm",
    Algorithm,
    parent
);

#[test]
fn algorithm_handles_mutate_one_real_base_subobject() {
    let derived = Handle::new(DerivedMessageAlgorithm::new());
    let base: Handle<Algorithm> = derived.cast();
    let root: Handle<StandardTransient> = base.cast();
    base.set_status_with_integer(Status::FAIL2, 17);
    assert!(derived.parent.get_status().is_set(Status::FAIL2));
    assert_eq!(base.get_message_numbers(Status::FAIL2).unwrap().len(), 1);
    assert_eq!(
        root.dynamic_type().unwrap().name(),
        "Probe_MessageAlgorithm"
    );
    assert_eq!(
        base.dynamic_type().unwrap().name(),
        "Probe_MessageAlgorithm"
    );
    assert!(!root.cast::<DerivedMessageAlgorithm>().is_null());
    assert_eq!(root.ref_count(), 3);
    base.change_status().clear_status(Status::FAIL2);
    assert!(!derived.parent.get_status().is_fail());
    assert!(
        derived
            .parent
            .get_message_numbers(Status::FAIL2)
            .unwrap()
            .contains(17)
    );
}

#[test]
fn standalone_algorithm_handles_and_self_merging_preserve_state() {
    let algorithm = Handle::new(Algorithm::new());
    let alias = algorithm.clone();
    algorithm.set_status_with_integer(Status::WARN1, 3);
    algorithm.set_status_with_c_str(Status::WARN1, b"first", true);
    algorithm.set_status_with_c_str(Status::WARN1, b"last", true);
    alias.add_status(&algorithm);
    assert_eq!(
        algorithm.get_message_strings(Status::WARN1).unwrap().len(),
        2
    );
    assert_eq!(
        algorithm.get_message_numbers(Status::WARN1).unwrap().len(),
        1
    );
    let root: Handle<StandardTransient> = alias.cast();
    assert_eq!(root.dynamic_type().unwrap().name(), "Message_Algorithm");
    assert!(!root.cast::<Algorithm>().is_null());
    alias.clear_status();
    assert!(!algorithm.get_status().is_warn());
    assert!(algorithm.get_message_numbers(Status::WARN1).is_none());
}
