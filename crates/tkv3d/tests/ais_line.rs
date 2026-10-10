//! AIS_Line identity, retained geometry, units, errors and native appearance-state quirks.
#[allow(dead_code)]
#[path = "support/sensitive_display_driver.rs"]
mod display_driver;
#[path = "support/ais_line_fixture.rs"]
mod fixture;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::Handle,
    units_api::SystemUnits,
};
use rustycad_tkg3d::geom::{GeomCartesianPoint, GeomPoint};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::{
    ais::*, prs_mgr::*, prs3d::Prs3dDrawer, select_mgr::*, select3d::*, v3d::V3dViewer,
};
fn presentation() -> Handle<dyn Graphic3dStructureSource> {
    let (driver, _) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default())
        .unwrap()
        .cast()
}
fn selected_segment(
    line: &AisLine,
) -> (Handle<SelectMgrSelection>, Handle<Select3DSensitiveSegment>) {
    let selection = Handle::new(SelectMgrSelection::new(0));
    line.compute_selection(&selection, 0).unwrap();
    let segment = selection.entities()[0].base_sensitive().cast();
    (selection, segment)
}
#[test]
fn live_units_change_infinite_selection_and_owner_identity() {
    let units = fixture::units();
    let line = fixture::object(false, units.clone());
    assert_eq!(line.kind_of_interactive(), AisKindOfInteractive::Datum);
    assert_eq!(line.signature(), 5);
    let (_first_selection, first) = selected_segment(&line);
    assert!((first.start_point().distance(&first.end_point()) - 500.).abs() < 1e-10);
    assert_eq!(first.owner_id().priority(), 5);
    assert_eq!(
        first.owner_id().selectable().object_address(),
        line.object_address()
    );
    std::thread::spawn(move || units.write().set_local_system(SystemUnits::Mdtv).unwrap())
        .join()
        .unwrap();
    let (_second_selection, second) = selected_segment(&line);
    assert!((second.start_point().distance(&second.end_point()) - 500000.).abs() < 1e-9);
    let selection = Handle::new(SelectMgrSelection::new(1));
    line.compute_selection(&selection, 1).unwrap();
    assert!(selection.entities().is_empty());
}
#[test]
fn setters_preserve_native_cached_geometry_and_infinite_state() {
    let line = fixture::object(true, fixture::units());
    let (a, b) = line.points();
    assert_eq!(line.start_point(), a);
    assert_eq!(line.end_point(), b);
    assert!(line.line().is_null());
    line.attributes().setup_own_defaults().unwrap();
    line.compute(&Handle::default(), &presentation(), 4)
        .unwrap();
    let cached = line.line();
    assert!(!cached.is_null());
    let moved = Handle::new(GeomCartesianPoint::new(5., 4., 0.));
    line.set_points(&a, &moved.cast());
    assert_eq!(line.line(), cached);
    assert!(!line.is_infinite());
    moved.set_pnt(Point3::new(6., 5., 0.));
    assert_eq!(selected_segment(&line).1.end_point(), moved.pnt());
    line.set_line(&fixture::component());
    assert!(!line.is_infinite());
    let infinite = fixture::object(false, fixture::units());
    infinite.set_points(&a, &b);
    assert!(infinite.is_infinite());
}
#[test]
fn width_errors_dispatch_through_base_and_preserve_partial_assignment() {
    let line = fixture::object(true, fixture::units());
    let base = line.cast::<dyn PrsMgrPresentableObject>();
    let error = base.set_width(-2.).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    assert_eq!(line.width(), -2.);
    assert!(!line.attributes().has_own_line_aspect());
    base.set_width(3.).unwrap();
    base.unset_width().unwrap();
    assert_eq!(line.width(), 3.);
    assert!(line.has_width());
    assert!(!line.attributes().has_own_line_aspect());
}
#[test]
fn appearance_restoration_uses_link_and_keeps_native_own_width() {
    let line = fixture::object(true, fixture::units());
    let linked = Handle::new(Prs3dDrawer::new());
    linked.setup_own_defaults().unwrap();
    linked.line_aspect().set_width(2.5).unwrap();
    linked
        .line_aspect()
        .set_color(Color::named(NamedColor::GREEN));
    line.attributes().link_to(&linked);
    line.set_color(Color::named(NamedColor::RED)).unwrap();
    assert_eq!(line.attributes().line_aspect().aspect().width(), 2.5);
    line.set_width(4.).unwrap();
    line.unset_color().unwrap();
    assert_eq!(line.color(), Color::named(NamedColor::GREEN));
    line.set_color(Color::named(NamedColor::RED)).unwrap();
    line.unset_width().unwrap();
    assert_eq!(line.width(), 2.5);
    assert!(line.has_width());
}
#[test]
fn coincident_segment_fails_before_replacing_the_cached_component() {
    let line = fixture::object(true, fixture::units());
    let prs = presentation();
    line.attributes().setup_own_defaults().unwrap();
    line.compute(&Handle::default(), &prs, 0).unwrap();
    let cached = line.line();
    let point = line.start_point();
    line.set_points(&point, &point);
    assert_eq!(
        line.compute(&Handle::default(), &prs, 0)
            .unwrap_err()
            .exception_type(),
        "Standard_ConstructionError"
    );
    assert_eq!(line.line(), cached);
    assert_eq!(
        selected_segment(&line).1.start_point(),
        selected_segment(&line).1.end_point()
    );
}
