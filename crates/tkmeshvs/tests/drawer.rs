// SPDX-License-Identifier: GPL-2.0-only
//! Independent drawer invariants complement the pinned native transcript.
use martensite_tkmeshvs::{MeshVSDrawer,MeshVSDrawerSource};
#[test]
fn integer_missing_output_overwrite_and_remove_are_native() {
    let drawer=MeshVSDrawer::default();
    let mut output=99;
    assert!(!drawer.get_integer(-7,&mut output));
    assert_eq!(output,99);
    drawer.set_integer(-7,42);
    drawer.set_integer(-7,51);
    assert!(drawer.get_integer(-7,&mut output));
    assert_eq!(output,51);
    assert!(drawer.remove_integer(-7));
    assert!(!drawer.remove_integer(-7));
}
#[test]
fn null_self_and_independent_assignment_preserve_type_separation() {
    use rustycad_tkernel::standard::Handle;
    let source=Handle::new(MeshVSDrawer::default());
    source.set_integer(7,42);
    source.set_double(7,3.5);
    source.set_boolean(7,true);
    source.assign(&source);
    source.assign(&Handle::<dyn MeshVSDrawerSource>::default());
    let target=MeshVSDrawer::default();
    target.assign(&source);
    source.set_integer(7,9);
    let mut output=0;
    assert!(target.get_integer(7,&mut output));
    assert_eq!(output,42);
    assert_eq!(source.ref_count(),1);
}
