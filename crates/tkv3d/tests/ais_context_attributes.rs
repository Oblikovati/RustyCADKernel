//! AIS_InteractiveContext selection mode activation, filters and default drawer members (the
//! TKV3d numeric probe v3_ais_ctx_attr compares the complete behaviour with native OCCT).
#[allow(dead_code)]
#[path = "support/sensitive_display_driver.rs"]
mod display;

use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
use rustycad_tkv3d::ais::{
    AisInteractiveContext, AisInteractiveObject, AisKindOfInteractive,
    AisSelectionModesConcurrency, AisShape, AisTypeFilter,
};
use rustycad_tkv3d::select_mgr::{SelectMgrFilter, SelectMgrFilterType};
use rustycad_tkv3d::v3d::V3dViewer;

fn scene() -> (
    Handle<V3dViewer>,
    Handle<AisInteractiveContext>,
    Handle<dyn AisInteractiveObject>,
) {
    let (driver, _trace) = display::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let context = AisInteractiveContext::new(&viewer).unwrap();
    let edge =
        BRepBuilderAPIMakeEdge::from_points(Point3::new(0., 0., 0.), Point3::new(1., 0., 0.))
            .unwrap()
            .edge()
            .unwrap();
    let object: Handle<dyn AisInteractiveObject> = AisShape::new(&Shape::from(edge)).into_cast();
    context.display(&object, false).unwrap();
    (viewer, context, object)
}

fn modes(context: &AisInteractiveContext, object: &Handle<dyn AisInteractiveObject>) -> Vec<i32> {
    let mut modes = List::default();
    context.activated_modes(object, &mut modes);
    modes.iter().copied().collect()
}

#[test]
fn selection_modes_follow_the_concurrency_policy() {
    let (_viewer, context, object) = scene();
    assert_eq!(modes(&context, &object), vec![0]);
    // A local mode replaces the global one (GlobalOrLocal).
    context.activate(&object, 2, false).unwrap();
    assert_eq!(modes(&context, &object), vec![2]);
    context
        .set_selection_mode_active(
            &object,
            1,
            true,
            AisSelectionModesConcurrency::Multiple,
            false,
        )
        .unwrap();
    assert_eq!(modes(&context, &object), vec![2, 1]);
    context.deactivate_mode(&object, 2).unwrap();
    assert_eq!(modes(&context, &object), vec![1]);
    context.deactivate(&object).unwrap();
    assert!(modes(&context, &object).is_empty());
}

#[test]
fn filters_and_default_drawer_members() {
    let (_viewer, context, _object) = scene();
    let filter: Handle<dyn SelectMgrFilter> =
        Handle::new(AisTypeFilter::new(AisKindOfInteractive::Datum)).into_cast();
    context.add_filter(&filter);
    assert_eq!(context.filters().len(), 1);
    context.set_filter_type(SelectMgrFilterType::And);
    assert_eq!(context.filter_type(), SelectMgrFilterType::And);
    context.remove_filter(&filter);
    assert!(context.filters().is_empty());
    context.set_plane_size_xy(10., 20., false).unwrap();
    assert_eq!(context.plane_size(), (false, 10., 20.));
    context.set_plane_size(5., false).unwrap();
    assert_eq!(context.plane_size(), (true, 5., 5.));
    // The tolerance map adds the custom tolerance to the largest sensitivity (3 for an edge).
    context.set_pixel_tolerance(7);
    assert_eq!(context.pixel_tolerance(), 10);
    context.enable_draw_hidden_line();
    assert!(context.draw_hidden_line());
}

#[test]
fn per_object_deviation_and_isolines_reach_the_shape() {
    use rustycad_tkv3d::ais::{AisShapeParameters, AisShapeSource};
    use rustycad_tkv3d::prs_mgr::PrsMgrPresentableObjectMembers;
    let (_viewer, context, object) = scene();
    let shape = object.clone().cast::<dyn AisShapeSource>();
    context
        .set_object_deviation_coefficient(&object, 0.02, false)
        .unwrap();
    assert_eq!(shape.own_deviation_coefficient().1, 0.02);
    assert!(shape.own_deviation_coefficient().0);
    context
        .set_object_deviation_angle(&object, 0.3, true)
        .unwrap();
    assert!(shape.own_deviation_angle().0);
    assert_eq!(shape.own_deviation_angle().1, 0.3);
    context.set_object_iso_on_triangulation(true, &object);
    assert!(object.attributes().iso_on_triangulation());
    // A null object is ignored by all three, as natively.
    let null = Handle::<dyn AisInteractiveObject>::default();
    context
        .set_object_deviation_coefficient(&null, 0.5, false)
        .unwrap();
    context
        .set_object_deviation_angle(&null, 0.5, false)
        .unwrap();
    context.set_object_iso_on_triangulation(true, &null);
}
