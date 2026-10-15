//! AIS_Shape: presentation modes, selection, parameters and the shared RTTI of the AIS_Shape
//! family (the TKV3d numeric probe v3_ais_shape compares the complete behaviour natively).
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
use rustycad_tkv3d::ais::{
    AisInteractiveObject, AisKindOfInteractive, AisShape, AisShapeParameters, AisShapeSource,
    ais_shape_selection_mode, ais_shape_selection_type,
};
use rustycad_tkv3d::prs_mgr::{PrsMgrPresentableObject, PrsMgrPresentableObjectMembers};
use rustycad_tkv3d::select_mgr::{SelectMgrSelectableObject, SelectMgrSelection};

fn edge() -> Shape {
    Shape::from(
        BRepBuilderAPIMakeEdge::from_points(Point3::new(0., 0., 0.), Point3::new(2., 1., 0.))
            .unwrap()
            .edge()
            .unwrap(),
    )
}

#[test]
fn shape_identity_and_display_modes() {
    let object = AisShape::new(&edge());
    assert_eq!(object.kind_of_interactive(), AisKindOfInteractive::Shape);
    assert_eq!(object.signature(), 0);
    assert!(object.accept_display_mode(2) && !object.accept_display_mode(3));
    assert!(object.accept_shape_decomposition());
    // The AIS_Shape interface shares the concrete class descriptor.
    assert!(std::ptr::eq(
        <dyn AisShapeSource as RuntimeTyped>::static_type(),
        AisShape::static_type()
    ));
    assert!(!object.clone().cast::<dyn AisShapeSource>().is_null());
}

#[test]
fn shape_bounding_box_and_parameters() {
    let object = AisShape::new(&edge());
    let bounds = object.shape_bounding_box().unwrap();
    assert!(!bounds.is_void());
    object.set_texture_repeat_uv(Point2::new(2., 3.));
    assert_eq!(object.texture_repeat_uv(), Point2::new(2., 3.));
    object.set_own_deviation_coefficient(0.01);
    assert!(object.own_deviation_coefficient().0);
    assert!(object.reset_own_deviation_coefficient());
    assert!(!object.own_deviation_coefficient().0);
    assert!(object.user_angle().is_err());
    object.set_angle_and_deviation(0.25);
    assert_eq!(object.user_angle().unwrap(), 0.25);
}

#[test]
fn shape_selection_of_an_edge() {
    let object = AisShape::new(&edge());
    object.attributes().setup_own_defaults().unwrap();
    let selection = Handle::new(SelectMgrSelection::new(2));
    object.compute_selection(&selection, 2).unwrap();
    assert_eq!(selection.entities().len(), 1);
    assert_eq!(ais_shape_selection_mode(ShapeEnum::Face), 4);
    assert_eq!(ais_shape_selection_type(2), ShapeEnum::Edge);
}

#[test]
fn protected_drawer_helpers_apply_to_the_given_drawer() {
    use rustycad_tkernel::quantity::{Color, NamedColor};
    use rustycad_tkservice::graphic3d::{
        Graphic3dAspectsLineMembers as _, Graphic3dAspectsMembers as _,
    };
    use rustycad_tkv3d::ais::{
        set_shape_drawer_color, set_shape_drawer_transparency, set_shape_drawer_width,
    };
    let object = AisShape::new(&edge());
    let drawer = object.attributes();
    // The first width call creates the own line aspects (a recompute), the second reuses them.
    assert!(set_shape_drawer_width(&drawer, 3.0).unwrap());
    assert!(!set_shape_drawer_width(&drawer, 4.0).unwrap());
    assert_eq!(drawer.wire_aspect().aspect().line_width(), 4.0);
    let red = Color::named(NamedColor::RED);
    set_shape_drawer_color(&*object, &drawer, red).unwrap();
    assert_eq!(drawer.shading_aspect().aspect().interior_color(), red);
    set_shape_drawer_transparency(&*object, &drawer, 0.5).unwrap();
    assert!(drawer.has_own_shading_aspect());
}
