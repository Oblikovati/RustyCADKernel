//! The per-thread stack of active handlers: the C++ `thread_local Top` pointer and the
//! `myPrevious` / `myCallbackPtr` / `myCaughtError` fields it links.
//!
//! A handler is a slot addressed by an id; the slot outlives unlinking and is removed
//! only when the handler value drops, as the C++ object outlives `Destroy()`.
use super::callback::CallbackSlot;
use crate::standard::Failure;
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

/// Identity of one handler in its thread's stack.
pub(super) type HandlerId = u64;
/// The C++ fields of one handler.
#[derive(Default)]
pub(super) struct HandlerFrame {
    pub(super) previous: Option<HandlerId>,
    /// Registered callbacks, most recently registered first (the C++ list head).
    pub(super) callbacks: Vec<Rc<CallbackSlot>>,
    pub(super) caught: Option<Failure>,
}
#[derive(Default)]
struct HandlerStack {
    top: Option<HandlerId>,
    frames: BTreeMap<HandlerId, HandlerFrame>,
    next_id: HandlerId,
}
thread_local! {
    static HANDLERS: RefCell<HandlerStack> = RefCell::default();
}
/// Links a new handler on top of the thread's stack, as the constructor.
pub(super) fn push_handler() -> HandlerId {
    HANDLERS.with_borrow_mut(|stack| {
        let id: HandlerId = stack.next_id;
        stack.next_id += 1;
        let previous: Option<HandlerId> = stack.top.replace(id);
        stack.frames.insert(
            id,
            HandlerFrame {
                previous,
                ..HandlerFrame::default()
            },
        );
        id
    })
}
/// The innermost active handler, as `FindHandler()`.
pub(super) fn top_handler() -> Option<HandlerId> {
    HANDLERS.with_borrow(|stack| stack.top)
}
/// Runs `edit` on a handler's fields; `None` once the handler has dropped.
pub(super) fn with_frame<R>(id: HandlerId, edit: impl FnOnce(&mut HandlerFrame) -> R) -> Option<R> {
    HANDLERS.with_borrow_mut(|stack| stack.frames.get_mut(&id).map(edit))
}
/// Forgets a dropped handler's fields.
pub(super) fn remove_frame(id: HandlerId) {
    HANDLERS.with_borrow_mut(|stack| stack.frames.remove(&id));
}
/// `Unlink()`: first unlinks every handler above this one (the C++ comment says this
/// "shouldn't happen in normal usage"), pops this one when it is on top, then detaches
/// and runs its callbacks, most recently registered first. A handler that is no longer
/// linked therefore unlinks the whole stack above it, as in C++.
pub(super) fn unlink(id: HandlerId) {
    while let Some(above) = top_handler().filter(|top| *top != id) {
        unlink(above);
    }
    let callbacks: Vec<Rc<CallbackSlot>> = HANDLERS.with_borrow_mut(|stack| {
        let previous: Option<HandlerId> = stack.frames.get(&id).and_then(|frame| frame.previous);
        if stack.top == Some(id) {
            stack.top = previous;
        }
        stack
            .frames
            .get_mut(&id)
            .map_or_else(Vec::new, detach_frame)
    });
    // No borrow is held here: a callback may register or unregister other callbacks.
    for callback in callbacks {
        callback.destroy_callback();
    }
}
/// Clears `myPrevious` and hands over the callback list, as the C++ loop does.
fn detach_frame(frame: &mut HandlerFrame) -> Vec<Rc<CallbackSlot>> {
    frame.previous = None;
    std::mem::take(&mut frame.callbacks)
}
