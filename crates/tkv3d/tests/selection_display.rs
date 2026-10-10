//! Sensitive debug geometry uses real groups and preserves native partial output.
#[path = "support/sensitive_display_driver.rs"]
mod display_driver;
#[path = "support/sensitive_display_scene.rs"]
mod display_scene;
use display_driver::SensitiveDisplayDriver;
use display_scene::*;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::{select_mgr::*, select3d::*, v3d::V3dViewer};
#[test]
fn every_sensitive_geometry_kind_emits_native_graphics() {
    for kind in 0..15 {
        render_kind(kind);
    }
}
fn render_kind(kind: i32) {
    let (driver, trace) = SensitiveDisplayDriver::new_handle(-1);
    let viewer = V3dViewer::new(&driver).unwrap();
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
    let selection = display_selection(kind, 1).unwrap();
    SelectMgr::compute_sensitive_prs(
        &prs.cast(),
        &selection,
        &display_location(2),
        &Handle::default(),
    )
    .unwrap();
    assert_eq!(prs.groups().is_empty(), kind == 13);
    assert!(!trace.read().events.is_empty());
}
#[test]
fn sensitive_curves_expose_the_shared_polyline_base() {
    let selection = display_selection(7, 0).unwrap();
    let entity = selection.entities()[0].base_sensitive();
    let poly = entity.cast::<dyn Select3DSensitivePolySource>();
    assert!(!poly.is_null());
    assert!(entity.is_kind_name("Select3D_SensitivePoly"));
    assert_eq!(poly.sensitive_poly_core().points_3d(), display_points());
}
#[allow(dead_code)]
#[path = "support/boxed_selectable.rs"]
mod boxed_selectable;
#[path = "support/sensitive_display_view.rs"]
mod display_view;
use display_view::SensitiveDisplayView;
#[test]
fn display_all_appends_empty_objects_and_redisplays_retained_structures() {
    let scene = SensitiveDisplayView::new(0, 1, 0, -1).unwrap();
    scene.run(0, 1).unwrap();
    scene.run(0, 1).unwrap();
    let fields = scene.selector.viewer_selector_base().fields.read();
    assert_eq!(fields.debug_structures.length(), 4);
    assert_eq!(
        fields
            .debug_structures
            .iter()
            .map(|p| p.groups().len())
            .collect::<Vec<_>>(),
        vec![1, 0, 1, 0]
    );
    assert!(
        fields
            .debug_structures
            .iter()
            .all(|p| p.is_displayed() && p.display_priority() == Graphic3dDisplayPriority::Topmost)
    );
}
#[test]
fn explicit_display_clears_old_structures_and_ignores_selection_activation() {
    let scene = SensitiveDisplayView::new(0, 0, 4, -1).unwrap();
    scene.run(0, 0).unwrap();
    let first = scene
        .selector
        .viewer_selector_base()
        .fields
        .read()
        .debug_structures
        .first()
        .unwrap()
        .clone();
    assert!(first.groups().is_empty());
    scene.run(2, 0).unwrap();
    assert!(!first.is_displayed());
    let fields = scene.selector.viewer_selector_base().fields.read();
    assert_eq!(fields.debug_structures.length(), 1);
    assert_eq!(fields.debug_structures.first().unwrap().groups().len(), 1);
}
#[test]
fn failing_group_creation_preserves_only_previously_appended_structures() {
    let scene = SensitiveDisplayView::new(0, 0, 0, 2).unwrap();
    scene.run(1, 0).unwrap();
    assert!(scene.run(1, 0).is_err());
    assert_eq!(retained_display_count(&scene), 1);
    scene.run(1, 0).unwrap();
    assert_eq!(retained_display_count(&scene), 2);
}
#[test]
fn flipping_callbacks_can_append_entities_and_live_object_selections() {
    let scene = SensitiveDisplayView::new(0, 0, 25, -1).unwrap();
    scene.run(0, 0).unwrap();
    assert_eq!(scene.selection.entities().len(), 2);
    let fields = scene.selector.viewer_selector_base().fields.read();
    assert_eq!(fields.debug_structures.first().unwrap().groups().len(), 3);
    assert!(scene.trace.read().events.contains(&6.));
    assert!(scene.trace.read().events.contains(&7.));
}

fn retained_display_count(scene: &SensitiveDisplayView) -> i32 {
    scene
        .selector
        .viewer_selector_base()
        .fields
        .read()
        .debug_structures
        .length()
}
