//! Native point object state, geometry, highlights and mode-independent selection.
#[allow(dead_code)]
#[path = "support/sensitive_display_driver.rs"]
mod display_driver;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::Handle,
};
use rustycad_tkg3d::geom::GeomCartesianPoint;
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::{aspect::AspectTypeOfMarker, graphic3d::*};
use rustycad_tkv3d::{
    ais::*, prs_mgr::*, prs3d::Prs3dDrawer, select_mgr::*, select3d::*, std_prs::*, v3d::V3dViewer,
};
fn point() -> Handle<AisPoint> {
    AisPoint::new(&Handle::new(GeomCartesianPoint::new(1., 2., 3.)).cast()).unwrap()
}
#[test]
fn identity_and_custom_highlights_preserve_native_defaults() {
    let point = point();
    assert_eq!(point.signature(), 1);
    assert_eq!(point.kind_of_interactive(), AisKindOfInteractive::Datum);
    for (drawer, color, layer) in [
        (
            point.hilight_attributes(),
            NamedColor::GRAY80,
            GRAPHIC3D_ZLAYER_ID_UNKNOWN,
        ),
        (
            point.dynamic_hilight_attributes(),
            NamedColor::CYAN1,
            GRAPHIC3D_ZLAYER_ID_TOP,
        ),
    ] {
        assert_eq!(drawer.display_mode(), -99);
        assert_eq!(drawer.color(), Color::named(color));
        assert_eq!(drawer.z_layer(), layer);
        assert_eq!(drawer.point_aspect().aspect().scale(), 3.);
        assert!(!drawer.has_link());
    }
    for mode in [-100, -99, -1, 0, 1, 2] {
        assert_eq!(point.accept_display_mode(mode), mode == 0 || mode == -99);
    }
}
#[test]
fn live_coordinates_drive_standard_matching_and_fresh_vertices() {
    let point = point();
    let first = point.vertex().unwrap();
    point
        .component()
        .cast::<GeomCartesianPoint>()
        .set_coord(4., 5., 6.);
    assert_eq!(StdPrsToolPoint::coord(&point.component()), (4., 5., 6.));
    assert!(StdPrsPoint::matches(&point.component(), 4., 5., 8., 2.));
    assert!(!StdPrsPoint::matches(&point.component(), 4., 5., 8., 1.999));
    let second = point.vertex().unwrap();
    assert_eq!(BRepTool::pnt(&first).unwrap(), Point3::new(1., 2., 3.));
    assert_eq!(BRepTool::pnt(&second).unwrap(), Point3::new(4., 5., 6.));
    let replacement = Handle::new(GeomCartesianPoint::new(-1., -2., -3.)).cast();
    point.set_component(&replacement);
    assert_eq!(point.component(), replacement);
}
#[test]
fn every_selection_mode_has_a_live_priority_ten_owner() {
    let point = point();
    for mode in [-99, -1, 0, 1, 100] {
        let selection = Handle::new(SelectMgrSelection::new(mode));
        point.compute_selection(&selection, mode).unwrap();
        assert_eq!(selection.entities().len(), 1);
        let sensitive = selection.entities()[0]
            .base_sensitive()
            .cast::<Select3DSensitivePoint>();
        assert_eq!(sensitive.point(), point.component().pnt());
        assert_eq!(sensitive.owner_id().priority(), 10);
        assert_eq!(
            sensitive.owner_id().selectable().object_address(),
            point.object_address()
        );
    }
}
#[test]
fn marker_precedence_and_width_refresh_preserve_inherited_state() {
    let point = point();
    let linked = Handle::new(Prs3dDrawer::new());
    linked.setup_own_defaults().unwrap();
    linked.point_aspect().set_scale(2.5).unwrap();
    linked
        .point_aspect()
        .set_type_of_marker(AspectTypeOfMarker::O);
    linked
        .point_aspect()
        .set_color(Color::named(NamedColor::GREEN));
    point.attributes().link_to(&linked);
    point.set_marker(AspectTypeOfMarker::Star).unwrap();
    point.set_color(Color::named(NamedColor::RED)).unwrap();
    point.set_width(4.).unwrap();
    assert_eq!(point.attributes().point_aspect().aspect().scale(), 2.5);
    point.unset_marker().unwrap();
    let aspect = point.attributes().point_aspect().aspect();
    assert_eq!(aspect.marker_kind(), AspectTypeOfMarker::O);
    assert_eq!(aspect.scale(), 4.);
    point.unset_color().unwrap();
    assert_eq!(aspect.interior_color(), Color::named(NamedColor::GREEN));
    point.unset_width().unwrap();
    point.unset_marker().unwrap();
    assert!(!point.attributes().has_own_point_aspect());
}
#[test]
fn negative_inherited_width_fails_after_marker_and_color_mutation() {
    let point = point();
    point.set_marker(AspectTypeOfMarker::Plus).unwrap();
    point.set_width(-2.).unwrap();
    let error = point.set_color(Color::named(NamedColor::RED)).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    assert!(point.has_color());
    assert_eq!(
        point.attributes().point_aspect().aspect().interior_color(),
        Color::named(NamedColor::RED)
    );
    assert_eq!(point.attributes().point_aspect().aspect().scale(), 1.);
}
#[test]
fn normal_and_highlight_presentations_follow_modes_and_infinite_flag() {
    let point = point();
    point.attributes().setup_own_defaults().unwrap();
    point.set_infinite_state(true);
    let (driver, trace) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    for mode in [0, -99, 1] {
        let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
        point
            .compute(&Handle::default(), &prs.cast(), mode)
            .unwrap();
        assert!(prs.is_infinite());
        assert_eq!(prs.groups().len(), usize::from(mode != 1));
    }
    assert!(!trace.read().events.is_empty());
}
