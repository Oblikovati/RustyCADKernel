//! AIS_Manipulator construction, part sizes, attachment and an interactive translation.
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::GeomCartesianPoint;
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::aspect::AspectNeutralWindow;
use rustycad_tkservice::graphic3d::{
    GRAPHIC3D_ZLAYER_ID_TOPMOST, Graphic3dTransModeFlags, Graphic3dTransformPersMembers,
};
use rustycad_tkservice::null_driver::NullGraphicDriver;
use rustycad_tkv3d::ais::*;
use rustycad_tkv3d::prs_mgr::*;
use rustycad_tkv3d::v3d::{V3dTypeOfOrientation, V3dViewSource, V3dViewer};

#[test]
fn construction_matches_the_native_defaults() {
    let m = AisManipulator::new().unwrap();
    assert!(!m.is_attached() && !m.has_active_mode());
    assert_eq!(m.active_axis_index(), -1);
    assert!(m.zoom_persistence());
    assert_eq!(m.skin_mode(), AisManipulatorSkin::Shaded);
    assert_eq!(m.z_layer(), GRAPHIC3D_ZLAYER_ID_TOPMOST);
    assert!(m.is_infinite());
    // SetSize(100) keeps the 0.2 indent: length 0.8 * 99.6, box 0.15 * 99.6, disk 0.05 * 99.6.
    let axis = m.fields.read().axes[0].clone();
    assert_eq!(axis.axis_length(), 99.6f32 * 0.8);
    assert_eq!(axis.box_size(), 99.6f32 * 0.15);
    assert_eq!(m.size(), axis.inner_radius + axis.disk_thickness + 0.4);
    let options = AisManipulatorOptionsForAttach::default();
    assert!(options.adjust_position && !options.adjust_size && options.enable_modes);
    let behavior = AisManipulatorBehaviorOnTransform::default();
    assert!(behavior.follow_translation && behavior.follow_rotation && behavior.follow_dragging);
}

#[test]
fn parts_reject_axis_indexes_outside_the_three_axes() {
    let m = AisManipulator::new().unwrap();
    let failure = m
        .set_part(3, AisManipulatorMode::Rotation, false)
        .unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ProgramError");
    m.set_part(1, AisManipulatorMode::Rotation, false).unwrap();
    assert!(!m.fields.read().axes[1].has_rotation());
    m.set_part_all(AisManipulatorMode::Scaling, false);
    assert!(m.fields.read().axes.iter().all(|a| !a.has_scaling()));
    // An unattached manipulator has no objects: a null dereference natively.
    assert_eq!(
        m.object().unwrap_err().exception_type(),
        "Standard_NullObject"
    );
}

#[test]
fn attached_translation_moves_the_object_and_the_manipulator() {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let view: Handle<dyn V3dViewSource> = viewer.create_view().unwrap();
    let window = Handle::new(AspectNeutralWindow::default());
    window.set_size(400, 300);
    view.set_window(&window.cast(), 0).unwrap();
    let ctx = AisInteractiveContext::new(&viewer).unwrap();
    let object: Handle<dyn AisInteractiveObject> =
        AisPoint::new(&Handle::new(GeomCartesianPoint::new(10.0, 20.0, 0.0)).cast())
            .unwrap()
            .into_cast();
    ctx.display(&object, false).unwrap();
    view.set_proj_orientation(V3dTypeOfOrientation::ZUP_TOP, false)
        .unwrap();
    view.fit_all(0.1, false).unwrap();
    let m = AisManipulator::new().unwrap();
    m.attach(&object, &AisManipulatorOptionsForAttach::default())
        .unwrap();
    assert!(m.is_attached());
    assert_eq!(m.position().location(), Point3::new(10.0, 20.0, 0.0));
    let pers = m.transform_persistence();
    assert_eq!(pers.mode(), Graphic3dTransModeFlags::AXIAL_ZOOM_PERS);
    // Selecting the X translation part activates it; dragging along X translates.
    m.fields.write().current_index = 0;
    m.fields.write().current_mode = AisManipulatorMode::Translation;
    let (px, py) = view.convert_world_to_pixels(10.0, 20.0, 0.0).unwrap();
    m.start_transform(px, py, &view).unwrap();
    assert!(m.has_active_transformation());
    let trsf = m.transform_at(px + 20, py, &view).unwrap();
    let moved = trsf.translation_part();
    assert!(moved.x() > 0.0 && moved.y().abs() < 1e-9);
    assert_eq!(
        object.local_transformation().translation_part().x(),
        moved.x()
    );
    assert!((m.position().location().x() - (10.0 + moved.x())).abs() < 1e-9);
    m.stop_transform(false).unwrap();
    assert_eq!(object.local_transformation().translation_part().x(), 0.0);
    assert_eq!(m.position().location(), Point3::new(10.0, 20.0, 0.0));
}
