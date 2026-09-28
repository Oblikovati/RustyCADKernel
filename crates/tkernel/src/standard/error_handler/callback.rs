// Created on: 1992-09-28
// Created by: Ramin BARRETO
// Copyright (c) 1992-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! `Standard_ErrorHandler::Callback`, as built with `OCC_CONVERT_SIGNALS` (the Linux build).
use super::handler_stack::{self, HandlerId};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

/// The shared state of one callback: its `myHandler` link and its `DestroyCallback` action.
pub(super) struct CallbackSlot {
    handler: Cell<Option<HandlerId>>,
    action: RefCell<Box<dyn FnMut()>>,
}
impl CallbackSlot {
    /// Runs the action, as the handler's `Unlink` calls `DestroyCallback()`.
    pub(super) fn destroy_callback(&self) {
        (self.action.borrow_mut())();
    }
}
/// An object that the innermost error handler notifies when it is destroyed while the
/// object is still registered; the C++ subclass's `DestroyCallback()` override is the
/// closure given to [`ErrorHandlerCallback::new`]. Callbacks belong to one thread.
/// `let callback = ErrorHandlerCallback::new(|| release()); callback.register_callback();`.
pub struct ErrorHandlerCallback {
    slot: Rc<CallbackSlot>,
}
impl ErrorHandlerCallback {
    /// An unregistered callback that runs `destroy` from `DestroyCallback()`;
    /// `ErrorHandlerCallback::new(|| println!("released"))`.
    pub fn new(destroy: impl FnMut() + 'static) -> Self {
        Self {
            slot: Rc::new(CallbackSlot {
                handler: Cell::new(None),
                action: RefCell::new(Box::new(destroy)),
            }),
        }
    }
    /// `RegisterCallback()`: links this callback first in the innermost handler's list.
    /// It does nothing when already linked, including (as in C++) when the handler it
    /// was linked to has since been destroyed without unregistering it, and nothing when
    /// no handler is active; `callback.register_callback()`.
    pub fn register_callback(&self) {
        if self.slot.handler.get().is_some() {
            return;
        }
        let Some(top) = handler_stack::top_handler() else {
            return;
        };
        self.slot.handler.set(Some(top));
        let slot: Rc<CallbackSlot> = self.slot.clone();
        handler_stack::with_frame(top, |frame| frame.callbacks.insert(0, slot));
    }
    /// `UnregisterCallback()`: removes this callback from its handler's list, if it is
    /// still there, and forgets the handler; `callback.unregister_callback()`.
    pub fn unregister_callback(&self) {
        let Some(handler) = self.slot.handler.take() else {
            return;
        };
        handler_stack::with_frame(handler, |frame| {
            frame
                .callbacks
                .retain(|linked| !Rc::ptr_eq(linked, &self.slot))
        });
    }
    /// True while linked to a handler (the C++ `myHandler != nullptr`); `callback.is_registered()`.
    pub fn is_registered(&self) -> bool {
        self.slot.handler.get().is_some()
    }
    /// `DestroyCallback()`, called directly; `callback.destroy_callback()`.
    pub fn destroy_callback(&self) {
        self.slot.destroy_callback();
    }
}
/// `~Callback()` unregisters the object.
impl Drop for ErrorHandlerCallback {
    fn drop(&mut self) {
        self.unregister_callback();
    }
}
