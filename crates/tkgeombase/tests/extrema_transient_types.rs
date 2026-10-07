//! `Extrema_ExtPExtS` and `Extrema_ExtPRevS` are Standard_Transient classes
//! (DEFINE_STANDARD_RTTIEXT(..., Standard_Transient)); the Rust types register the same
//! descriptor name and parent, which `get_type_name`, `get_type_descriptor` and
//! `DynamicType` answer.
use rustycad_tkernel::standard::{RuntimeTyped, Transient};
use rustycad_tkgeombase::extrema::{ExtPExtS, ExtPRevS};

fn assert_descriptor<T: RuntimeTyped + Transient>(object: &T, name: &str) {
    assert_eq!(T::TYPE_NAME, name);
    assert_eq!(T::static_type().name(), name);
    assert_eq!(object.dynamic_type().name(), name);
    let parent = T::static_type()
        .parent()
        .map(|parent| parent.name().to_owned());
    assert_eq!(parent.as_deref(), Some("Standard_Transient"));
}

#[test]
fn ext_p_ext_s_registers_its_native_descriptor() {
    assert_descriptor(&ExtPExtS::default(), "Extrema_ExtPExtS");
}

#[test]
fn ext_p_rev_s_registers_its_native_descriptor() {
    assert_descriptor(&ExtPRevS::default(), "Extrema_ExtPRevS");
}
