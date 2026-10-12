//! Actual inherited clipping fields enter the renderer through ordinary public V3d setters.
use super::ViewerScene;
use rustycad_tkernel::{ncollection::Vec4, standard::Handle};
use rustycad_tkservice::graphic3d::*;
struct RenderClipDerivedPlane {
    base: Graphic3dClipPlane,
}
rustycad_tkernel::impl_transient!(RenderClipDerivedPlane,
    "Martensite_RenderClipDerivedPlane", Graphic3dClipPlane, base,
    interfaces: [dyn Graphic3dClipPlaneSource]);
impl Graphic3dClipPlaneSource for RenderClipDerivedPlane {
    fn clip_plane_base(&self) -> &Graphic3dClipPlane {
        &self.base
    }
}
struct RenderClipDerivedSequence {
    base: Graphic3dSequenceOfHClipPlane,
}
rustycad_tkernel::impl_transient!(RenderClipDerivedSequence,
    "Martensite_RenderClipDerivedSequence", Graphic3dSequenceOfHClipPlane, base,
    interfaces: [dyn Graphic3dClipPlaneSequenceSource]);
impl Graphic3dClipPlaneSequenceSource for RenderClipDerivedSequence {
    fn clip_sequence_base(&self) -> &Graphic3dSequenceOfHClipPlane {
        &self.base
    }
}
/// Submit real subclass storage and mutate through its captured native field; `configure(scene,mode)`.
pub fn configure(scene: &ViewerScene, mode: &str) {
    let plane = derived_render_clip_plane();
    let sequence = derived_render_clip_sequence();
    let source = sequence.observe();
    source.append_reference(&plane);
    scene.view.set_clip_planes(&sequence);
    check_render_clip_slot(scene, sequence.object_address());
    if mode == "clip-derived-alias-clear" {
        scene.render_view().try_redraw().unwrap();
        scene.view.clip_planes_ref().observe().clear();
    }
}

fn check_render_clip_slot(scene: &ViewerScene, expected: *const ()) {
    // Public virtual getter must identify the same actual field consumed by direct native rendering.
    assert_eq!(scene.view.clip_planes_ref().object_address(), expected);
}

fn derived_render_clip_plane() -> Handle<dyn Graphic3dClipPlaneSource> {
    Handle::new(RenderClipDerivedPlane {
        base: Graphic3dClipPlane::from_equation(Vec4::new(1., 0., 0., 0.)).unwrap(),
    })
    .into_cast()
}
fn derived_render_clip_sequence() -> Handle<dyn Graphic3dClipPlaneSequenceSource> {
    Handle::new(RenderClipDerivedSequence {
        base: Graphic3dSequenceOfHClipPlane::default(),
    })
    .into_cast()
}
