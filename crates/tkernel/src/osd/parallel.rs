// Created by: Kirill Gavrilov
// Copyright (c) 2017-2019 OPEN CASCADE SAS
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

//! Threads, the OCCT thread pool and parallel loops: `OSD_Thread`, `OSD_ThreadPool`,
//! `OSD_ThreadPool::Launcher` and `OSD_Parallel` (the non-TBB build of this profile).
//!
//! The pool is a port of the native one on `std::thread`, so thread indices, the locking
//! of idle workers and the "busy pool runs in the caller" fallback match OCCT exactly.
mod event;
mod job;
mod launcher;
mod loops;
mod thread;
mod thread_pool;
mod worker;

pub use launcher::Launcher;
pub use loops::Parallel;
pub use thread::{OsdThread, ThreadFunction};
pub use thread_pool::ThreadPool;
