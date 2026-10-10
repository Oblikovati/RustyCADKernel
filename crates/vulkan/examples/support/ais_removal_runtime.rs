//! Destructive AIS transitions are checked on completed frames, independently of retained redraw.
use super::{
    ais_runtime::{ais_lifecycle, scene},
    device_options,
};
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkv3d::prs_mgr::PrsMgrPresentableObjectMembers;
pub fn check_ais_removal() {
    let driver = Graphic3dRenderDriver::new(device_options::validation_options()).unwrap();
    for (mode, visible) in [
        ("ais-cleared", false),
        ("ais-removed", false),
        ("ais-recreated", true),
        ("ais-erased-all", false),
        ("ais-displayed-all", true),
        ("ais-removed-all", false),
        ("ais-erased-selected", false),
        ("ais-displayed-selected", true),
    ] {
        let scene = scene::AisViewerScene::new(&driver);
        ais_lifecycle::apply_capture_transition(&scene, mode).unwrap();
        scene.render_view().try_redraw().unwrap();
        let color = if visible {
            [255, 0, 0, 255]
        } else {
            [0, 0, 255, 255]
        };
        assert_eq!(
            scene.render_view().capture().unwrap().pixel(32, 32),
            Some(color),
            "{mode}"
        );
        scene.context.remove_all(false).unwrap();
        assert!(scene.object.presentations().is_empty());
        scene.view.remove();
    }
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
}
