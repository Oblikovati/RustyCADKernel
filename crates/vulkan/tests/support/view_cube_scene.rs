//! Actual AIS context display keeps native Topmost and triedron persistence defaults.
use super::view_setup;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::{
    aspect::AspectTypeOfTriedronPosition,
    graphic3d::{Graphic3dTransModeFlags, Graphic3dTransformPersMembers},
};
use rustycad_tkv3d::{
    ais::*,
    prs_mgr::{PrsMgrDisplayStatus, PrsMgrPresentableObjectMembers},
    v3d::*,
};
pub struct ViewCubeCaptureScene {
    pub view: Handle<V3dView>,
    pub context: Handle<AisInteractiveContext>,
    pub cube: Handle<AisViewCube>,
}
impl ViewCubeCaptureScene {
    /// Display native defaults through a real context; `ViewCubeCaptureScene::new(&driver)?`.
    pub fn new(driver: &Handle<Graphic3dRenderDriver>) -> Result<Self, Failure> {
        let viewer = V3dViewer::new(&driver.cast())?;
        let view = cube_capture_view(&viewer)?;
        let context = AisInteractiveContext::new(&viewer)?;
        let cube = AisViewCube::new()?;
        context.display_with_modes(&cube.cast(), 0, 0, false, PrsMgrDisplayStatus::Displayed)?;
        let scene = Self {
            view,
            context,
            cube,
        };
        scene.assert_display_defaults();
        Ok(scene)
    }
}

impl ViewCubeCaptureScene {
    /// Check retained native display policy; `scene.assert_display_defaults()`.
    pub fn assert_display_defaults(&self) {
        assert!(self.context.is_displayed(&self.cube.cast()));
        assert_eq!(
            self.cube.z_layer(),
            rustycad_tkservice::graphic3d::GRAPHIC3D_ZLAYER_ID_TOPMOST
        );
        assert_cube_persistence(&self.cube.transform_persistence());
    }
}

fn cube_capture_view(viewer: &V3dViewer) -> Result<Handle<V3dView>, Failure> {
    let view = viewer.create_view()?;
    let window = view_setup::configure_view(&view);
    window.set_size(320, 320);
    view.must_be_resized();
    view.camera().set_eye(&Point3::new(10., 8., 20.))?;
    Ok(view)
}

impl ViewCubeCaptureScene {
    /// Clear only label strings through the live context; `scene.hide_labels()?`.
    pub fn hide_labels(&self) -> Result<(), Failure> {
        let empty = rustycad_tkernel::tcollection::AsciiString::default();
        for side in &V3dTypeOfOrientation::ALL[..6] {
            self.cube.set_box_side_label(*side, &empty)?;
        }
        self.cube.set_axes_labels(&empty, &empty, &empty);
        self.context
            .recompute_prs_only(&self.cube.cast(), false, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn real_context_extracts_geometry_labels_and_live_label_removal() {
        let device = crate::recording_device::RecordingDevice::default();
        let fonts = std::sync::Arc::new(crate::fixed_glyph_source::FixedGlyphSource::default());
        let driver =
            Graphic3dRenderDriver::with_backend_and_fonts(device.clone(), 512, fonts).unwrap();
        let scene = ViewCubeCaptureScene::new(&driver).unwrap();
        let render = scene.render_view();
        render.try_redraw().unwrap();
        assert_labeled_frame(&device.last_frame());
        scene.hide_labels().unwrap();
        render.try_redraw().unwrap();
        assert_unlabeled_frame(&device.last_frame());
        assert_restored_labels(&scene);
        scene.view.remove();
    }
    fn assert_restored_labels(scene: &ViewCubeCaptureScene) {
        scene.restore_labels().unwrap();
        scene.enable_lights().unwrap();
        scene.render_view().try_redraw().unwrap();
        scene.assert_display_defaults();
    }
    fn assert_labeled_frame(frame: &martensite_vulkan::RenderFrame) {
        assert!(
            frame
                .draws()
                .iter()
                .any(|draw| draw.textured_style().is_some())
        );
        assert!(frame.draws().iter().any(|draw| draw.vertices().len() > 100));
    }
    fn assert_unlabeled_frame(frame: &martensite_vulkan::RenderFrame) {
        assert!(
            frame
                .draws()
                .iter()
                .all(|draw| draw.textured_style().is_none())
        );
    }
}

impl ViewCubeCaptureScene {
    /// Borrow the real retained render view; `scene.render_view().try_redraw()?`.
    pub fn render_view(&self) -> Handle<martensite_vulkan::Graphic3dRenderView> {
        self.view.get().unwrap().view().cast()
    }
}

impl ViewCubeCaptureScene {
    /// Restore native label strings without changing geometry; `scene.restore_labels()?`.
    pub fn restore_labels(&self) -> Result<(), Failure> {
        let defaults = AisViewCube::new()?;
        for side in &V3dTypeOfOrientation::ALL[..6] {
            self.cube
                .set_box_side_label(*side, &defaults.box_side_label(*side))?;
        }
        self.cube.set_axes_labels(
            &defaults.axis_label(rustycad_tkv3d::prs3d::Prs3dDatumParts::XAxis),
            &defaults.axis_label(rustycad_tkv3d::prs3d::Prs3dDatumParts::YAxis),
            &defaults.axis_label(rustycad_tkv3d::prs3d::Prs3dDatumParts::ZAxis),
        );
        self.context
            .recompute_prs_only(&self.cube.cast(), false, false)
    }
    /// Illuminate unchanged cube defaults using the viewer's native light preset; `scene.enable_lights()?`.
    pub fn enable_lights(&self) -> Result<(), Failure> {
        self.context.current_viewer().set_default_lights()?;
        self.context.current_viewer().set_light_on()
    }
}

fn assert_cube_persistence(
    persistence: &Handle<dyn rustycad_tkservice::graphic3d::Graphic3dTransformPersSource>,
) {
    assert_eq!(persistence.mode(), Graphic3dTransModeFlags::TRIEDRON_PERS);
    assert_eq!(
        persistence.corner_2d().unwrap(),
        AspectTypeOfTriedronPosition::LEFT_LOWER
    );
    let offset = persistence.offset_2d().unwrap();
    assert_eq!((offset.x(), offset.y()), (100, 100));
}
