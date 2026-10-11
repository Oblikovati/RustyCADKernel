//! Device-bound actual OpenGl_Text assets keep strict native color/depth replay independent of OCCT.
type SubtitleNativePair = (&'static [u8], &'static [u8]);
macro_rules! subtitle_asset_pair {
    ($profile:literal, $mode:literal) => {{
        (
            include_bytes!(concat!(
                "../../tests/fixtures/subtitle_text/",
                $profile,
                "/",
                $mode,
                ".rgba"
            ))
            .as_slice(),
            include_bytes!(concat!(
                "../../tests/fixtures/subtitle_text/",
                $profile,
                "/",
                $mode,
                ".depth"
            ))
            .as_slice(),
        )
    }};
}
const HOST_ASSETS: [SubtitleNativePair; 25] = [
    subtitle_asset_pair!("host", "subtitle-basic"),
    subtitle_asset_pair!("host", "subtitle-spaces"),
    subtitle_asset_pair!("host", "subtitle-tabs"),
    subtitle_asset_pair!("host", "subtitle-multiline"),
    subtitle_asset_pair!("host", "subtitle-centered"),
    subtitle_asset_pair!("host", "subtitle-right"),
    subtitle_asset_pair!("host", "subtitle-top"),
    subtitle_asset_pair!("host", "subtitle-bottom"),
    subtitle_asset_pair!("host", "subtitle-zero-alpha"),
    subtitle_asset_pair!("host", "subtitle-translucent"),
    subtitle_asset_pair!("host", "subtitle-opaque-translucent"),
    subtitle_asset_pair!("host", "subtitle-annotation"),
    subtitle_asset_pair!("host", "subtitle-back-culled"),
    subtitle_asset_pair!("host", "subtitle-right-back-culled"),
    subtitle_asset_pair!("host", "subtitle-front-culled"),
    subtitle_asset_pair!("host", "subtitle-mirror"),
    subtitle_asset_pair!("host", "subtitle-view-double"),
    subtitle_asset_pair!("host", "subtitle-offset"),
    subtitle_asset_pair!("host", "subtitle-zoomable"),
    subtitle_asset_pair!("host", "subtitle-resolution"),
    subtitle_asset_pair!("host", "subtitle-empty"),
    subtitle_asset_pair!("host", "subtitle-whitespace"),
    subtitle_asset_pair!("host", "subtitle-zero-alpha-depth"),
    subtitle_asset_pair!("host", "subtitle-zero-alpha-no-depth-write"),
    subtitle_asset_pair!("host", "subtitle-annotation-depth"),
];
/// Keep native device provenance explicit; `subtitle_native_pair("host", 0)` selects the actual AMD OpenGL fixture.
pub fn subtitle_native_pair(profile: &str, index: usize) -> SubtitleNativePair {
    match profile {
        "host" => HOST_ASSETS[index],
        _ => panic!("profile {profile}; expected host (actual native AMD OpenGL)"),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_subtitle_assets_keep_complete_shape_on_hardware_profile() {
        for index in 0..25 {
            let (rgba, depth) = subtitle_native_pair("host", index);
            assert_eq!(rgba.len(), 128 * 128 * 4);
            assert_eq!(depth.len(), rgba.len());
        }
    }
}
