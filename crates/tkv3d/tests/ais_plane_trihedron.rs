//! Plane datum semantics include fresh accessor objects and retained selection child identities.
#[allow(dead_code)]
#[path = "support/sensitive_display_driver.rs"]
mod display_driver;
#[allow(dead_code)]
#[path = "support/ais_line_fixture.rs"]
mod fixture;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::Handle,
    tcollection::AsciiString,
    units_api::SystemUnits,
};
use rustycad_tkg3d::geom::GeomPlane;
use rustycad_tkmath::gp::{Axis3, Point3};
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::{ais::*, prs_mgr::*, prs3d::*, select_mgr::*, select3d::*, v3d::V3dViewer};
fn object() -> Handle<AisPlaneTrihedron> {
    AisPlaneTrihedron::new(
        &Handle::new(GeomPlane::new(Axis3::default())),
        fixture::units(),
    )
    .unwrap()
}
fn selection(object: &AisPlaneTrihedron, mode: i32) -> Handle<SelectMgrSelection> {
    let selection = Handle::new(SelectMgrSelection::new(mode));
    object.compute_selection(&selection, mode).unwrap();
    selection
}
#[test]
fn constructors_convert_units_and_accessors_allocate_live_datums() {
    for (system, expected) in [(SystemUnits::Si, 0.1), (SystemUnits::Mdtv, 100.)] {
        let units = fixture::units();
        units.write().set_local_system(system).unwrap();
        let plane = Handle::new(GeomPlane::new(Axis3::default()));
        let object = AisPlaneTrihedron::new(&plane, units).unwrap();
        assert!((object.get_length() - expected).abs() < 1e-14);
        assert_eq!(object.component(), plane);
        assert_eq!(object.signature(), 4);
        assert_eq!(object.kind_of_interactive(), AisKindOfInteractive::Datum);
        assert_ne!(object.x_axis().unwrap(), object.x_axis().unwrap());
        assert_eq!(
            object.y_axis().unwrap().line().position(),
            plane.pln().y_axis()
        );
        assert_eq!(
            object.position().unwrap().component().pnt(),
            Point3::default()
        );
        assert_eq!(
            object.x_axis().unwrap().color(),
            Color::named(NamedColor::ROYALBLUE1)
        );
        assert!(!object.is_infinite());
        for mode in [-1, 0, 1, 2, 3] {
            assert_eq!(object.accept_display_mode(mode), mode == 0);
        }
    }
}
#[test]
fn selection_modes_share_whole_owner_and_keep_old_child_geometry() {
    let object = object();
    object.set_length(4.);
    let whole = selection(&object, 0);
    let entities = whole.entities();
    let first = entities[0]
        .base_sensitive()
        .cast::<Select3DSensitiveSegment>();
    let second = entities[1]
        .base_sensitive()
        .cast::<Select3DSensitiveSegment>();
    assert_eq!(first.owner_id(), second.owner_id());
    assert_eq!(first.owner_id().priority(), 5);
    assert_eq!(
        first.owner_id().selectable().object_address(),
        object.object_address()
    );
    let old = selection(&object, 2);
    let owner = old.entities()[0].base_sensitive().owner_id().selectable();
    let mut frame = Axis3::default();
    frame.set_location(Point3::new(8., 9., 10.));
    object.set_component(&Handle::new(GeomPlane::new(frame)));
    let updated = selection(&object, 2);
    let segment = updated.entities()[0]
        .base_sensitive()
        .cast::<Select3DSensitiveSegment>();
    assert_eq!(segment.owner_id().priority(), 7);
    assert_eq!(segment.owner_id().selectable(), owner);
    assert_eq!(
        owner.cast::<AisLine>().line().position().location(),
        Point3::default()
    );
    assert_eq!(segment.start_point(), frame.location());
    assert_eq!(segment.end_point(), Point3::new(12., 9., 10.));
    assert_eq!(
        object.x_axis().unwrap().line().position().location(),
        frame.location()
    );
}
#[test]
fn origin_selection_tracks_plane_but_not_child_point_and_minus_one_clears() {
    let object = object();
    let initial = selection(&object, 1);
    let owner = initial.entities()[0]
        .base_sensitive()
        .owner_id()
        .selectable();
    let mut plane = object.component().pln();
    plane.set_location(Point3::new(3., 2., 1.));
    object.component().set_pln(plane);
    let selected = selection(&object, 1);
    let point = selected.entities()[0]
        .base_sensitive()
        .cast::<Select3DSensitivePoint>();
    assert_eq!(point.point(), plane.location());
    assert_eq!(point.owner_id().priority(), 8);
    assert_eq!(point.owner_id().selectable(), owner);
    assert_eq!(
        owner.cast::<AisPoint>().component().pnt(),
        Point3::default()
    );
    object.compute_selection(&selected, 99).unwrap();
    assert_eq!(selected.entities().len(), 1);
    object.compute_selection(&selected, -1).unwrap();
    assert!(selected.entities().is_empty());
}
#[test]
fn length_and_color_mutations_preserve_native_drawer_rules() {
    let object = object();
    for length in [-3., 0., 2.] {
        object.set_length(length);
        assert_eq!(object.get_length(), length);
        assert_eq!(
            object
                .attributes()
                .datum_aspect()
                .axis_length(Prs3dDatumParts::ZAxis),
            length
        );
    }
    object.set_color(Color::named(NamedColor::RED)).unwrap();
    assert!(object.has_color());
    assert_eq!(object.color(), Color::named(NamedColor::RED));
    assert_eq!(
        object
            .attributes()
            .datum_aspect()
            .line_aspect(Prs3dDatumParts::YAxis)
            .aspect()
            .interior_color(),
        object.color()
    );
    assert_eq!(
        object.y_axis().unwrap().color(),
        Color::named(NamedColor::ROYALBLUE1)
    );
}
#[test]
fn presentation_retains_byte_labels_and_infinite_state_on_structure() {
    let object = object();
    let linked = Handle::new(Prs3dDrawer::new());
    linked.setup_own_defaults().unwrap();
    object.attributes().link_to(&linked);
    object.set_x_label(&AsciiString::from_raw(b"U\0ignored"));
    object.set_y_label(&AsciiString::from("V"));
    object.set_length(3.);
    let (driver, trace) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let manager = PrsMgrPresentationManager::new(&viewer.structure_manager());
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
    object.compute(&manager, &prs.cast(), 0).unwrap();
    assert!(prs.is_infinite());
    assert!(!object.is_infinite());
    let texts = &trace.read().texts;
    assert_eq!(texts.len(), 2);
    assert_eq!(texts[0].text().to_c_string(), b"U\0");
    assert_eq!(texts[1].text().to_c_string(), b"V\0");
    assert_eq!(texts[0].position(), Point3::new(3., 0., 0.));
    assert_eq!(texts[1].position(), Point3::new(0., 3., 0.));
}

#[test]
fn only_length_marks_existing_presentations_for_update() {
    let object = object();
    let linked = Handle::new(Prs3dDrawer::new());
    linked.setup_own_defaults().unwrap();
    object.attributes().link_to(&linked);
    let (driver, _) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let manager = PrsMgrPresentationManager::new(&viewer.structure_manager());
    manager.display(&object.cast(), 0).unwrap();
    assert!(!object.to_be_updated(false));
    object.set_component(&object.component());
    object.set_x_label(&AsciiString::from("U"));
    object.set_y_label(&AsciiString::from("V"));
    assert!(!object.to_be_updated(false));
    object.set_length(5.);
    assert!(object.to_be_updated(false));
}
