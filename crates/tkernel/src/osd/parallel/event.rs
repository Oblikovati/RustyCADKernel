//! The pool uses the shared `Standard_Condition` manual-reset event port:
//! `set` wakes every waiter and stays set until `reset`.
//! A panicking job never holds its waiter lock, so a poisoned flag remains consistent.
pub(super) use crate::standard::StandardCondition as ManualResetEvent;
