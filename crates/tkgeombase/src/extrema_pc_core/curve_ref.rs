// Copyright (c) 2025 OPEN CASCADE SAS
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

//! The adaptor held by the offset and general-curve evaluators.
//!
//! Natively they keep a `const Adaptor_Curve*` to an adaptor that must outlive them;
//! `ExtremaPC(2d)_Curve` points it at an adaptor it owns (`myAdaptorOwned`) when built
//! from a Geom curve. The Rust evaluators hold either the borrowed adaptor or a handle
//! sharing the owned one, so no evaluator refers into its own owner.
use super::space::PcSpace;
use rustycad_tkernel::standard::Handle;

/// A borrowed or shared curve adaptor; `PcCurveRef::Borrowed(&adaptor)`.
pub enum PcCurveRef<'a, S: PcSpace> {
    /// An adaptor owned by the caller, which must outlive the evaluator.
    Borrowed(&'a S::Curve),
    /// An adaptor shared through a handle (the dispatcher's owned adaptor).
    Shared(Handle<S::Curve>),
}

impl<S: PcSpace> PcCurveRef<'_, S> {
    /// The adaptor, `None` for a null shared handle; `curve_ref.get()`.
    pub fn get(&self) -> Option<&S::Curve> {
        match self {
            Self::Borrowed(curve) => Some(*curve),
            Self::Shared(handle) => handle.get(),
        }
    }
}
