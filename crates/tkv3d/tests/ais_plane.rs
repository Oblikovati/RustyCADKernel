//! Native plane state, cached selection and appearance ownership regressions.
#[allow(dead_code)]
#[path = "support/sensitive_display_driver.rs"]
mod display_driver;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::Handle,
};
use rustycad_tkg3d::geom::{GeomAxis2Placement, GeomPlane};
use rustycad_tkmath::gp::{Axis2, Axis3, Direction3, Point3};
use rustycad_tkservice::{aspect::AspectTypeOfFacingModel, graphic3d::*};
use rustycad_tkv3d::{ais::*, prs_mgr::*, prs3d::*, select_mgr::*, select3d::*, v3d::V3dViewer};
fn prepared(object: Handle<AisPlane>) -> Handle<AisPlane> {
    let d = Handle::new(Prs3dDrawer::new());
    d.setup_own_defaults().unwrap();
    object.attributes().link_to(&d);
    object
}
fn geometry() -> Handle<GeomPlane> {
    Handle::new(GeomPlane::new(Axis3::default()))
}
struct PlaneSelectionSnapshot {
    _selection: Handle<SelectMgrSelection>,
    entity: Handle<Select3DSensitiveTriangulation>,
}
impl std::ops::Deref for PlaneSelectionSnapshot {
    type Target = Select3DSensitiveTriangulation;
    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}
