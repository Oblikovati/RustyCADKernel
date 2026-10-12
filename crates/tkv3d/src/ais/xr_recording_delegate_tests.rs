//! Mandatory inherited driver hooks delegate to real null-driver state rather than canned responses.
use super::{
    xr_recording_group::XrRecordingEvents, xr_recording_structure::xr_recording_presentation,
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::Axis2;
use rustycad_tkservice::graphic3d::*;
use std::sync::Arc;
#[test]
fn xr_named_driver_delegates_structure_and_group_hooks() {
    let events = Arc::new(XrRecordingEvents::new());
    let prs = xr_recording_presentation(&events);
    assert!(!prs.structure_manager().is_null());
    let driver = prs.cstructure();
    exercise_xr_structure_hooks(&driver);
    let group = prs.new_group().unwrap();
    exercise_xr_group_hooks(&group);
    let shadow = driver.shadow_link(&prs.structure_manager()).unwrap();
    // OCCT 3d097a0 shadows retain parents without adopting removable group nodes.
    assert!(shadow.groups().is_empty());
    shadow.clear();
    assert_eq!(driver.groups().len(), 1);
    driver.remove_group(&group);
    assert!(driver.groups().is_empty());
    driver.clear();
    assert!(driver.groups().is_empty());
}
fn exercise_xr_structure_hooks(driver: &Handle<dyn Graphic3dCStructure>) {
    driver.on_visibility_changed();
    driver.connect(&**driver);
    driver.disconnect(&**driver);
    let style = Handle::new(Graphic3dPresentationAttributes::default()).cast();
    driver.graphic_highlight(&style);
    assert!(!driver.highlight_style().is_null());
    driver.graphic_unhighlight();
    assert!(driver.highlight_style().is_null());
}
fn exercise_xr_group_hooks(group: &Handle<dyn Graphic3dGroup>) {
    let aspects = Handle::new(Graphic3dAspects::default()).cast::<dyn Graphic3dAspectsSource>();
    group.set_primitives_aspect(&aspects);
    assert_eq!(group.aspects(), aspects);
    group.synchronize_aspects();
    group.replace_aspects(&Graphic3dAspectsReplaceMap::default());
    group.set_stencil_test_options(true);
    group.set_flipping_options(false, &Axis2::default());
    assert_eq!(group.aspects(), aspects);
}
