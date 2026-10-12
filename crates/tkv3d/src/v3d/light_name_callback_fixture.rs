//! Named CPU backend reuses production view forwarding; only the shadow byte observer differs.
#![allow(unsafe_code)] // One anchored UnsafeCell buffer is edited/read serially by actual callbacks.
use super::{V3dView, V3dViewSource};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkservice::{aspect::*, graphic3d::*, image::ImagePixMap};
use std::{
    cell::UnsafeCell,
    ffi::c_char,
    sync::{Arc, Mutex},
};

pub(super) struct CallbackNameStorage {
    bytes: Mutex<UnsafeCell<[u8; 8]>>,
    pub rows: SharedState<Vec<String>>,
}
impl CallbackNameStorage {
    /// Anchor the foreign bytes through all callbacks; `CallbackNameStorage::new()`.
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            bytes: Mutex::new(UnsafeCell::new(*b"sun\0\0\0\0\0")),
            rows: SharedState::default(),
        })
    }
    /// Observe the same physical array without a lasting guard; `control.pointer()`.
    pub fn pointer(&self) -> *const c_char {
        let bytes = self.bytes.lock().unwrap();
        bytes.get().cast::<c_char>()
    }
    /// Update bytes between consumption reads; `control.overwrite(*b"moon\0xxx")`.
    pub fn overwrite(&self, replacement: [u8; 8]) {
        let bytes = self.bytes.lock().unwrap();
        // No test thread runs a read concurrently; UnsafeCell retains the raw buffer provenance.
        unsafe { bytes.get().write(replacement) };
    }
}
pub(super) struct ObservedRetainedView {
    base: Graphic3dRetainedView,
    pub control: Arc<CallbackNameStorage>,
}
impl std::ops::Deref for ObservedRetainedView {
    type Target = Graphic3dRetainedView;
    fn deref(&self) -> &Self::Target {
        // Forward against the actual embedded view fields, not a copied scene.
        &self.base
    }
}
impl ObservedRetainedView {
    fn shadow_map_dump(&self, _: &ImagePixMap, name: &AsciiString) -> bool {
        let same =
            i32::from(name.as_bytes_with_nul().as_ptr().cast::<c_char>() == self.control.pointer());
        self.control.overwrite(*b"star\0xxx");
        let mut row = format!("consumed {same} {}", name.len());
        for byte in name.as_bytes() {
            row.push_str(&format!(" {byte}"));
        }
        self.control.rows.write().push(row);
        false
    }
}
pub(super) struct CallbackObservedView {
    transient: StandardTransient,
    retained: ObservedRetainedView,
}
rustycad_tkernel::impl_transient!(CallbackObservedView,"Martensite_CallbackObservedView",StandardTransient,transient,
    interfaces:[dyn Graphic3dCView,dyn Graphic3dDataStructureManager]);
impl CallbackObservedView {
    /// Register a real retained backend with a byte observer; `CallbackObservedView::create(manager,control)`.
    pub fn create(
        manager: &Handle<Graphic3dStructureManager>,
        control: &Arc<CallbackNameStorage>,
    ) -> Handle<dyn Graphic3dCView> {
        let base = Graphic3dRetainedView::new(manager, &[]).unwrap();
        base.set_fbo(&Handle::new(StandardTransient::default()));
        let view = Handle::new(Self {
            transient: Default::default(),
            retained: ObservedRetainedView {
                base,
                control: control.clone(),
            },
        })
        .cast();
        graphic3d_cview_register(&view).unwrap();
        view
    }
}
pub(super) struct CallbackNameView {
    pub base: V3dView,
    pub control: Arc<CallbackNameStorage>,
}
rustycad_tkernel::impl_transient!(CallbackNameView,"Martensite_CallbackNameView",V3dView,base,
    interfaces:[dyn V3dViewSource]);
impl V3dViewSource for CallbackNameView {
    fn view_base(&self) -> &V3dView {
        // Callback dispatch must preserve the original view/implementation identities.
        &self.base
    }
    fn redraw(&self) {
        self.control.overwrite(*b"moon\0xxx");
        super::view_source_redraw::v3d_view_redraw(&self.base);
    }
}
impl Graphic3dCView for CallbackObservedView {
    rustycad_tkservice::martensite_retained_cview_methods!();
    fn cview_base(&self) -> &Graphic3dCViewBase {
        // The real base preserves original camera and layer state across the observer.
        self.retained.cview_base()
    }
    fn redraw(&self) {
        // Only the outer native Redraw callback changes the borrowed bytes.
        self.retained.redraw();
    }
    fn redraw_immediate(&self) {
        // Immediate callbacks keep the same retained backend behavior.
        self.retained.redraw_immediate();
    }
    fn resized(&self) {
        // No test-specific camera or viewport state is substituted.
        self.retained.resized();
    }
    fn set_window(
        &self,
        parent: &Handle<dyn Graphic3dCView>,
        window: &Handle<dyn AspectWindow>,
        context: AspectRenderingContext,
    ) {
        self.retained.set_window(parent, window, context);
    }
    fn buffer_dump(&self, image: &ImagePixMap, kind: &Graphic3dBufferType) -> bool {
        // Ordinary buffers must keep the retained backend result.
        self.retained.buffer_dump(image, kind)
    }
    fn fbo_create(&self, width: i32, height: i32) -> Handle<StandardTransient> {
        // Allocation remains the named backend responsibility.
        self.retained.fbo_create(width, height)
    }
    fn fbo_release(&self, fbo: &mut Handle<StandardTransient>) {
        // Release the original caller slot instead of an owning snapshot.
        self.retained.fbo_release(fbo);
    }
    fn fbo_get_dimensions(&self, _: &Handle<StandardTransient>) -> [i32; 4] {
        // Both native/Rust named framebuffers have this fixed validated extent.
        [8, 8, 128, 128]
    }
    fn fbo_change_viewport(&self, fbo: &Handle<StandardTransient>, width: i32, height: i32) {
        // Viewport updates target the actual retained framebuffer.
        self.retained.fbo_change_viewport(fbo, width, height);
    }
    fn diagnostic_information(
        &self,
        dict: &mut Graphic3dFrameStatsDict,
        flags: Graphic3dDiagnosticInfo,
    ) {
        // Diagnostics preserve the same real base state.
        self.retained.diagnostic_information(dict, flags);
    }
}
