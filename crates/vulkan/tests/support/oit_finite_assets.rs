//! Independently generated finite OIT glyph/cap inputs; strict raster diagnostics stay raw.
pub struct OitFiniteAsset {
    pub fixture: &'static str,
    pub method: &'static str,
    pub samples: i32,
    pub fixed: bool,
    pub host: (&'static [u8], &'static [u8]),
    pub software: (&'static [u8], &'static [u8]),
}
macro_rules! finite_asset {
    ($fixture:literal,$method:literal,$samples:literal,$fixed:literal,$name:literal) => {
        OitFiniteAsset {
            fixture: $fixture,
            method: $method,
            samples: $samples,
            fixed: $fixed,
            host: (
                include_bytes!(concat!("../fixtures/oit/host/", $name, ".rgba")),
                include_bytes!(concat!("../fixtures/oit/host/", $name, ".depth")),
            ),
            software: (
                include_bytes!(concat!("../fixtures/oit/software/", $name, ".rgba")),
                include_bytes!(concat!("../fixtures/oit/software/", $name, ".depth")),
            ),
        }
    };
}
pub const OIT_FINITE_ASSETS: &[OitFiniteAsset] = &[
    finite_asset!(
        "text-colored",
        "unordered",
        0,
        false,
        "text-colored-text-colored-unordered-0-0"
    ),
    finite_asset!(
        "text-colored",
        "unordered",
        0,
        true,
        "text-colored-text-colored-unordered-0-1"
    ),
    finite_asset!(
        "text-colored",
        "weighted",
        0,
        false,
        "text-colored-text-colored-weighted-0-0"
    ),
    finite_asset!(
        "text-colored",
        "weighted",
        0,
        true,
        "text-colored-text-colored-weighted-0-1"
    ),
    finite_asset!(
        "text-colored",
        "peeling",
        0,
        false,
        "text-colored-text-colored-peeling-0-0"
    ),
    finite_asset!(
        "text-colored",
        "peeling",
        0,
        true,
        "text-colored-text-colored-peeling-0-1"
    ),
    finite_asset!(
        "cap-source-mask-alpha-updated",
        "unordered",
        0,
        true,
        "cap-live-cap-source-mask-alpha-updated-unordered-0-1"
    ),
    finite_asset!(
        "cap-source-mask-alpha-updated",
        "unordered",
        4,
        true,
        "cap-live-cap-source-mask-alpha-updated-unordered-4-1"
    ),
    finite_asset!(
        "cap-source-mask-alpha-updated",
        "weighted",
        0,
        true,
        "cap-live-cap-source-mask-alpha-updated-weighted-0-1"
    ),
    finite_asset!(
        "cap-source-mask-alpha-updated",
        "weighted",
        4,
        true,
        "cap-live-cap-source-mask-alpha-updated-weighted-4-1"
    ),
    finite_asset!(
        "cap-source-mask-alpha-updated",
        "peeling",
        0,
        true,
        "cap-live-cap-source-mask-alpha-updated-peeling-0-1"
    ),
    finite_asset!(
        "cap-source-mask-alpha-updated",
        "peeling",
        4,
        true,
        "cap-live-cap-source-mask-alpha-updated-peeling-4-1"
    ),
    finite_asset!(
        "cap-object-texture-blend-alpha-updated",
        "unordered",
        0,
        true,
        "cap-live-cap-object-texture-blend-alpha-updated-unordered-0-1"
    ),
    finite_asset!(
        "cap-object-texture-blend-alpha-updated",
        "weighted",
        0,
        true,
        "cap-live-cap-object-texture-blend-alpha-updated-weighted-0-1"
    ),
    finite_asset!(
        "cap-object-texture-blend-alpha-updated",
        "peeling",
        0,
        true,
        "cap-live-cap-object-texture-blend-alpha-updated-peeling-0-1"
    ),
];
