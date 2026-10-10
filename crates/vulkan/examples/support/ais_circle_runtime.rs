//! Every datum transition renders on the pinned device, with retained sensitive geometry checked.
use crate::{circle_scene, device_options};
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkv3d::ais::AisInteractiveObjectMembers;
pub fn check_circle_datums() {
    let driver = Graphic3dRenderDriver::new(device_options::validation_options()).unwrap();
    for mode in circle_scene::MODES {
        let scene = circle_scene::CircleScene::new(&driver, mode);
        scene.apply(mode).unwrap();
        scene.render_view().try_redraw().unwrap();
        let capture = scene.render_view().capture().unwrap();
        let visible = capture
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .any(|p| *p != [0, 0, 255, 255]);
        assert_eq!(visible, mode != "circle-erased", "{mode}");
        if mode.ends_with("detected") {
            assert_eq!(
                scene.context.detected_interactive().object_address(),
                scene.circle.object_address(),
                "{mode}"
            );
        }
        assert_eq!(scene.circle.get_context(), scene.context);
        scene.view.remove();
    }
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    println!("MARTENSITE_DEVICE={}", driver.device_name());
}
