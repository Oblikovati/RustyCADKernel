//! Circle/arc identity, live components, selection snapshots and appearance notification rules.
#[allow(dead_code)]
#[path = "support/sensitive_display_driver.rs"]
mod display_driver;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::Handle,
};
use rustycad_tkg3d::geom::GeomCircle;
use rustycad_tkmath::gp::Axis2;
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::{
    ais::*, prs_mgr::*, prs3d::Prs3dDrawer, select_mgr::*, select3d::*, v3d::V3dViewer,
};
fn component(radius: f64) -> Handle<GeomCircle> {
    Handle::new(GeomCircle::new(Axis2::default(), radius).unwrap())
}
fn selected(
    object: &AisCircle,
    mode: i32,
) -> (
    Handle<SelectMgrSelection>,
    Handle<dyn Select3DSensitiveEntity>,
) {
    let selection = Handle::new(SelectMgrSelection::new(mode));
    object.compute_selection(&selection, mode).unwrap();
    let entity = selection.entities()[0].base_sensitive();
    (selection, entity)
}
#[test]
fn setters_force_arc_state_while_full_turn_constructors_keep_exact_circles() {
    let object = AisCircle::with_parameters(&component(3.), std::f64::consts::TAU, 0., true);
    assert_eq!(object.signature(), 6);
    assert_eq!(object.kind_of_interactive(), AisKindOfInteractive::Datum);
    assert!(object.is_filled_circle_sens());
    for mode in [-1, 0, 1, 99] {
        assert!(
            !selected(&object, mode)
                .1
                .cast::<Select3DSensitiveCircle>()
                .is_null()
        );
    }
    object.set_last_param(std::f64::consts::TAU);
    object.set_first_param(0.);
    let (selection, entity) = selected(&object, 99);
    assert_eq!(selection.entities().len(), 1);
    assert!(!entity.cast::<Select3DSensitivePoly>().is_null());
    assert_eq!(object.parameters(), (0., std::f64::consts::TAU));
    assert_eq!(entity.owner_id().priority(), 0);
    assert_eq!(
        entity.owner_id().selectable().object_address(),
        object.object_address()
    );
}
#[test]
fn selection_snapshots_geometry_and_filled_sensitivity() {
    let circle = component(3.);
    let object = AisCircle::new(&circle);
    let (_selection, old) = selected(&object, 0);
    assert_eq!(object.circle(), circle);
    assert_eq!(old.sensitivity_factor(), 6);
    circle.set_radius(4.).unwrap();
    object.set_filled_circle_sens(true);
    let (_selection2, new) = selected(&object, 0);
    assert_eq!(new.sensitivity_factor(), 2);
    assert_eq!(new.cast::<Select3DSensitiveCircle>().radius(), 4.);
    assert_eq!(old.cast::<Select3DSensitiveCircle>().radius(), 3.);
    object.set_circle(&component(2.));
    assert_eq!(object.circle().radius(), 2.);
}
#[test]
fn circle_unset_width_preserves_native_state_without_synchronizing_groups() {
    let (driver, trace) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let manager = PrsMgrPresentationManager::new(&viewer.structure_manager());
    let object = AisCircle::new(&component(3.));
    let drawer = Handle::new(Prs3dDrawer::new());
    drawer.setup_own_defaults().unwrap();
    drawer.line_aspect().set_width(2.5).unwrap();
    object.attributes().link_to(&drawer);
    object.set_color(Color::named(NamedColor::RED)).unwrap();
    object.set_width(4.).unwrap();
    manager
        .presentation(&object.cast(), 0, true, &Handle::default())
        .unwrap();
    trace.write().events.clear();
    object.unset_width().unwrap();
    assert!(trace.read().events.is_empty());
    assert_eq!(object.width(), 2.5);
    assert!(object.has_width());
    object.unset_color().unwrap();
    assert_eq!(
        object.color(),
        drawer.line_aspect().aspect().interior_color()
    );
}
#[test]
fn computation_restores_coefficient_after_success_and_retains_it_after_sampling_failure() {
    let (driver, _) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default())
        .unwrap()
        .cast();
    let object = AisCircle::new(&component(3.));
    object.attributes().setup_own_defaults().unwrap();
    object.attributes().set_deviation_coefficient(0.03);
    object.compute(&Handle::default(), &prs, 99).unwrap();
    assert_eq!(object.attributes().deviation_coefficient(), 0.03);
    object.attributes().set_deviation_angle(1.0e-13);
    assert!(object.compute(&Handle::default(), &prs, 0).is_err());
    assert_eq!(object.attributes().deviation_coefficient(), 1.0e-5);
}
