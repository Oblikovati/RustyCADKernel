//! AIS_ConnectedInteractive and AIS_MultipleConnectedInteractive connection rules (the TKV3d
//! numeric probe v3_ais_connected compares presentations and selections with native OCCT).
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::{Point3, Transform3, Vector3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
use rustycad_tkv3d::ais::{
    AisConnectedInteractive, AisInteractiveObject, AisKindOfInteractive,
    AisMultipleConnectedInteractive, AisShape,
};
use rustycad_tkv3d::prs_mgr::{PrsMgrPresentableObjectMembers, PrsMgrTypeOfPresentation3d};

fn edge() -> Handle<dyn AisInteractiveObject> {
    let edge =
        BRepBuilderAPIMakeEdge::from_points(Point3::new(0., 0., 0.), Point3::new(1., 0., 0.))
            .unwrap()
            .edge()
            .unwrap();
    AisShape::new(&Shape::from(edge)).into_cast()
}

#[test]
fn connected_instances_share_the_reference() {
    let reference = edge();
    let first = AisConnectedInteractive::new(PrsMgrTypeOfPresentation3d::AllView);
    let mut trsf = Transform3::default();
    trsf.set_translation(Vector3::new(1., 2., 3.));
    first.connect_trsf(&reference, &trsf).unwrap();
    assert!(first.has_connection());
    assert_eq!(first.kind_of_interactive(), AisKindOfInteractive::Object);
    assert!(first.has_transformation());
    // An instance of an instance refers to the original object.
    let second = AisConnectedInteractive::new(PrsMgrTypeOfPresentation3d::AllView);
    second.connect(&first.clone().into_cast()).unwrap();
    assert_eq!(
        second.connected_to().object_address(),
        reference.object_address()
    );
    // An assembly has no own presentations and cannot be connected.
    let assembly: Handle<dyn AisInteractiveObject> =
        AisMultipleConnectedInteractive::new().into_cast();
    let third = AisConnectedInteractive::new(PrsMgrTypeOfPresentation3d::AllView);
    let failure = third.connect(&assembly).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ProgramError");
    assert!(!third.has_connection());
}

#[test]
fn assemblies_copy_nested_assemblies_and_share_the_owner() {
    let assembly = AisMultipleConnectedInteractive::new();
    assert!(!assembly.has_connection());
    assert!(!assembly.has_own_presentations());
    let child = assembly.connect(&edge()).unwrap();
    assert!(assembly.has_connection());
    assert!(child.is_kind_name("AIS_ConnectedInteractive"));
    let outer = AisMultipleConnectedInteractive::new();
    let copy = outer.connect(&assembly.clone().into_cast()).unwrap();
    let copy = copy.cast::<AisMultipleConnectedInteractive>();
    assert_eq!(copy.children().len(), 1);
    assert_eq!(
        copy.fields.read().assembly_owner.object_address(),
        outer.fields.read().assembly_owner.object_address()
    );
    assembly.disconnect_all().unwrap();
    assert!(!assembly.has_connection());
}
