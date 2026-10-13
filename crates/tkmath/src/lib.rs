//! OCCT mathematical foundation types. Toolkit API and legacy-test acceptance remain in progress.
/// Whether out-of-line OCCT matrix checks are enabled in this TKMath build.
/// Consumers must read the dependency's profile because Cargo unifies features.
/// Example: `let checked_matrices = rustycad_tkmath::OCCT_DEBUG_CHECKS;`.
pub const OCCT_DEBUG_CHECKS: bool = cfg!(feature = "occt-debug-checks");

pub mod bnd;
pub mod bvh;
pub mod cslib;
pub mod elclib;
pub mod elslib;
pub mod gp;
pub mod math;
pub mod math_utils;
pub mod poly;
mod stream_format;
pub mod toploc;

pub mod math_root;

pub mod math_opt;

pub mod math_lin;

pub mod math_poly;

pub mod math_integ;

pub mod math_sys;

pub mod bsplclib;
pub mod bsplslib;
pub mod convert;
pub mod geom_abs;
pub mod plib;

/// Exact signs for extension topology; mapped OCCT predicates are unchanged.
pub mod predicates;
