//! Real embedded view exercises native virtual transport on the renderer, without a retained core view.
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::graphic3d::Graphic3dCViewMembers as _;
use rustycad_tkv3d::v3d::{V3dTypeOfView, V3dView, V3dViewSource, V3dViewer};
use std::sync::atomic::{AtomicUsize, Ordering};

struct DerivedRenderSceneView {
    base: V3dView,
    redrawn: AtomicUsize,
}
rustycad_tkernel::impl_transient!(DerivedRenderSceneView,"Martensite_DerivedRenderSceneView",V3dView,base,
    interfaces:[dyn V3dViewSource]);
impl V3dViewSource for DerivedRenderSceneView {
    fn view_base(&self) -> &V3dView {
        &self.base
    }
    fn redraw_immediate(&self) {
        self.redrawn.fetch_add(1, Ordering::Relaxed);
        assert!(!self.base.camera_ref().is_null());
        rustycad_tkv3d::v3d::v3d_view_redraw_immediate(&self.base);
    }
    fn redraw(&self) {
        self.redrawn.fetch_add(1, Ordering::Relaxed);
        // Both calls forward through the actual driver view's virtual DSM Camera getter.
        assert!(!self.base.camera_ref().is_null());
        rustycad_tkv3d::v3d::v3d_view_redraw(&self.base);
    }
}

/// Bind one actual embedded view to the viewer; `create_embedded_render_view(&viewer)?`.
pub(super) fn create_embedded_render_view(
    viewer: &Handle<V3dViewer>,
) -> Result<Handle<dyn V3dViewSource>, Failure> {
    let derived = Handle::new(DerivedRenderSceneView {
        base: V3dView::new_embedded(viewer, V3dTypeOfView::Orthographic)?,
        redrawn: AtomicUsize::new(0),
    });
    let source: Handle<dyn V3dViewSource> = derived.cast();
    V3dView::bind_source(&source)?;
    Ok(source)
}

/// Exercise both virtual callbacks on a mapped renderer scene; `prove_render_source_dispatch(&scene.view)`.
pub(super) fn prove_render_source_dispatch(view: &Handle<dyn V3dViewSource>) {
    view.redraw();
    view.redraw_immediate();
    let derived: Handle<DerivedRenderSceneView> = view.cast();
    assert!(derived.redrawn.load(Ordering::Relaxed) >= 2);
    assert!(view.view_ref().observe().is_active());
}