fn selected(object: &AisPlane, mode: i32) -> PlaneSelectionSnapshot {
    let selection = Handle::new(SelectMgrSelection::new(mode));
    object.compute_selection(&selection, mode).unwrap();
    object.compute_selection(&selection, mode).unwrap();
    assert_eq!(selection.entities().len(), 1);
    let entity = selection.entities()[0].base_sensitive().cast();
    PlaneSelectionSnapshot {
        _selection: selection,
        entity,
    }
}
fn compute(object: &AisPlane, mode: i32) -> display_driver::DisplayTrace {
    let (driver, trace) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
    object
        .compute(&Handle::default(), &prs.cast(), mode)
        .unwrap();
    trace
}
#[test]
fn plane_constructors_and_geometry_setters_preserve_native_state() {
    let component = geometry();
    let center = Point3::new(2., 3., 4.);
    let object = AisPlane::with_center(&component, center, true).unwrap();
    assert_eq!(object.signature(), 7);
    assert_eq!(object.kind_of_interactive(), AisKindOfInteractive::Datum);
    assert!(object.current_mode());
    assert!(!object.has_own_size());
    assert!(!object.is_xyz_plane());
    check_explicit_attributes(&object, &component, center);
    object.set_component(&component);
    assert_eq!(object.center(), center);
    object.set_current_mode(false);
    assert!(!object.current_mode());
    object.set_center(Point3::default());
    assert_eq!(object.center(), Point3::default());
}
#[test]
fn rectangle_selection_replaces_entities_and_snapshots_geometry_and_sensitivity() {
    let object = AisPlane::with_center(&geometry(), Point3::new(2., 3., 4.), false).unwrap();
    let object = prepared(object);
    object.set_size_xy(4., 6.).unwrap();
    let old = selected(&object, -1);
    assert_eq!(old.owner_id().priority(), 10);
    assert_eq!(
        old.triangulation().node(1).unwrap(),
        Point3::new(4., 6., 4.)
    );
    assert_eq!(old.triangulation().triangle(2).unwrap().get(), (1, 3, 4));
    object.set_type_of_sensitivity(Select3DTypeOfSensitivity::Interior);
    assert_eq!(
        object.type_of_sensitivity(),
        Select3DTypeOfSensitivity::Interior
    );
    object.set_center(Point3::default());
    let new = selected(&object, 99);
    check_rectangle_snapshots(&old, &new);
}
#[test]
fn xyz_sizes_leave_cached_corners_until_compute_and_keep_global_component_planes() {
    let frame = Handle::new(GeomAxis2Placement::new(
        Axis2::with_x_direction(Point3::new(2., 3., 4.), Direction3::Z, Direction3::X).unwrap(),
    ));
    for kind in [
        AisTypeOfPlane::XYPlane,
        AisTypeOfPlane::XZPlane,
        AisTypeOfPlane::YZPlane,
    ] {
        check_xyz_refresh(&frame, kind);
    }
}
#[test]
fn automatic_frame_projects_center_and_manual_frame_ignores_maximum_point() {
    let automatic = AisPlane::with_center(&geometry(), Point3::new(2., 3., 4.), false).unwrap();
    let automatic = prepared(automatic);
    compute(&automatic, 0);
    assert_eq!(automatic.center(), Point3::new(2., 3., 0.));
    let manual = manual_fixture();
    let manual = prepared(manual);
    compute(&manual, 0);
    let (mut x, mut y) = (0., 0.);
    assert!(!manual.size(&mut x, &mut y));
    assert_eq!((x, y), (0.1, 4.8));
    assert!(manual.has_own_size());
    assert_eq!(
        manual
            .attributes()
            .plane_aspect()
            .edges_aspect()
            .aspect()
            .width(),
        1.
    );
    assert!(manual.accept_display_mode(0));
    assert!(!manual.accept_display_mode(1));
}
#[test]
fn size_and_color_copy_on_write_create_defaults_and_restore_only_native_properties() {
    let object = AisPlane::new(&geometry(), false).unwrap();
    let inherited = Handle::new(Prs3dDrawer::new());
    inherited.setup_own_defaults().unwrap();
    inherited.plane_aspect().set_plane_length(11., 13.);
    inherited.plane_aspect().set_iso_distance(7.);
    inherited.datum_aspect().set_axis_length(2., 3., 5.);
    object.attributes().link_to(&inherited);
    check_color_size_restore(&object, &inherited);
    object.set_size(3.).unwrap();
    object.set_color(Color::named(NamedColor::BLUE)).unwrap();
    object.unset_color().unwrap();
    assert!(object.has_own_size());
    assert!(!object.has_color());
    object.unset_size().unwrap();
    object.unset_size().unwrap();
    object.unset_color().unwrap();
}
#[test]
fn minimum_size_uses_concrete_persistence_and_clears_for_nonpositive_values() {
    let object = AisPlane::new(&geometry(), false).unwrap();
    let object = prepared(object);
    object.set_size_xy(4., 6.).unwrap();
    object.set_minimum_size(2.).unwrap();
    assert!(object.has_minimum_size());
    let pers = object.transform_persistence();
    assert_eq!(pers.anchor_point().unwrap(), Point3::default());
    object.set_minimum_size(0.).unwrap();
    assert!(!object.has_minimum_size());
    object.set_minimum_size(1.).unwrap();
    object.unset_minimum_size();
    assert!(!object.has_minimum_size());
    let model = AspectTypeOfFacingModel::BothSide;
    assert_eq!(
        object.attributes().shading_aspect().transparency(model),
        f64::from(0.8_f32)
    );
}
#[test]
fn shaded_rectangle_keeps_untranslated_plane_origin_and_quadrangle_topology() {
    let object = AisPlane::with_center(&geometry(), Point3::new(10., 20., 30.), false).unwrap();
    let object = prepared(object);
    object.set_size_xy(4., 6.).unwrap();
    let trace = compute(&object, 1);
    let trace = trace.read();
    let index = trace
        .events
        .iter()
        .position(|v| *v == Graphic3dTypeOfPrimitiveArray::Quadrangles as i32 as f64)
        .unwrap();
    assert_eq!(
        &trace.events[index..index + 14],
        &[11., 4., -2., 3., 0., 2., 3., 0., 2., -3., 0., -2., -3., 0.]
    );
    assert_eq!(object.center(), Point3::new(10., 20., 0.));
}

