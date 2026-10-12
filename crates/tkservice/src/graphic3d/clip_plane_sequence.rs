//! `Graphic3d_SequenceOfHClipPlane`: the clipping planes of an object or a view, each plane
//! at most once, with the flag that overrides the global planes.
use super::Graphic3dClipPlane;
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient};

/// Genuine `Graphic3d_SequenceOfHClipPlane::base_type`; `let parent=Graphic3dClipSequenceParent::default()`.
pub type Graphic3dClipSequenceParent = StandardTransient;

/// A sequence of distinct clip planes; `Graphic3dSequenceOfHClipPlane::default().append(plane)`.
#[derive(Default)]
pub struct Graphic3dSequenceOfHClipPlane {
    transient: StandardTransient,
    /// Actual protected node storage; `planes.clip_plane_items.read().length()`.
    /// Its distinct payload handle fields retain native ownership independently of cursor leases.
    pub clip_plane_items: super::clip_sequence_storage::ClipPlaneNativeItems,
    /// Actual protected override flag; `*planes.clip_override_global.write() = true`.
    pub clip_override_global: SharedState<bool>,
}
rustycad_tkernel::impl_transient!(
    Graphic3dSequenceOfHClipPlane,
    "Graphic3d_SequenceOfHClipPlane",
    StandardTransient,
    transient,
    interfaces: [dyn super::Graphic3dClipPlaneSequenceSource]
);

impl Graphic3dSequenceOfHClipPlane {
    /// The planes in order as explicitly owning convenience snapshots; `planes.items()`.
    pub fn items(&self) -> Vec<Handle<Graphic3dClipPlane>> {
        self.clip_plane_items
            .read()
            .iter()
            .map(|field| {
                use rustycad_tkernel::standard::HandleReference;
                field.observe().observe_as::<Graphic3dClipPlane>().clone()
            })
            .collect()
    }
    /// Whether these planes replace the global ones; `planes.to_override_global()`.
    pub fn to_override_global(&self) -> bool {
        // The flag belongs to the real embedded collection base, including native subclasses.
        *self.clip_override_global.read()
    }
    /// Sets the override flag; `planes.set_override_global(true)`.
    pub fn set_override_global(&self, to_override: bool) {
        // Callback edits and protected-field edits share exactly one retained flag.
        *self.clip_override_global.write() = to_override;
    }
    /// Whether the sequence is empty; `planes.is_empty()`.
    pub fn is_empty(&self) -> bool {
        // Native metadata remains observable throughout controlled clear callbacks.
        self.clip_plane_items.read().is_empty()
    }
    /// The number of planes; `planes.size()`.
    pub fn size(&self) -> i32 {
        // Native Clear preserves this old length until the final allocator callback returns.
        self.clip_plane_items.read().length()
    }
}
