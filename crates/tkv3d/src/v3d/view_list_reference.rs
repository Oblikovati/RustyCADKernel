//! Actual viewer view-list fields use the shared native node borrowing mechanism.
use super::handle_chain_reference::{
    HandleChainStorage, V3dHandleChainCursor, V3dHandleChainNodeRef, V3dHandleChainRef,
};
use super::{V3dViewSource, V3dViewer};
use rustycad_tkernel::{ncollection::AllocatorList, standard::Handle};
/// Borrow native ActiveViews or DefinedViews; `viewer.active_views_ref().cursor()`.
pub type V3dViewListRef<'a> = V3dHandleChainRef<'a, dyn V3dViewSource>;
/// Detached native view cursor; `viewer.active_view_iterator()`.
pub type V3dViewCursor<'a> = V3dHandleChainCursor<'a, dyn V3dViewSource>;
/// Native constHandle view node; `viewer.active_view_ref().observe()`.
pub type V3dViewNodeRef<'a> = V3dHandleChainNodeRef<'a, dyn V3dViewSource>;
impl HandleChainStorage<dyn V3dViewSource> for V3dViewer {
    fn inspect_chain(
        &self,
        key: usize,
        inspect: &mut dyn FnMut(&AllocatorList<Handle<dyn V3dViewSource>>),
    ) {
        let fields = self.state.read();
        if key == 0 {
            inspect(&fields.active_views);
            return;
        }
        inspect(&fields.defined_views);
    }
}
impl V3dViewer {
    /// Borrow native ActiveViews storage; `let alias=viewer.active_views_ref()`.
    pub fn active_views_ref(&self) -> V3dViewListRef<'_> {
        V3dViewListRef::new(self, 0)
    }
    /// Borrow deprecated active cursor's handle node; `viewer.active_view_ref().observe()`.
    pub fn active_view_ref(&self) -> V3dViewNodeRef<'_> {
        self.active_views_ref()
            .node_reference(self.state.read().view_cursors[0])
    }
    /// Borrow deprecated defined cursor's handle node; `viewer.defined_view_ref().observe()`.
    pub fn defined_view_ref(&self) -> V3dViewNodeRef<'_> {
        self.defined_views_ref()
            .node_reference(self.state.read().view_cursors[1])
    }
    /// Borrow native DefinedViews storage; `viewer.defined_views_ref().cursor()`.
    pub fn defined_views_ref(&self) -> V3dViewListRef<'_> {
        V3dViewListRef::new(self, 1)
    }
}
