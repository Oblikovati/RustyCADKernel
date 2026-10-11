//! Select checked-in native OCCT output without changing numeric comparisons.
// Run 38123916241 proves these host captures independently match Martensite.

/// Select the native host asset; `native_boolean_snapshot!("tkbo_cells_builder")`.
macro_rules! native_boolean_snapshot {
    ($probe:literal) => {{
        if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
            include_str!(concat!("fixtures/", $probe, ".macos.expected"))
        } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
            include_str!(concat!(
                "fixtures/",
                $probe,
                ".windows.expected"
            ))
        } else {
            include_str!(concat!("fixtures/", $probe, ".expected"))
        }
    }};
}

pub(crate) use native_boolean_snapshot;
