// Created on: 2013-01-17
// Created by: Kirill GAVRILOV
// Copyright (c) 2013-2014 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKernel/Standard/Standard_CLocaleSentry.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Standard_CLocaleSentry`: formatting and parsing in the "C" locale within a scope.
//!
//! Native code switches the calling thread to the C locale (`uselocale`, or `setlocale`
//! where POSIX.1-2008 is missing) so that `sprintf`/`atof`-style calls ignore the user's
//! locale, and restores the previous locale when the sentry is destroyed. Rust's number
//! formatting and parsing never consult a process locale, and every OCCT formatting
//! routine in this port already produces C-locale text, so there is nothing to switch:
//! the sentry is a scope marker and the locale object describes the fixed C conventions.

/// The "C" locale that `_l` style calls format with; `CLocaleSentry::get_c_locale().decimal_point()`.
#[derive(Debug, Eq, PartialEq)]
pub struct CLocale {
    name: &'static str,
}

impl CLocale {
    /// The locale name; `CLocaleSentry::get_c_locale().name() == "C"`.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// The radix character (`localeconv()->decimal_point`); `c_locale.decimal_point() == '.'`.
    pub fn decimal_point(&self) -> char {
        '.'
    }

    /// The digit-group separator, empty in the C locale; `c_locale.thousands_separator() == ""`.
    pub fn thousands_separator(&self) -> &'static str {
        ""
    }
}

static C_LOCALE: CLocale = CLocale { name: "C" };

/// Scope in which formatting uses the C locale; `let _sentry = CLocaleSentry::new();`.
#[derive(Debug)]
pub struct CLocaleSentry {
    // Keeps construction explicit, as the native class has no copy constructor.
    _scope: (),
}

impl CLocaleSentry {
    /// Enters the C locale scope (a no-op for locale-independent Rust formatting);
    /// `CLocaleSentry::new()`.
    pub fn new() -> Self {
        Self { _scope: () }
    }

    /// The process-wide C locale object; every call returns the same instance, as the
    /// native static does; `std::ptr::eq(CLocaleSentry::get_c_locale(), CLocaleSentry::get_c_locale())`.
    pub fn get_c_locale() -> &'static CLocale {
        &C_LOCALE
    }
}

impl Default for CLocaleSentry {
    fn default() -> Self {
        Self::new()
    }
}
