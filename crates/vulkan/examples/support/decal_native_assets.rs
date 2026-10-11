//! Content-addressed actual native buffers avoid duplicating identical formatter/alpha scenes.
type NativeTextCapture = (&'static str, &'static str, &'static [u8], &'static [u8]);
macro_rules! native_text_asset {
    ($family:literal, $mode:literal, $rgba:literal, $depth:literal) => {
        (
            $family,
            $mode,
            include_bytes!(concat!(
                "../../tests/fixtures/decal_text/native-host/",
                $rgba
            )),
            include_bytes!(concat!(
                "../../tests/fixtures/decal_text/native-host/",
                $depth
            )),
        )
    };
}
#[path = "decal_native_decals.rs"]
mod decal_native_decals;
#[path = "decal_native_shadows.rs"]
mod decal_native_shadows;
#[path = "decal_native_subtitles.rs"]
mod decal_native_subtitles;
const NATIVE_TEXT_FAMILIES: [&[NativeTextCapture]; 3] = [
    &decal_native_decals::NATIVE_DECALS,
    &decal_native_shadows::NATIVE_SHADOWS,
    &decal_native_subtitles::NATIVE_SUBTITLES,
];
/// Select actual AMD OpenGL buffers; `native_text_capture("decal", "decal-basic")` links no OCCT library.
pub fn native_text_capture(family: &str, mode: &str) -> (&'static [u8], &'static [u8]) {
    let entry = NATIVE_TEXT_FAMILIES
        .iter()
        .flat_map(|family| family.iter())
        .find(|entry| entry.0 == family && entry.1 == mode)
        .unwrap_or_else(|| {
            panic!("family {family} mode {mode}; expected a captured native text variant")
        });
    (entry.2, entry.3)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_native_text_variants_keep_complete_color_and_depth_shape() {
        for entry in NATIVE_TEXT_FAMILIES.iter().flat_map(|family| family.iter()) {
            let (rgba, depth) = native_text_capture(entry.0, entry.1);
            assert_eq!(rgba.len(), 128 * 128 * 4);
            assert_eq!(depth.len(), rgba.len());
        }
    }
}
