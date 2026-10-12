//! Borrow actual native handle-list nodes; internal inspection releases locks before external callbacks.
use rustycad_tkernel::ncollection::{AllocatorList, ListCursor};
use rustycad_tkernel::standard::{Handle, HandleObserver, HandleReference, RuntimeTyped};

pub(crate) trait HandleChainStorage<T: RuntimeTyped + ?Sized> {
    fn inspect_chain(&self, key: usize, inspect: &mut dyn FnMut(&AllocatorList<Handle<T>>));
}
/// Genuine native list-field borrow; `viewer.active_lights_ref().cursor()`.
pub struct V3dHandleChainRef<'a, T: RuntimeTyped + ?Sized> {
    owner: &'a dyn HandleChainStorage<T>,
    key: usize,
}
impl<T: RuntimeTyped + ?Sized> Copy for V3dHandleChainRef<'_, T> {}
impl<T: RuntimeTyped + ?Sized> Clone for V3dHandleChainRef<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}
/// Native detached cursor with no owning handle snapshots; `cursor.value_ref().observe()`.
pub struct V3dHandleChainCursor<'a, T: RuntimeTyped + ?Sized> {
    chain: V3dHandleChainRef<'a, T>,
    position: ListCursor,
}
/// Actual constHandle node alias; `alias.reference_address()`.
pub struct V3dHandleChainNodeRef<'a, T: RuntimeTyped + ?Sized> {
    chain: V3dHandleChainRef<'a, T>,
    position: ListCursor,
}
impl<'a, T: RuntimeTyped + ?Sized> V3dHandleChainRef<'a, T> {
    pub(crate) fn new(owner: &'a dyn HandleChainStorage<T>, key: usize) -> Self {
        Self { owner, key }
    }
    fn reborrow(&self) -> Self {
        Self {
            owner: self.owner,
            key: self.key,
        }
    }
    fn inspect<R>(&self, inspect: impl FnOnce(&AllocatorList<Handle<T>>) -> R) -> R {
        let mut inspect = Some(inspect);
        let mut result = None;
        self.owner.inspect_chain(self.key, &mut |chain| {
            result = Some(inspect
                .take()
                .expect("list inspection count=2; expected exactly one")(
                chain
            ));
        });
        result.expect("list inspection count=0; expected exactly one")
    }
    /// Read the actual current extent; `viewer.active_lights_ref().len()`.
    pub fn len(&self) -> usize {
        self.inspect(AllocatorList::len)
    }
    /// Read empty state without ownership; `viewer.active_lights_ref().is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Identify actual native field storage; `a.reference_address()==b.reference_address()`.
    pub fn reference_address(&self) -> *const () {
        self.inspect(|chain| std::ptr::from_ref(chain).cast())
    }
    pub(crate) fn node_reference(&self, position: ListCursor) -> V3dHandleChainNodeRef<'a, T> {
        V3dHandleChainNodeRef {
            chain: self.reborrow(),
            position,
        }
    }
    /// Begin with the current first native node; `let cursor=viewer.active_lights_ref().cursor()`.
    pub fn cursor(&self) -> V3dHandleChainCursor<'a, T> {
        V3dHandleChainCursor {
            chain: self.reborrow(),
            position: self.inspect(AllocatorList::cursor),
        }
    }
}
impl<'a, T: RuntimeTyped + ?Sized> V3dHandleChainCursor<'a, T> {
    /// Test native node identity; `while cursor.more() { cursor.next(); }`.
    pub fn more(&self) -> bool {
        self.position.more()
    }
    /// Borrow the actual current handle slot; `cursor.value_ref().observe()`.
    pub fn value_ref(&self) -> V3dHandleChainNodeRef<'a, T> {
        self.chain.node_reference(self.position)
    }
    /// Advance only after callbacks so edits are visible; `cursor.next()`.
    pub fn next(&mut self) {
        self.chain.inspect(|chain| self.position.next(chain));
    }
    /// Observe stable native node identity; `cursor.node_address()`.
    pub fn node_address(&self) -> Option<usize> {
        self.position.current_node()
    }
}
impl<T: RuntimeTyped + ?Sized> V3dHandleChainNodeRef<'_, T> {
    pub(crate) fn with_handle<R>(&self, inspect: impl FnOnce(&Handle<T>) -> R) -> R {
        self.chain.inspect(|chain| {
            inspect(
                self.position
                    .value(chain)
                    .unwrap_or_else(|| invalid_handle_node(self.position)),
            )
        })
    }
}
impl<T: RuntimeTyped + ?Sized> HandleReference<T> for V3dHandleChainNodeRef<'_, T> {
    fn observe(&self) -> HandleObserver<T> {
        self.chain.inspect(|chain| {
            self.position
                .value(chain)
                .unwrap_or_else(|| invalid_handle_node(self.position))
                .observe()
        })
    }
    fn reference_address(&self) -> *const () {
        self.chain.inspect(|chain| {
            std::ptr::from_ref(
                self.position
                    .value(chain)
                    .unwrap_or_else(|| invalid_handle_node(self.position)),
            )
            .cast()
        })
    }
}
fn invalid_handle_node(position: ListCursor) -> ! {
    match rustycad_tkernel::standard::raise_native_failure::<std::convert::Infallible>(Err(
        rustycad_tkernel::standard::exceptions::program_error(&format!(
            "V3d handle node {:?}: expected a live native list node",
            position.current_node()
        )),
    )) {}
}
