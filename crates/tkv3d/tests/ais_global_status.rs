//! Context state retains ordered modes and drawer identity.
use rustycad_tkernel::standard::{Handle, Transient};
use rustycad_tkv3d::{
    ais::AisGlobalStatus, prs3d::Prs3dDrawer, std_select::StdSelectViewerSelector3d,
};
#[test]
fn global_status_defaults_and_independent_flags() {
    let status = AisGlobalStatus::default();
    assert_eq!(status.display_mode(), 0);
    assert!(!status.is_hilighted());
    assert!(!status.is_sub_intensity_on());
    assert!(status.hilight_style().is_null());
    status.set_display_mode(-7);
    status.set_hilight_status(true);
    status.set_sub_intensity(true);
    assert_eq!(status.display_mode(), -7);
    assert!(status.is_hilighted());
    assert!(status.is_sub_intensity_on());
    status.set_hilight_status(false);
    assert!(status.is_sub_intensity_on());
}
#[test]
fn global_status_modes_are_unique_ordered_and_removable() {
    let status = AisGlobalStatus::new();
    assert!(status.add_selection_mode(5));
    assert!(status.add_selection_mode(-1));
    assert!(!status.add_selection_mode(5));
    assert!(status.is_s_mode_in(-1));
    assert_eq!(
        status.selection_modes().iter().copied().collect::<Vec<_>>(),
        [5, -1]
    );
    assert!(status.remove_selection_mode(5));
    assert!(!status.remove_selection_mode(5));
    status.clear_selection_modes();
    assert!(!status.is_s_mode_in(-1));
    assert!(status.selection_modes().is_empty());
}
#[test]
fn global_status_borrows_retained_style_and_modes() {
    let status = AisGlobalStatus::new();
    let drawer = Handle::new(Prs3dDrawer::new());
    status.set_hilight_style(&drawer);
    assert_eq!(*status.hilight_style(), drawer);
    assert!(std::ptr::eq(
        &*status.hilight_style(),
        &*status.hilight_style()
    ));
    assert!(std::ptr::eq(
        &*status.selection_modes(),
        &*status.selection_modes()
    ));
    status.set_hilight_style(&Handle::default());
    assert!(status.hilight_style().is_null());
}
#[test]
fn viewer_selector_typedef_preserves_native_type_identity() {
    let selector = StdSelectViewerSelector3d::new();
    assert_eq!(selector.dynamic_type().name(), "SelectMgr_ViewerSelector");
}
