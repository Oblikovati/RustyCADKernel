//! Exact profile-bound native buffers; wider strict failures remain a diagnostic matrix.
pub struct TexturedNativeBuffers {
    pub mode: &'static str,
    pub host: (&'static [u8], &'static [u8]),
    pub software: (&'static [u8], &'static [u8]),
}
macro_rules! native_textured_case {
    ($mode:literal) => {
        TexturedNativeBuffers {
            mode: $mode,
            host: (
                include_bytes!(concat!(
                    "../fixtures/ais_textured_shape/native-amd/",
                    $mode,
                    ".rgba"
                )),
                include_bytes!(concat!(
                    "../fixtures/ais_textured_shape/native-amd/",
                    $mode,
                    ".depth"
                )),
            ),
            software: (
                include_bytes!(concat!(
                    "../fixtures/ais_textured_shape/native-software/",
                    $mode,
                    ".rgba"
                )),
                include_bytes!(concat!(
                    "../fixtures/ais_textured_shape/native-software/",
                    $mode,
                    ".depth"
                )),
            ),
        }
    };
}
pub const TEXTURED_NATIVE_BUFFERS: &[TexturedNativeBuffers] = &[
    native_textured_case!("basic"),
    native_textured_case!("bottom-up"),
    native_textured_case!("no-modulate"),
    native_textured_case!("map-off"),
    native_textured_case!("transparent"),
    native_textured_case!("triangles"),
    native_textured_case!("face-boundaries"),
    native_textured_case!("red"),
    native_textured_case!("unset-red"),
    native_textured_case!("gold"),
    native_textured_case!("unset-gold"),
    native_textured_case!("updated-pixels"),
    native_textured_case!("wireframe"),
    native_textured_case!("shaded"),
    native_textured_case!("bbox"),
    native_textured_case!("map-0"),
    native_textured_case!("map-1"),
    native_textured_case!("map-2"),
    native_textured_case!("map-3"),
    native_textured_case!("map-4"),
    native_textured_case!("map-5"),
    native_textured_case!("map-6"),
    native_textured_case!("map-7"),
    native_textured_case!("face-boundaries-meshed"),
];
