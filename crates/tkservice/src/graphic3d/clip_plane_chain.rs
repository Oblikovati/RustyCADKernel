//! The chain members of `Graphic3d_ClipPlane`: the forward links, the weak back links and
//! the cached forward length.
use super::Graphic3dClipPlane;
use rustycad_tkernel::standard::exceptions::program_error;
use rustycad_tkernel::standard::{Failure, Handle};

impl Graphic3dClipPlane {
    /// Whether a next plane follows; `plane.is_chain()`.
    pub fn is_chain(&self) -> bool {
        !self.state.read().next.is_null()
    }
    /// The previous plane of the chain, null for a head or a destroyed predecessor.
    /// Example: `plane.chain_previous_plane().is_null()`.
    pub fn chain_previous_plane(&self) -> Handle<Self> {
        self.state.read().prev.upgrade()
    }
    /// The next plane of the chain; `plane.chain_next_plane()`.
    pub fn chain_next_plane(&self) -> Handle<Self> {
        self.state.read().next.clone()
    }
    /// The number of planes from this one to the chain end (at least 1).
    /// Example: `plane.nb_chain_next_planes()`.
    pub fn nb_chain_next_planes(&self) -> i32 {
        self.state.read().chain_len_fwd
    }
    /// `SetChainNextPlane`: links `next` after `this` (null cuts the chain), detaching the
    /// old next plane and updating the lengths of `this` and its predecessors. A link that
    /// would close a loop (natively an endless recursion) raises `Standard_ProgramError`
    /// before anything changes. Example: `Graphic3dClipPlane::set_chain_next_plane(&head, tail)?`.
    pub fn set_chain_next_plane(this: &Handle<Self>, next: Handle<Self>) -> Result<(), Failure> {
        Self::refuse_loop(this, &next)?;
        let old_next = {
            let mut state = this.state.write();
            state.equation_mod = state.equation_mod.wrapping_add(1);
            std::mem::replace(&mut state.next, next.clone())
        };
        if let Some(old) = old_next.get() {
            old.state.write().prev = Default::default();
        }
        if let Some(plane) = next.get() {
            plane.state.write().prev = this.downgrade();
        }
        Self::update_chain_len(this.clone());
        Ok(())
    }
    /// Refuses a `next` whose forward chain reaches `this`.
    fn refuse_loop(this: &Handle<Self>, next: &Handle<Self>) -> Result<(), Failure> {
        let mut cursor = next.clone();
        while let Some(plane) = cursor.get() {
            if cursor == *this {
                return Err(program_error(&format!(
                    "Graphic3d_ClipPlane::SetChainNextPlane() - plane {} would close a loop; \
                     expected a next chain that does not contain it",
                    String::from_utf8_lossy(plane.get_id().as_bytes())
                )));
            }
            let following = plane.chain_next_plane();
            cursor = following;
        }
        Ok(())
    }
    /// `updateChainLen`: recomputes the forward length of `plane` and of its predecessors.
    fn update_chain_len(start: Handle<Self>) {
        let mut cursor = start;
        while let Some(plane) = cursor.get() {
            let next = plane.chain_next_plane();
            let length = next.get().map_or(1, |n| n.nb_chain_next_planes() + 1);
            let previous = {
                let mut state = plane.state.write();
                state.chain_len_fwd = length;
                state.prev.upgrade()
            };
            cursor = previous;
        }
    }
}
