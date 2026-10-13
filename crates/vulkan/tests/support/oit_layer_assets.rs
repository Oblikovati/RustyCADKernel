//! Actual viewer layer sharing/flush assets replay independently of OCCT.
pub struct OitLayerAsset {
    pub fixture: &'static str,
    pub mode: &'static str,
    pub fixed: bool,
    pub host: (&'static [u8], &'static [u8]),
    pub software: (&'static [u8], &'static [u8]),
}
macro_rules! layer_asset {
    ($fixture:literal,$mode:literal,$fixed:literal,$name:literal) => {
        OitLayerAsset {
            fixture: $fixture,
            mode: $mode,
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
pub const OIT_LAYER_ASSETS: &[OitLayerAsset] = &[
    layer_asset!("layer-clear", "unordered", false, "layer-clear-unordered-0"),
    layer_asset!("layer-clear", "unordered", true, "layer-clear-unordered-1"),
    layer_asset!("layer-clear", "weighted", false, "layer-clear-weighted-0"),
    layer_asset!("layer-clear", "weighted", true, "layer-clear-weighted-1"),
    layer_asset!("layer-clear", "peeling", false, "layer-clear-peeling-0"),
    layer_asset!("layer-clear", "peeling", true, "layer-clear-peeling-1"),
    layer_asset!(
        "layer-shared",
        "unordered",
        false,
        "layer-shared-unordered-0"
    ),
    layer_asset!(
        "layer-shared",
        "unordered",
        true,
        "layer-shared-unordered-1"
    ),
    layer_asset!("layer-shared", "weighted", false, "layer-shared-weighted-0"),
    layer_asset!("layer-shared", "weighted", true, "layer-shared-weighted-1"),
    layer_asset!("layer-shared", "peeling", false, "layer-shared-peeling-0"),
    layer_asset!("layer-shared", "peeling", true, "layer-shared-peeling-1"),
];
