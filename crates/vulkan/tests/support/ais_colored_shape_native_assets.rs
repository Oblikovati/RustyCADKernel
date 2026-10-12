//! Device-bound native ColoredShape buffers, independent of the oracle at test time.
pub struct ColoredNativeBuffers {
    pub mode: &'static str,
    pub host: (&'static [u8], &'static [u8]),
    pub software: (&'static [u8], &'static [u8]),
}
macro_rules! colored_native_case {
    ($mode:literal) => {
        ColoredNativeBuffers {
            mode: $mode,
            host: (
                include_bytes!(concat!(
                    "../fixtures/ais_colored_shape/host/",
                    $mode,
                    ".rgba"
                )),
                include_bytes!(concat!(
                    "../fixtures/ais_colored_shape/host/",
                    $mode,
                    ".depth"
                )),
            ),
            software: (
                include_bytes!(concat!(
                    "../fixtures/ais_colored_shape/lavapipe/",
                    $mode,
                    ".rgba"
                )),
                include_bytes!(concat!(
                    "../fixtures/ais_colored_shape/lavapipe/",
                    $mode,
                    ".depth"
                )),
            ),
        }
    };
}
pub const COLORED_NATIVE_BUFFERS: &[ColoredNativeBuffers] = &[
    colored_native_case!("wireframe"),
    colored_native_case!("shaded"),
    colored_native_case!("bbox"),
    colored_native_case!("custom-color"),
    colored_native_case!("hidden"),
    colored_native_case!("custom-width"),
    colored_native_case!("transparent"),
    colored_native_case!("updated"),
];
