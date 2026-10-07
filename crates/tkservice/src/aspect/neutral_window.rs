//! `Aspect_NeutralWindow`: a window defined by its native handles and geometry only,
//! for an application that creates and manages the real window itself.
use super::{AspectDrawable, AspectFbConfig, AspectTypeOfResize, AspectWindow, AspectWindowBase};
use rustycad_tkernel::standard::{SharedState, StandardTransient};

/// The protected fields of `Aspect_NeutralWindow`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct NeutralFields {
    handle: AspectDrawable,
    parent_handle: AspectDrawable,
    fb_config: AspectFbConfig,
    pos_x: i32,
    pos_y: i32,
    width: i32,
    height: i32,
    is_mapped: bool,
}

/// A window whose handles, position and size the application sets.
///
/// Example: `let window = AspectNeutralWindow::default(); window.set_size(640, 480);`.
pub struct AspectNeutralWindow {
    transient: StandardTransient,
    base: AspectWindowBase,
    fields: SharedState<NeutralFields>,
}
impl Default for AspectNeutralWindow {
    /// No handles, empty geometry, mapped, as `Aspect_NeutralWindow()`.
    fn default() -> Self {
        let fields = NeutralFields {
            is_mapped: true,
            ..Default::default()
        };
        Self {
            transient: StandardTransient::default(),
            base: AspectWindowBase::default(),
            fields: SharedState::new(fields),
        }
    }
}
rustycad_tkernel::impl_transient!(AspectNeutralWindow, "Aspect_NeutralWindow", dyn AspectWindow, transient,
    interfaces: [dyn AspectWindow]);

impl AspectNeutralWindow {
    /// Sets the window handle only (no parent, no FB config); returns true when it changed.
    ///
    /// Example: `window.set_native_handle(0x2a)`.
    pub fn set_native_handle(&self, window: AspectDrawable) -> bool {
        self.set_native_handles(window, 0, 0)
    }
    /// Sets the window, parent and FB-config handles; returns true when any changed.
    ///
    /// Example: `window.set_native_handles(0x2a, 0, 0)`.
    pub fn set_native_handles(
        &self,
        window: AspectDrawable,
        parent: AspectDrawable,
        fb_config: AspectFbConfig,
    ) -> bool {
        let mut f = self.fields.write();
        let changed = (f.handle, f.parent_handle, f.fb_config) != (window, parent, fb_config);
        (f.handle, f.parent_handle, f.fb_config) = (window, parent, fb_config);
        changed
    }
    /// `SetPosition(x1, y1)`: moves the window; returns true when it changed.
    ///
    /// Example: `window.set_position(10, 20)`.
    pub fn set_position(&self, x1: i32, y1: i32) -> bool {
        let mut f = self.fields.write();
        let changed = (f.pos_x, f.pos_y) != (x1, y1);
        (f.pos_x, f.pos_y) = (x1, y1);
        changed
    }
    /// `SetPosition(x1, y1, x2, y2)`: moves and resizes; returns true when it changed.
    ///
    /// Example: `window.set_position_and_size(0, 0, 640, 480)`.
    pub fn set_position_and_size(&self, x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
        let (width, height) = (x2.wrapping_sub(x1), y2.wrapping_sub(y1));
        let mut f = self.fields.write();
        let changed = (f.pos_x, f.pos_y, f.width, f.height) != (x1, y1, width, height);
        (f.pos_x, f.pos_y, f.width, f.height) = (x1, y1, width, height);
        changed
    }
    /// Resizes the window; returns true when it changed; `window.set_size(640, 480)`.
    pub fn set_size(&self, width: i32, height: i32) -> bool {
        let mut f = self.fields.write();
        let changed = (f.width, f.height) != (width, height);
        (f.width, f.height) = (width, height);
        changed
    }
}

impl AspectWindow for AspectNeutralWindow {
    fn window_base(&self) -> &AspectWindowBase {
        &self.base
    }
    fn is_mapped(&self) -> bool {
        self.fields.read().is_mapped
    }
    fn map(&self) {
        self.fields.write().is_mapped = true;
    }
    fn unmap(&self) {
        self.fields.write().is_mapped = false;
    }
    fn do_resize(&self) -> AspectTypeOfResize {
        AspectTypeOfResize::Unknown
    }
    fn do_mapping(&self) -> bool {
        true
    }
    /// Width over height, 1.0 for an empty side.
    fn ratio(&self) -> f64 {
        let f = self.fields.read();
        if f.width != 0 && f.height != 0 {
            return f64::from(f.width) / f64::from(f.height);
        }
        1.0
    }
    fn position(&self) -> (i32, i32, i32, i32) {
        let f = self.fields.read();
        (
            f.pos_x,
            f.pos_y,
            f.pos_x.wrapping_add(f.width),
            f.pos_y.wrapping_add(f.height),
        )
    }
    fn size(&self) -> (i32, i32) {
        let f = self.fields.read();
        (f.width, f.height)
    }
    fn native_handle(&self) -> AspectDrawable {
        self.fields.read().handle
    }
    fn native_parent_handle(&self) -> AspectDrawable {
        self.fields.read().parent_handle
    }
    fn native_fb_config(&self) -> AspectFbConfig {
        self.fields.read().fb_config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aspect::AspectWindowMembers;

    #[test]
    fn setters_report_changes_and_geometry_follows() {
        let window = AspectNeutralWindow::default();
        assert!(window.is_mapped());
        assert!(window.set_position_and_size(10, 20, 650, 500));
        assert!(!window.set_position_and_size(10, 20, 650, 500));
        assert_eq!(window.position(), (10, 20, 650, 500));
        assert_eq!(window.ratio(), 640.0 / 480.0);
        assert!(window.set_size(0, 480));
        assert_eq!(window.ratio(), 1.0);
        assert!(window.set_position(1, 2) && !window.set_position(1, 2));
        assert_eq!(
            window.top_left(),
            rustycad_tkernel::ncollection::Vec2::new(1, 2)
        );
        assert!(window.set_native_handle(42) && !window.set_native_handle(42));
        assert_eq!(
            (window.native_handle(), window.native_parent_handle()),
            (42, 0)
        );
        window.unmap();
        assert!(!window.is_mapped());
        assert_eq!(window.do_resize(), AspectTypeOfResize::Unknown);
    }
}
