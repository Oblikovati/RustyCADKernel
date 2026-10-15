//! Port tests of TPrsStd_DriverTable and the standard presentation drivers.
mod support;

use rustycad_tkcaf::tdata_xtd::{
    TDataXtdAxis, TDataXtdConstraint, TDataXtdGeometry, TDataXtdGeometryEnum, TDataXtdPlane,
    TDataXtdPoint,
};
use rustycad_tkcaf::tnaming::TNamingNamedShape;
use rustycad_tkernel::quantity::NamedColor;
use rustycad_tkernel::standard::{Guid, Handle};
use rustycad_tkmath::gp::{Axis1, Direction3, Line3, Plane3, Point3};
use rustycad_tkv3d::ais::{
    AisAxis, AisCircle, AisInteractiveObject, AisLine, AisPlane, AisPoint, AisShape,
    AisShapeParameters,
};
use rustycad_tkv3d::prs_mgr::PrsMgrPresentableObjectMembers;
use rustycad_tkvcaf::tprs_std::{
    TPrsStdAxisDriver, TPrsStdDriver, TPrsStdDriverTable, TPrsStdGeometryDriver,
    TPrsStdNamedShapeDriver, TPrsStdPlaneDriver, TPrsStdPointDriver,
};

/// The standard GUIDs `InitStandardDrivers` binds.
fn standard_guids() -> [Guid; 6] {
    [
        TDataXtdAxis::get_id(),
        TDataXtdConstraint::get_id(),
        TDataXtdGeometry::get_id(),
        TNamingNamedShape::get_id(),
        TDataXtdPlane::get_id(),
        TDataXtdPoint::get_id(),
    ]
}

#[test]
fn a_new_table_has_the_standard_drivers() {
    let table = TPrsStdDriverTable::new();
    for guid in standard_guids() {
        let mut driver = Handle::default();
        assert!(table.find_driver(&guid, &mut driver));
        assert!(!driver.is_null());
    }
    let mut driver: Handle<dyn TPrsStdDriver> = Handle::default();
    assert!(table.find_driver(&TDataXtdPoint::get_id(), &mut driver));
    assert!(!driver.cast::<TPrsStdPointDriver>().is_null());
}

#[test]
fn table_add_remove_clear_and_init() {
    let table = TPrsStdDriverTable::new();
    let point = TDataXtdPoint::get_id();
    let driver: Handle<dyn TPrsStdDriver> = Handle::new(TPrsStdPointDriver::new()).cast();
    assert!(!table.add_driver(&point, &driver));
    assert!(table.remove_driver(&point));
    assert!(!table.remove_driver(&point));
    table.init_standard_drivers();
    let mut found = Handle::default();
    assert!(!table.find_driver(&point, &mut found));
    table.clear();
    table.init_standard_drivers();
    assert!(table.find_driver(&point, &mut found));
    assert!(
        TPrsStdDriverTable::get().object_address() == TPrsStdDriverTable::get().object_address()
    );
}

#[test]
fn construction_drivers_build_and_keep_their_objects() {
    let document = support::new_document();
    let point = support::main_child(&document, 1);
    TDataXtdPoint::set_point(&point, Point3::new(1.0, 0.0, 0.0)).unwrap();
    let mut object: Handle<dyn AisInteractiveObject> = Handle::default();
    assert!(
        TPrsStdPointDriver::new()
            .update(&point, &mut object)
            .unwrap()
    );
    assert!(!object.cast::<AisPoint>().is_null());
    let first = object.object_address();
    assert!(
        TPrsStdPointDriver::new()
            .update(&point, &mut object)
            .unwrap()
    );
    assert_eq!(object.object_address(), first);
    assert!(
        !TPrsStdAxisDriver::new()
            .update(&point, &mut object)
            .unwrap()
    );

    let axis = support::main_child(&document, 2);
    let line = Line3::from_axis(Axis1::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    ));
    TDataXtdAxis::set_line(&axis, &line).unwrap();
    assert!(TPrsStdAxisDriver::new().update(&axis, &mut object).unwrap());
    assert!(!object.cast::<AisAxis>().is_null());

    let plane = support::main_child(&document, 3);
    TDataXtdPlane::set_plane(&plane, &Plane3::default()).unwrap();
    let mut object: Handle<dyn AisInteractiveObject> = Handle::default();
    assert!(
        TPrsStdPlaneDriver::new()
            .update(&plane, &mut object)
            .unwrap()
    );
    assert!(!object.cast::<AisPlane>().is_null());
}

#[test]
fn geometry_driver_shows_red_points_lines_and_circles() {
    let document = support::new_document();
    let root = document.main().unwrap();
    let line = support::named_shape(
        &root,
        &support::segment_edge(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)),
    );
    let label = rustycad_tklcaf::tdf::TdfAttributeState::label(line.get().unwrap());
    assert_eq!(
        TDataXtdGeometry::type_of_label(&label).unwrap(),
        TDataXtdGeometryEnum::Line
    );
    let mut object: Handle<dyn AisInteractiveObject> = Handle::default();
    assert!(
        TPrsStdGeometryDriver::new()
            .update(&label, &mut object)
            .unwrap()
    );
    assert!(!object.cast::<AisLine>().is_null());
    assert_eq!(object.color().name(), NamedColor::RED);
    assert!(object.is_infinite());

    let circle = support::named_shape(
        &root,
        &support::circle_edge(Point3::new(0.0, 0.0, 0.0), 2.0),
    );
    let label = rustycad_tklcaf::tdf::TdfAttributeState::label(circle.get().unwrap());
    let mut object: Handle<dyn AisInteractiveObject> = Handle::default();
    assert!(
        TPrsStdGeometryDriver::new()
            .update(&label, &mut object)
            .unwrap()
    );
    assert!(!object.cast::<AisCircle>().is_null());
    assert_eq!(object.color().name(), NamedColor::RED);

    let empty = root.new_child().unwrap();
    assert!(
        !TPrsStdGeometryDriver::new()
            .update(&empty, &mut object)
            .unwrap()
    );
}

#[test]
fn named_shape_driver_shows_the_current_shape() {
    let document = support::new_document();
    let root = document.main().unwrap();
    let solid = support::make_box(1.0, 2.0, 3.0);
    let named = support::named_shape(&root, &solid);
    let label = rustycad_tklcaf::tdf::TdfAttributeState::label(named.get().unwrap());
    let mut object: Handle<dyn AisInteractiveObject> = Handle::default();
    assert!(
        TPrsStdNamedShapeDriver::new()
            .update(&label, &mut object)
            .unwrap()
    );
    let shape = object.cast::<AisShape>();
    assert!(shape.shape() == solid);
    let first = object.object_address();
    assert!(
        TPrsStdNamedShapeDriver::new()
            .update(&label, &mut object)
            .unwrap()
    );
    assert_eq!(object.object_address(), first);
    assert!(!object.is_infinite());
    let empty = root.new_child().unwrap();
    assert!(
        !TPrsStdNamedShapeDriver::new()
            .update(&empty, &mut object)
            .unwrap()
    );
}
