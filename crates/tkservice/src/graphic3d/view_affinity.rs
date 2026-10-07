//! `Graphic3d_ViewAffinity`: the per-view visibility mask of a presentation.
use crate::json_dump::{class_begin, field_unsigned};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{SharedState, StandardTransient};
use std::io;

/// A view visibility mask, visible in every view at first.
/// Example: `let affinity = Graphic3dViewAffinity::default(); affinity.set_view_visible(3, false);`.
pub struct Graphic3dViewAffinity {
    transient: StandardTransient,
    mask: SharedState<u32>,
}
rustycad_tkernel::impl_transient!(
    Graphic3dViewAffinity,
    "Graphic3d_ViewAffinity",
    StandardTransient,
    transient
);

impl Default for Graphic3dViewAffinity {
    fn default() -> Self {
        Self {
            transient: StandardTransient::default(),
            mask: SharedState::new(u32::MAX),
        }
    }
}

/// `1 << theViewId` as the x86 shift computes it: the count is taken modulo 32 (natively
/// undefined behaviour outside 0..31).
fn view_bit(view_id: i32) -> u32 {
    1u32.wrapping_shl(view_id as u32)
}

impl Graphic3dViewAffinity {
    /// Whether the view is visible; `affinity.is_visible(0)`.
    pub fn is_visible(&self, view_id: i32) -> bool {
        *self.mask.read() & view_bit(view_id) != 0
    }
    /// Shows or hides every view (`SetVisible(bool)`); `affinity.set_visible(false)`.
    pub fn set_visible(&self, is_visible: bool) {
        *self.mask.write() = if is_visible { u32::MAX } else { 0 };
    }
    /// Shows or hides one view (`SetVisible(int, bool)`); `affinity.set_view_visible(2, true)`.
    pub fn set_view_visible(&self, view_id: i32, is_visible: bool) {
        let mut mask = self.mask.write();
        *mask = if is_visible {
            *mask | view_bit(view_id)
        } else {
            *mask & !view_bit(view_id)
        };
    }
    /// `DumpJson`; `affinity.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, _depth: i32) -> io::Result<()> {
        class_begin(stream, "Graphic3d_ViewAffinity")?;
        field_unsigned(stream, "myMask", u64::from(*self.mask.read()))
    }
}
