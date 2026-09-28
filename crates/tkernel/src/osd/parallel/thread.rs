// Created on: 2006-04-12
// Created by: Andrey BETENEV
// Copyright (c) 2006-2014 OPEN CASCADE SAS
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

//! `OSD_Thread`: a function run on its own thread, with the pthread handle semantics of
//! the native class: copies share one thread, and detaching any copy (including by
//! destroying it) makes the thread unjoinable for all of them.
use super::event::ManualResetEvent;
use std::{
    sync::{Arc, Mutex, MutexGuard},
    thread::JoinHandle,
};

pub(super) mod native;

/// `OSD_ThreadFunction` (`void* (*)(void*)`): the opaque pointer-sized argument and
/// result travel as `usize`; `fn double(data: usize) -> usize { data * 2 }`.
pub type ThreadFunction = fn(usize) -> usize;

/// One started thread, shared by the `OsdThread` copies that refer to it.
struct RunningThread {
    handle: Mutex<Option<JoinHandle<usize>>>,
    finished: ManualResetEvent,
}
impl RunningThread {
    fn handle(&self) -> MutexGuard<'_, Option<JoinHandle<usize>>> {
        self.handle
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}

/// Signals the end of the thread function even when it panics.
struct FinishSignal(Arc<RunningThread>);
impl Drop for FinishSignal {
    fn drop(&mut self) {
        self.0.finished.set();
    }
}

/// `OSD_Thread`; `let mut thread = OsdThread::with_function(double); thread.run(21, 0);`.
pub struct OsdThread {
    function: Option<ThreadFunction>,
    thread: Option<Arc<RunningThread>>,
    thread_id: usize,
    priority: i32,
}

impl Default for OsdThread {
    fn default() -> Self {
        Self::new()
    }
}

impl OsdThread {
    /// A thread without function; `OsdThread::new().run(0, 0) == false`.
    pub const fn new() -> Self {
        Self {
            function: None,
            thread: None,
            thread_id: 0,
            priority: 0,
        }
    }
    /// A thread ready to run `function`; `OsdThread::with_function(double)`.
    pub const fn with_function(function: ThreadFunction) -> Self {
        Self {
            function: Some(function),
            thread: None,
            thread_id: 0,
            priority: 0,
        }
    }
    /// `Assign`: copies the function and priority, detaches this thread, then shares the
    /// other's thread and identifier; `thread.assign(&other)`.
    pub fn assign(&mut self, other: &Self) {
        self.function = other.function;
        self.priority = other.priority;
        self.detach();
        self.thread = other.thread.clone();
        self.thread_id = other.thread_id;
    }
    /// Stores the priority; applied with `SetThreadPriority` on Windows only (the pthread
    /// build keeps the value without effect); `thread.set_priority(1)`.
    pub fn set_priority(&mut self, priority: i32) {
        self.priority = priority;
        if let Some(thread) = &self.thread
            && let Some(handle) = thread.handle().as_ref()
        {
            native::apply_priority(handle, priority);
        }
    }
    /// Detaches the current thread, then stores the function; `thread.set_function(double)`.
    pub fn set_function(&mut self, function: ThreadFunction) {
        self.detach();
        self.function = Some(function);
    }
    /// `Run`: starts `function(data)` on a new thread after detaching the previous one;
    /// false without function or when the system refuses the thread; `thread.run(21, 0)`.
    ///
    /// `stack_size` is honoured on Windows only when positive, as natively (the pthread
    /// build ignores it).
    pub fn run(&mut self, data: usize, stack_size: i32) -> bool {
        let Some(function) = self.function else {
            return false;
        };
        self.detach();
        let running: Arc<RunningThread> = Arc::new(RunningThread {
            handle: Mutex::new(None),
            finished: ManualResetEvent::default(),
        });
        let signal: FinishSignal = FinishSignal(Arc::clone(&running));
        let spawned = thread_builder(stack_size).spawn(move || {
            let _signal: FinishSignal = signal;
            function(data)
        });
        let Ok(handle) = spawned else {
            return false;
        };
        self.thread_id = native::thread_id(&handle);
        native::apply_priority(&handle, self.priority);
        *running.handle() = Some(handle);
        self.thread = Some(running);
        true
    }
    /// `Detach`: lets the thread run on unjoined, for every copy; `thread.detach()`.
    pub fn detach(&mut self) {
        if let Some(thread) = self.thread.take() {
            // Dropping the JoinHandle is pthread_detach.
            drop(thread.handle().take());
        }
        self.thread_id = 0;
    }
    /// `Wait()`: joins the thread; `thread.wait()`.
    pub fn wait(&mut self) -> bool {
        self.wait_result().is_some()
    }
    /// `Wait(void*&)`: joins the thread and returns its result; `None` (native false with
    /// a null result) when nothing runs, when a copy already joined or detached it, or when
    /// the thread function panicked (natively a throwing thread terminates the process).
    pub fn wait_result(&mut self) -> Option<usize> {
        let thread: Arc<RunningThread> = self.thread.clone()?;
        self.join(&thread)
    }
    /// `Wait(int, void*&)`: joins the thread if it ends within `milliseconds` (negative
    /// waits count as 0); `thread.wait_for(100)`.
    pub fn wait_for(&mut self, milliseconds: i32) -> Option<usize> {
        let thread: Arc<RunningThread> = self.thread.clone()?;
        if !thread.finished.wait_for(milliseconds) {
            return None;
        }
        self.join(&thread)
    }
    fn join(&mut self, thread: &RunningThread) -> Option<usize> {
        // A failed join leaves the handle set, as natively.
        let handle: JoinHandle<usize> = thread.handle().take()?;
        let result: usize = handle.join().ok()?;
        self.thread = None;
        self.thread_id = 0;
        Some(result)
    }
    /// The identifier recorded by `run`, 0 when nothing runs; `thread.id()`.
    pub const fn id(&self) -> usize {
        self.thread_id
    }
    /// `Current`: the identifier of the calling thread (`pthread_self`,
    /// `GetCurrentThreadId` on Windows); `OsdThread::current()`.
    pub fn current() -> usize {
        native::current_thread_id()
    }
}

#[cfg(windows)]
fn thread_builder(stack_size: i32) -> std::thread::Builder {
    let builder: std::thread::Builder = std::thread::Builder::new();
    match usize::try_from(stack_size) {
        Ok(size) if size > 0 => builder.stack_size(size),
        _ => builder,
    }
}

#[cfg(not(windows))]
fn thread_builder(_stack_size: i32) -> std::thread::Builder {
    std::thread::Builder::new()
}

impl Clone for OsdThread {
    /// The copy constructor: shares the thread (see [`OsdThread::assign`]).
    fn clone(&self) -> Self {
        let mut copy: Self = Self::new();
        copy.assign(self);
        copy
    }
    /// `operator=`: [`OsdThread::assign`].
    fn clone_from(&mut self, source: &Self) {
        self.assign(source);
    }
}

impl Drop for OsdThread {
    /// `~OSD_Thread`: detaches the thread, for every copy (native quirk).
    fn drop(&mut self) {
        self.detach();
    }
}
