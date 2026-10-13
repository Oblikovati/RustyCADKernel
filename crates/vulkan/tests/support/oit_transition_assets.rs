//! Native same-view sequence; six/eight-sample coverage diagnostics retain their own raw assets.
pub struct NativeOitTransition {
    pub name: &'static str,
    pub mode: &'static str,
    pub samples: i32,
    pub strict: bool,
    pub host: (&'static [u8], &'static [u8]),
    pub software: (&'static [u8], &'static [u8]),
}
macro_rules! transition_asset {
    ($name:literal,$mode:literal,$samples:literal,$strict:literal) => {
        NativeOitTransition {
            name: $name,
            mode: $mode,
            samples: $samples,
            strict: $strict,
            host: (
                include_bytes!(concat!("../fixtures/oit/transitions/host/", $name, ".rgba")),
                include_bytes!(concat!(
                    "../fixtures/oit/transitions/host/",
                    $name,
                    ".depth"
                )),
            ),
            software: (
                include_bytes!(concat!(
                    "../fixtures/oit/transitions/software/",
                    $name,
                    ".rgba"
                )),
                include_bytes!(concat!(
                    "../fixtures/oit/transitions/software/",
                    $name,
                    ".depth"
                )),
            ),
        }
    };
}
pub const NATIVE_OIT_TRANSITIONS: &[NativeOitTransition] = &[
    transition_asset!("weighted-0", "weighted", 0, true),
    transition_asset!("weighted-1", "weighted", 1, true),
    transition_asset!("weighted-2", "weighted", 2, true),
    transition_asset!("weighted-3", "weighted", 3, true),
    transition_asset!("weighted-4", "weighted", 4, true),
    transition_asset!("weighted-6", "weighted", 6, false),
    transition_asset!("weighted-8", "weighted", 8, false),
    transition_asset!("peeling-4", "peeling", 4, true),
    transition_asset!("after-weighted-4", "weighted", 4, true),
];
