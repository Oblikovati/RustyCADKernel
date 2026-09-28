// Copyright (c) 2026 OPEN CASCADE SAS
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

/// Optional finite bounds and cached endpoint values; `MinBracketOptions::default()`.
#[derive(Debug, Clone)]
pub struct MinBracketOptions {
    /// Native `max_iterations` field; access with `result.max_iterations`.
    pub max_iterations: u32,
    /// Native `use_limits` field; access with `result.use_limits`.
    pub use_limits: bool,
    /// Native `left_limit` field; access with `result.left_limit`.
    pub left_limit: f64,
    /// Native `right_limit` field; access with `result.right_limit`.
    pub right_limit: f64,
    /// Native `has_fa` field; access with `result.has_fa`.
    pub has_fa: bool,
    /// Native `has_fb` field; access with `result.has_fb`.
    pub has_fb: bool,
    /// Native `fa` field; access with `result.fa`.
    pub fa: f64,
    /// Native `fb` field; access with `result.fb`.
    pub fb: f64,
}
impl Default for MinBracketOptions {
    fn default() -> Self {
        Self {
            max_iterations: 50,
            use_limits: false,
            left_limit: 0.0,
            right_limit: 0.0,
            has_fa: false,
            has_fb: false,
            fa: 0.0,
            fb: 0.0,
        }
    }
}
