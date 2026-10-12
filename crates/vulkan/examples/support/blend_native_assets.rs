//! Actual native XOR buffers remain Rust-only and retain strict exact-RGBA/depth gates.
type NativeBlendCapture = (&'static str, &'static [u8], &'static [u8]);
#[path = "blend_native_basic.rs"]
mod basic;
#[path = "blend_native_extras.rs"]
mod extras;
#[path = "blend_native_pixel.rs"]
mod pixel;
/// Select actual pinned buffers; `blend_capture("decal-basic", false)` links no OCCT.
pub fn blend_capture(mode: &str, is_pixel: bool) -> (&'static [u8], &'static [u8]) {
    let entries = if is_pixel {
        pixel::ASSETS.as_slice()
    } else {
        basic::ASSETS.as_slice()
    };
    let entry = entries
        .iter()
        .chain(extras::ASSETS.iter().filter(|_| !is_pixel))
        .find(|entry| entry.0 == mode)
        .unwrap_or_else(|| {
            panic!("mode {mode}, pixel={is_pixel}; expected committed native Blend fixture")
        });
    (entry.1, entry.2)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_native_capture_shapes_and_family_identities_are_preserved() {
        for (is_pixel, assets) in [
            (false, basic::ASSETS.as_slice()),
            (false, extras::ASSETS.as_slice()),
            (true, pixel::ASSETS.as_slice()),
        ] {
            for asset in assets {
                let (rgba, depth) = blend_capture(asset.0, is_pixel);
                assert_eq!(rgba.len(), 128 * 128 * 4);
                assert_eq!(depth.len(), rgba.len());
                assert_eq!(rgba, asset.1);
            }
        }
    }
}
