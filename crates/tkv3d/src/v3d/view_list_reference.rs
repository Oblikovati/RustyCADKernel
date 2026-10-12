//! Borrow native viewer list nodes without copying their retained view handles.
use super::viewer::V3dViewerState;
use super::{V3dViewSource, V3dViewer};
use rustycad_tkernel::ncollection::{AllocatorList, ListCursor};
use rustycad_tkernel::standard::{Handle, HandleObserver, HandleReference};
type ViewChain = AllocatorList<Handle<dyn V3dViewSource>>;
type ViewChainSelector = fn(&V3dViewerState) -> &ViewChain;

/// Genuine const viewer-list borrow; `viewer.active_views_ref().cursor()`.
#[derive(Clone, Copy)]
pub struct V3dViewListRef<'a> {
    viewer: &'a V3dViewer,
    select: ViewChainSelector,
}
/// Detached native node iterator, with no extra owning view references; `cursor.value_ref().observe()`.
pub struct V3dViewCursor<'a> {
    chain: V3dViewListRef<'a>,
    position: ListCursor,
}
/// Borrow the actual constHandle field in a retained node; `cursor.value_ref().reference_address()`.
pub struct V3dViewNodeRef<'a> {
    chain: V3dViewListRef<'a>,
    position: ListCursor,
}
impl<'a> V3dViewListRef<'a> {
    /// Observe the current native extent without a target copy; `viewer.active_views_ref().len()`.
    pub fn len(&self) -> usize {
        let state = self.viewer.state.read();
        (self.select)(&state).len()
    }
    /// Observe the empty list; `viewer.active_views_ref().is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Identify the native field itself, independent of its nodes; `left.reference_address()==right.reference_address()`.
    pub fn reference_address(&self) -> *const () {
        let state = self.viewer.state.read();
        std::ptr::from_ref((self.select)(&state)).cast()
    }
    fn node_reference(&self, position: ListCursor) -> V3dViewNodeRef<'a> {
        V3dViewNodeRef {
            chain: *self,
            position,
        }
    }
    /// Begin at the current native first node; `let cursor=viewer.active_views_ref().cursor()`.
    pub fn cursor(&self) -> V3dViewCursor<'a> {
        let state = self.viewer.state.read();
        V3dViewCursor {
            chain: *self,
            position: (self.select)(&state).cursor(),
        }
    }
}
impl<'a> V3dViewCursor<'a> {
    /// Test the native current-node link; `while cursor.more() { cursor.next(); }`.
    pub fn more(&self) -> bool {
        self.position.more()
    }
    /// Read the actual positioned constHandle slot; `let alias=cursor.value_ref()`.
    pub fn value_ref(&self) -> V3dViewNodeRef<'a> {
        V3dViewNodeRef {
            chain: self.chain,
            position: self.position,
        }
    }
    /// Advance after callbacks so live append/remove operations remain observable; `cursor.next()`.
    pub fn next(&mut self) {
        let state = self.chain.viewer.state.read();
        self.position.next((self.chain.select)(&state));
    }
    /// Expose native node identity for retained-cursor checks; `cursor.node_address()`.
    pub fn node_address(&self) -> Option<usize> {
        self.position.current_node()
    }
}
impl HandleReference<dyn V3dViewSource> for V3dViewNodeRef<'_> {
    fn observe(&self) -> HandleObserver<dyn V3dViewSource> {
        let state = self.chain.viewer.state.read();
        let value = self.position.value((self.chain.select)(&state));
        value
            .unwrap_or_else(|| invalid_view_node(self.position))
            .observe()
    }
    fn reference_address(&self) -> *const () {
        let state = self.chain.viewer.state.read();
        let value = self.position.value((self.chain.select)(&state));
        std::ptr::from_ref(value.unwrap_or_else(|| invalid_view_node(self.position))).cast()
    }
}
fn invalid_view_node(position: ListCursor) -> ! {
    match rustycad_tkernel::standard::raise_native_failure::<std::convert::Infallible>(Err(
        rustycad_tkernel::standard::exceptions::program_error(&format!(
            "V3d view node {:?}: expected a live native list node",
            position.current_node()
        )),
    )) {}
}
impl V3dViewer {
    /// Borrow native ActiveViews storage; `let alias=viewer.active_views_ref()`.
    pub fn active_views_ref(&self) -> V3dViewListRef<'_> {
        V3dViewListRef {
            viewer: self,
            select: active_view_chain,
        }
    }
    /// Borrow the deprecated active cursor's actual handle node; `viewer.active_view_ref().observe()`.
    pub fn active_view_ref(&self) -> V3dViewNodeRef<'_> {
        self.active_views_ref()
            .node_reference(self.state.read().view_cursors[0])
    }
    /// Borrow the deprecated defined cursor's actual handle node; `viewer.defined_view_ref().observe()`.
    pub fn defined_view_ref(&self) -> V3dViewNodeRef<'_> {
        self.defined_views_ref()
            .node_reference(self.state.read().view_cursors[1])
    }
    /// Borrow native DefinedViews storage; `viewer.defined_views_ref().cursor()`.
    pub fn defined_views_ref(&self) -> V3dViewListRef<'_> {
        V3dViewListRef {
            viewer: self,
            select: defined_view_chain,
        }
    }
}
fn active_view_chain(fields: &V3dViewerState) -> &ViewChain {
    &fields.active_views
}
fn defined_view_chain(fields: &V3dViewerState) -> &ViewChain {
    &fields.defined_views
}
