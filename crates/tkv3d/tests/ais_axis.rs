//! Native axis identity, snapshot selection and observable appearance mutation order.
#[allow(dead_code)]
#[path = "support/ais_line_fixture.rs"]
mod fixture;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::Handle,
    units_api::SystemUnits,
};
use rustycad_tkg3d::geom::{GeomAxis1Placement, GeomAxis2Placement};
use rustycad_tkmath::gp::{Axis1, Axis2, Point3};
use rustycad_tkservice::{aspect::AspectTypeOfLine, graphic3d::*};
use rustycad_tkv3d::{ais::*, prs_mgr::*, prs3d::*, select_mgr::*, select3d::*};
struct SelectedAxis {
    _selection: Handle<SelectMgrSelection>,
    segment: Handle<Select3DSensitiveSegment>,
}
impl std::ops::Deref for SelectedAxis {
    type Target = Select3DSensitiveSegment;
    fn deref(&self) -> &Self::Target {
        &self.segment
    }
}
fn selected(axis: &AisAxis, mode: i32) -> SelectedAxis {
    let selection = Handle::new(SelectMgrSelection::new(mode));
    axis.compute_selection(&selection, mode).unwrap();
    assert_eq!(selection.entities().len(), 1);
    let segment = selection.entities()[0].base_sensitive().cast();
    SelectedAxis {
        _selection: selection,
        segment,
    }
}
fn coordinate_axis() -> Handle<AisAxis> {
    let axis = AisAxis::from_axis2(
        &Handle::new(GeomAxis2Placement::new(Axis2::default())),
        AisTypeOfAxis::XAxis,
        fixture::units(),
    )
    .unwrap();
    let linked = Handle::new(Prs3dDrawer::new());
    linked.setup_own_defaults().unwrap();
    axis.attributes().link_to(&linked);
    axis
}
#[test]
fn infinite_axis_snapshots_units_and_geometry_until_setter_is_called() {
    let units = fixture::units();
    let component = fixture::component();
    let axis = AisAxis::new(&component, units.clone()).unwrap();
    assert_eq!(axis.signature(), 2);
    assert_eq!(axis.kind_of_interactive(), AisKindOfInteractive::Datum);
    assert!(axis.is_infinite());
    assert!(!axis.is_xyz_axis());
    assert!(axis.axis2_placement().is_null());
    let first = selected(&axis, 0);
    assert!((first.start_point().distance(&first.end_point()) - 500.).abs() < 1e-10);
    units.write().set_local_system(SystemUnits::Mdtv).unwrap();
    assert_eq!(selected(&axis, 12).end_point(), first.end_point());
    axis.set_component(&component).unwrap();
    assert!(
        (selected(&axis, -1)
            .start_point()
            .distance(&selected(&axis, -1).end_point())
            - 500000.)
            .abs()
            < 1e-9
    );
}
#[test]
fn coordinate_axis_setters_keep_native_cached_fields_and_owner_identity() {
    let axis = coordinate_axis();
    assert!(axis.is_xyz_axis());
    assert_eq!(axis.type_of_axis(), AisTypeOfAxis::XAxis);
    let frame = axis.axis2_placement();
    let initial = selected(&axis, 0);
    assert_eq!(initial.owner_id().priority(), 3);
    assert_eq!(
        initial.owner_id().selectable().object_address(),
        axis.object_address()
    );
    axis.set_type_of_axis(AisTypeOfAxis::YAxis);
    assert_eq!(selected(&axis, 0).end_point(), initial.end_point());
    axis.attributes().datum_aspect().set_axis_length(2., 3., 4.);
    axis.set_axis2_placement(&frame, AisTypeOfAxis::YAxis)
        .unwrap();
    assert_eq!(selected(&axis, 0).end_point(), Point3::new(0., 3., 0.));
    axis.set_axis2_placement(&frame, AisTypeOfAxis::ZAxis)
        .unwrap();
    assert_eq!(selected(&axis, 0).end_point(), Point3::new(0., 0., 4.));
    axis.set_axis2_placement(&frame, AisTypeOfAxis::Unknown)
        .unwrap();
    assert_eq!(selected(&axis, 0).end_point(), Point3::default());
}
#[test]
fn ray_and_axis1_constructors_preserve_modes_and_native_length_validation() {
    for length in [0., -2.] {
        let result = AisAxis::from_ray(&Axis1::default(), length, fixture::units());
        assert_eq!(
            result.err().unwrap().exception_type(),
            "Standard_NumericError"
        );
    }
    for (length, expected) in [(-1., 250.), (4., 4.)] {
        let axis = AisAxis::from_ray(&Axis1::default(), length, fixture::units()).unwrap();
        assert!(axis.is_infinite() && axis.is_xyz_axis());
        assert!(!axis.attributes().datum_aspect().to_draw_labels());
        assert_eq!(
            selected(&axis, 3).end_point(),
            Point3::new(0., 0., expected)
        );
        assert!(axis.accept_display_mode(0));
        assert!(!axis.accept_display_mode(1));
    }
    let placement = Handle::new(GeomAxis1Placement::new(Axis1::default()));
    let axis = AisAxis::from_axis1(&placement, fixture::units()).unwrap();
    assert_eq!(axis.component().position(), placement.ax1());
    let xyz = coordinate_axis();
    let retained_frame = xyz.axis2_placement();
    xyz.set_axis1_placement(&placement).unwrap();
    assert!(!xyz.is_xyz_axis());
    assert_eq!(xyz.axis2_placement(), retained_frame);
}
#[test]
fn native_width_zero_resets_then_throws_without_own_width_assignment() {
    let axis = coordinate_axis();
    axis.set_width(4.).unwrap();
    assert!(!axis.has_width());
    assert_eq!(axis.attributes().line_aspect().aspect().width(), 4.);
    axis.set_width(-2.).unwrap();
    assert_eq!(axis.attributes().line_aspect().aspect().width(), 4.);
    assert!(axis.set_width(0.).is_err());
    assert_eq!(axis.attributes().line_aspect().aspect().width(), 1.);
    assert_eq!(
        axis.attributes()
            .datum_aspect()
            .line_aspect(Prs3dDatumParts::XAxis)
            .aspect()
            .width(),
        1.
    );
    axis.set_width(3.).unwrap();
    axis.unset_width().unwrap();
    assert_eq!(axis.width(), 0.);
}
#[test]
fn display_aspect_and_color_reset_keep_native_handle_and_stale_drawer_color() {
    let axis = coordinate_axis();
    let custom = Handle::new(
        Prs3dLineAspect::new(Color::named(NamedColor::GREEN), AspectTypeOfLine::Dash, 2.).unwrap(),
    );
    axis.set_display_aspect(&custom).unwrap();
    assert_eq!(axis.attributes().line_aspect(), custom);
    assert!(axis.has_color());
    axis.set_color(Color::named(NamedColor::BLUE)).unwrap();
    axis.unset_color().unwrap();
    assert!(!axis.has_color());
    assert_eq!(axis.color(), Color::named(NamedColor::BLUE));
    assert_eq!(
        custom.aspect().interior_color(),
        Color::named(NamedColor::RED)
    );
    assert_eq!(
        axis.attributes()
            .datum_aspect()
            .line_aspect(Prs3dDatumParts::YAxis)
            .aspect()
            .interior_color(),
        Color::named(NamedColor::TURQUOISE)
    );
}