fn check_explicit_attributes(object: &AisPlane, component: &Handle<GeomPlane>, center: Point3) {
    let mut output = Handle::default();
    let mut c = center;
    let mut a = center;
    let mut b = center;
    assert!(!object.plane_attributes(&mut output, &mut c, &mut a, &mut b));
    assert!(output.is_null());
    assert_eq!([c, a, b], [center; 3]);
    assert!(object.axis2_placement().is_null());
    object.set_plane_attributes(
        component,
        center,
        Point3::new(1., 0., 0.),
        Point3::new(99., 99., 99.),
    );
    assert!(object.plane_attributes(&mut output, &mut c, &mut a, &mut b));
    assert_eq!(output, *component);
}

fn check_rectangle_snapshots(old: &PlaneSelectionSnapshot, new: &PlaneSelectionSnapshot) {
    assert_eq!(
        new.triangulation().node(1).unwrap(),
        Point3::new(2., 3., 0.)
    );
    assert_eq!(
        old.triangulation().node(1).unwrap(),
        Point3::new(4., 6., 4.)
    );
    assert_eq!(old.nb_sub_elements(), 4);
    assert_eq!(new.nb_sub_elements(), 4);
    assert_eq!(Select3DSensitiveSet::size(&**old), 4);
    assert_eq!(Select3DSensitiveSet::size(&**new), 2);
}

fn check_xyz_refresh(frame: &Handle<GeomAxis2Placement>, kind: AisTypeOfPlane) {
    let object = prepared(AisPlane::with_axis2(frame, kind, false).unwrap());
    assert_eq!(object.axis2_placement(), *frame);
    assert_eq!(object.type_of_plane(), kind);
    let old = selected(&object, 0).triangulation().node(2).unwrap();
    object.set_size_xy(4., 6.).unwrap();
    assert_eq!(selected(&object, 0).triangulation().node(2).unwrap(), old);
    compute(&object, 77);
    check_xyz_mesh(&object, kind);
    object
        .set_axis2_placement(frame, AisTypeOfPlane::Unknown)
        .unwrap();
    assert_eq!(object.fields.read().pmin, Point3::default());
    assert_eq!(object.center(), Point3::new(2., 3., 4.));
}

fn manual_fixture() -> Handle<AisPlane> {
    AisPlane::with_bounds(
        &geometry(),
        Point3::new(2., 3., 4.),
        Point3::new(2., 1., 50.),
        Point3::new(100., 100., 100.),
        true,
    )
    .unwrap()
}

fn check_color_size_restore(object: &AisPlane, inherited: &Handle<Prs3dDrawer>) {
    object.set_size(4.).unwrap();
    assert_ne!(object.attributes().plane_aspect(), inherited.plane_aspect());
    assert_ne!(object.attributes().plane_aspect().iso_distance(), 7.);
    object.set_color(Color::named(NamedColor::RED)).unwrap();
    object.unset_size().unwrap();
    assert!(!object.has_own_size());
    assert!(object.has_color());
    let (mut x, mut y) = (0., 0.);
    object.size(&mut x, &mut y);
    assert_eq!((x, y), (11., 13.));
    assert_eq!(
        object
            .attributes()
            .datum_aspect()
            .axis_length(Prs3dDatumParts::ZAxis),
        5.
    );
    check_released_color(object, inherited);
}

fn check_xyz_mesh(object: &AisPlane, kind: AisTypeOfPlane) {
    let mesh = selected(object, 1).triangulation();
    let expected = if kind == AisTypeOfPlane::YZPlane {
        Point3::new(2., 9., 4.)
    } else {
        Point3::new(6., 3., 4.)
    };
    assert_eq!(mesh.node(2).unwrap(), expected);
    assert_eq!(mesh.nb_triangles(), 1);
    assert_eq!(object.component().pln().location(), Point3::default());
}

fn check_released_color(object: &AisPlane, inherited: &Handle<Prs3dDrawer>) {
    object.unset_color().unwrap();
    assert_eq!(object.attributes().plane_aspect(), inherited.plane_aspect());
    assert_eq!(
        object
            .attributes()
            .shading_aspect()
            .color(AspectTypeOfFacingModel::BothSide),
        Color::named(NamedColor::RED)
    );
}
