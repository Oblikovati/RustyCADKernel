//! Owned GUI packets stay outside model structures and follow the complete model frame.
use crate::{RenderError, RenderLayer, error::rejected};
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct GuiLayerPacket {
    pub extent: [u32; 2],
    pub layer: RenderLayer,
}
pub(crate) struct GuiLayerSlot {
    pub packet: Option<GuiLayerPacket>,
    pub stale: bool,
    pub epoch: Arc<()>,
}
impl Default for GuiLayerSlot {
    fn default() -> Self {
        Self {
            packet: None,
            stale: false,
            epoch: Arc::new(()),
        }
    }
}
impl GuiLayerSlot {
    pub fn replace(&mut self, packet: Option<GuiLayerPacket>) -> Option<GuiLayerPacket> {
        self.stale = false;
        self.epoch = Arc::new(());
        std::mem::replace(&mut self.packet, packet)
    }
    pub fn retire_removed(&mut self) -> Option<GuiLayerPacket> {
        self.packet.as_ref()?;
        self.replace(None)
    }
    pub fn resized(&mut self) -> bool {
        if self.packet.is_none() {
            return false;
        }
        self.stale = true;
        self.epoch = Arc::new(());
        true
    }
    pub fn snapshot(&self, extent: [u32; 2]) -> Result<Option<GuiLayerPacket>, RenderError> {
        let Some(packet) = &self.packet else {
            return Ok(None);
        };
        if self.stale || packet.extent != extent {
            return Err(rejected(
                (packet.extent, extent, self.stale),
                "a fresh GUI packet matching the current target extent",
            ));
        }
        Ok(Some(packet.clone()))
    }
}
pub(crate) fn validate_gui_layer(layer: &RenderLayer) -> Result<(), RenderError> {
    let policy = (layer.depth_test, layer.depth_write, layer.clear_depth);
    if policy != (false, false, false) {
        return Err(rejected(
            policy,
            "GUI depth test, write and clear all disabled",
        ));
    }
    if layer
        .draws
        .iter()
        .any(crate::PrimitiveDraw::is_surface_blended)
    {
        return Err(rejected(
            "deferred surface blend",
            "ordered GUI draws outside model OIT",
        ));
    }
    crate::capping_pass::validate_capping_sequence(&layer.draws)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PrimitiveDraw, RasterImage, RenderVertex, TexturedStyle, identity_clip};
    fn font_packet() -> (GuiLayerPacket, std::sync::Weak<Vec<[f32; 4]>>) {
        let image = RasterImage::new([1, 1], vec![[1.; 4]]).unwrap();
        let weak = Arc::downgrade(&image.pixels);
        let style = TexturedStyle::new(image, vec![[0.; 2]; 3], 0.).unwrap();
        let vertices = vec![RenderVertex::new([0.; 3], [1.; 4]); 3];
        let draw =
            PrimitiveDraw::textured(vertices, vec![0, 1, 2], identity_clip(), style).unwrap();
        let layer = RenderLayer::new(vec![draw], false, false, false);
        (
            GuiLayerPacket {
                extent: [64, 64],
                layer,
            },
            weak,
        )
    }
    #[test]
    fn snapshot_owns_font_after_replacement_until_its_completion() {
        let (packet, weak) = font_packet();
        let mut slot = GuiLayerSlot::default();
        assert!(slot.snapshot([64, 64]).unwrap().is_none());
        slot.replace(Some(packet));
        let snapshot = slot.snapshot([64, 64]).unwrap();
        drop(slot.replace(None));
        assert!(weak.upgrade().is_some());
        drop(snapshot);
        assert!(weak.upgrade().is_none());
    }
    #[test]
    fn epoch_and_extent_checks_cover_clear_resize_and_replacement() {
        let (packet, _) = font_packet();
        let mut slot = GuiLayerSlot::default();
        let initial = slot.epoch.clone();
        assert!(!slot.resized());
        assert!(slot.retire_removed().is_none());
        slot.replace(Some(packet.clone()));
        assert!(!Arc::ptr_eq(&initial, &slot.epoch));
        assert!(slot.snapshot([80, 64]).is_err());
        assert!(slot.resized());
        assert!(slot.snapshot([64, 64]).is_err());
        slot.replace(Some(packet));
        assert!(slot.snapshot([64, 64]).is_ok());
        assert!(slot.retire_removed().is_some());
    }
    #[test]
    fn invalid_depth_or_deferred_surface_policy_is_refused() {
        let (packet, _) = font_packet();
        assert!(validate_gui_layer(&packet.layer).is_ok());
        for policy in [
            (true, false, false),
            (false, true, false),
            (false, false, true),
        ] {
            let mut layer = packet.layer.clone();
            (layer.depth_test, layer.depth_write, layer.clear_depth) = policy;
            assert!(validate_gui_layer(&layer).is_err());
        }
        assert_deferred_gui_rejected();
    }
    fn assert_deferred_gui_rejected() {
        let vertices = vec![RenderVertex::new([0.; 3], [1.; 4]); 3];
        let draw = PrimitiveDraw::new(vertices, vec![0, 1, 2], identity_clip()).unwrap();
        let deferred = draw.with_surface_blending(true).unwrap();
        let layer = RenderLayer::new(vec![deferred], false, false, false);
        assert!(validate_gui_layer(&layer).is_err());
    }
}
