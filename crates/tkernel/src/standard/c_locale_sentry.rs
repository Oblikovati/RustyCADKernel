// Rust translation of FoundationClasses/TKernel/Standard/Standard_CLocaleSentry.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Standard_CLocaleSentry`: formatting and parsing in the "C" locale within a scope.
//!
//! Rust's number formatting and parsing never consult a process locale, but OCCT also
//! exposes this sentry to code that calls locale-sensitive C routines. POSIX targets
//! switch only the calling thread to `C` and restore its previous thread locale on drop.
//! Android keeps OCCT's native no-op behavior. Other targets retain the descriptive scope API;
//! their platform-specific locale switching is not ported.

#[cfg(all(unix, not(target_os = "android")))]
#[path = "c_locale_sentry/locale_posix.rs"]
mod locale_scope;
#[cfg(any(not(unix), target_os = "android"))]
#[path = "c_locale_sentry/locale_fallback.rs"]
mod locale_scope;

use std::marker::PhantomData;
use std::rc::Rc;

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

/// Scope in which locale-sensitive C routines use the C locale; `CLocaleSentry::new()`.
#[derive(Debug)]
pub struct CLocaleSentry {
    // A sentry must be dropped on the thread whose prior locale it captured.
    _scope: locale_scope::ThreadLocaleScope,
    _thread_bound: PhantomData<Rc<()>>,
}

impl CLocaleSentry {
    /// Enters the C locale scope and restores the prior thread locale on drop;
    /// panics if the locale cannot be activated. Example: `let _sentry = CLocaleSentry::new();`.
    pub fn new() -> Self {
        Self {
            _scope: locale_scope::ThreadLocaleScope::enter(),
            _thread_bound: PhantomData,
        }
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
