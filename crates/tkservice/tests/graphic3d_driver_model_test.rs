//! The Graphic3d driver model on the null driver: layers, structures, groups, the structure
//! manager and the view.
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::aspect::{AspectNeutralWindow, AspectWindow};
use rustycad_tkservice::graphic3d::{
    GRAPHIC3D_ZLAYER_ID_DEFAULT, GRAPHIC3D_ZLAYER_ID_TOP, Graphic3dCStructureMembers,
    Graphic3dCView, Graphic3dCViewComputed, Graphic3dCViewMembers, Graphic3dCViewStructures,
    Graphic3dDisplayPriority, Graphic3dGraphicDriver, Graphic3dGraphicDriverMembers,
    Graphic3dGroupMembers, Graphic3dStructure, Graphic3dStructureBounds, Graphic3dStructureDisplay,
    Graphic3dStructureManager, Graphic3dStructureMembers, Graphic3dStructureNetwork,
    Graphic3dStructureSource, Graphic3dTypeOfConnection, Graphic3dZLayerSettings,
};
use rustycad_tkservice::null_driver::{NullGraphicDriver, NullView};

struct Scene {
    driver: Handle<dyn Graphic3dGraphicDriver>,
    manager: Handle<Graphic3dStructureManager>,
    view: Handle<dyn Graphic3dCView>,
}

fn scene() -> Scene {
    let driver = NullGraphicDriver::new_handle();
    let manager = Handle::new(Graphic3dStructureManager::new(&driver).expect("manager"));
    let view = driver.create_view(&manager).expect("view");
    let window = AspectNeutralWindow::default();
    window.set_size(800, 600);
    let window: Handle<dyn AspectWindow> = Handle::new(window).cast();
    view.cast::<NullView>().set_null_window(&window);
    view.activate();
    Scene {
        driver,
        manager,
        view,
    }
}

fn structure_with_box(scene: &Scene, bounds: [f64; 6]) -> Handle<dyn Graphic3dStructureSource> {
    let structure: Handle<dyn Graphic3dStructureSource> =
        Graphic3dStructure::new(&scene.manager, &Handle::default())
            .expect("structure")
            .cast();
    let group = structure.new_group().expect("group");
    group.set_min_max_values(bounds);
    structure
}

#[test]
fn driver_layers_and_identifiers() {
    let scene = scene();
    assert_eq!(scene.driver.z_layers(), vec![-5, 0, -2, -3, -4]);
    let settings = Graphic3dZLayerSettings::default();
    scene
        .driver
        .insert_layer_after(1, &settings, GRAPHIC3D_ZLAYER_ID_DEFAULT)
        .expect("insert");
    assert_eq!(scene.driver.z_layers(), vec![-5, 0, 1, -2, -3, -4]);
    assert!(scene.driver.insert_layer_after(1, &settings, 0).is_err());
    assert!(scene.driver.z_layer_settings(7).is_err());
    assert_eq!(scene.view.identification(), 0);
    assert_eq!(scene.manager.max_num_of_views(), 32);
    assert_eq!(scene.driver.new_identification().expect("id"), 0);
}

#[test]
fn display_erase_and_bounds() {
    let scene = scene();
    let structure = structure_with_box(&scene, [0.0, 0.0, 0.0, 1.0, 2.0, 3.0]);
    structure.display();
    assert!(scene.view.is_displayed(&structure));
    let default_layer = scene.view.layer(GRAPHIC3D_ZLAYER_ID_DEFAULT);
    assert_eq!(default_layer.nb_structures(), 1);
    let bounds = scene.view.min_max_values(false).expect("bounds");
    assert_eq!(bounds.corner_max().expect("max").z(), 3.0);
    assert_eq!(
        structure
            .min_max_values(false)
            .corner_min()
            .expect("min")
            .x(),
        0.0
    );
    structure
        .set_display_priority(Graphic3dDisplayPriority::Above)
        .expect("priority");
    assert_eq!(
        default_layer
            .structures(Graphic3dDisplayPriority::Above)
            .extent(),
        1
    );
    structure.set_z_layer(GRAPHIC3D_ZLAYER_ID_TOP);
    assert_eq!(
        (
            default_layer.nb_structures(),
            scene.view.layer(GRAPHIC3D_ZLAYER_ID_TOP).nb_structures()
        ),
        (0, 1)
    );
    structure.erase();
    assert!(
        !scene.view.is_displayed(&structure) && scene.manager.number_of_displayed_structures() == 0
    );
    assert!(
        structure
            .set_display_priority(Graphic3dDisplayPriority::Invalid)
            .is_err()
    );
    assert_eq!(
        scene
            .view
            .consider_zoom_persistence_objects()
            .expect("coef"),
        1.0
    );
}

#[test]
fn network_groups_and_removal() {
    let scene = scene();
    let parent = structure_with_box(&scene, [0.0; 6]);
    let child = structure_with_box(&scene, [5.0, 5.0, 5.0, 6.0, 6.0, 6.0]);
    parent.connect(&child, Graphic3dTypeOfConnection::Descendant, true);
    let mut set = Vec::new();
    parent.descendants(&mut set);
    assert_eq!(set.len(), 1);
    assert_eq!(parent.cstructure().bounding_box().corner_max().x, 6.0);
    child.connect(&parent, Graphic3dTypeOfConnection::Descendant, true);
    set.clear();
    child.descendants(&mut set);
    assert!(set.is_empty());
    parent.disconnect(&child);
    let group = parent.groups()[0].clone();
    group.remove();
    assert!(group.is_deleted() && parent.number_of_groups() == 0);
    let id = child.identification();
    child.remove();
    assert!(child.is_deleted() && child.identification() == -1);
    assert_eq!(scene.driver.new_identification().expect("reuse"), id);
}
