//! Byte-exact retained native controls; final producer/profile receipts accompany these assets.
type OitBasicAsset = (
    &'static str,
    bool,
    &'static [u8],
    &'static [u8],
    &'static [u8],
    &'static [u8],
);
macro_rules! oit_basic_asset {
    ($method:literal, $fixed:literal, $name:literal) => {
        (
            $method,
            $fixed,
            include_bytes!(concat!("../fixtures/oit/host/", $name, ".rgba")).as_slice(),
            include_bytes!(concat!("../fixtures/oit/host/", $name, ".depth")).as_slice(),
            include_bytes!(concat!("../fixtures/oit/software/", $name, ".rgba")).as_slice(),
            include_bytes!(concat!("../fixtures/oit/software/", $name, ".depth")).as_slice(),
        )
    };
}
pub const OIT_BASIC_ASSETS: &[OitBasicAsset] = &[
    oit_basic_asset!("unordered", false, "unordered-0"),
    oit_basic_asset!("weighted", false, "weighted-0"),
    oit_basic_asset!("peeling", false, "peeling-0"),
    oit_basic_asset!("unordered", true, "unordered-1"),
    oit_basic_asset!("weighted", true, "weighted-1"),
    oit_basic_asset!("peeling", true, "peeling-1"),
];

macro_rules! sprite_alpha_asset {
    ($mode:literal) => {
        (
            $mode,
            include_bytes!(concat!("../fixtures/oit/host/sprite-mask-", $mode, ".rgba")).as_slice(),
            include_bytes!(concat!(
                "../fixtures/oit/host/sprite-mask-",
                $mode,
                ".depth"
            ))
            .as_slice(),
            include_bytes!(concat!(
                "../fixtures/oit/software/sprite-mask-",
                $mode,
                ".rgba"
            ))
            .as_slice(),
            include_bytes!(concat!(
                "../fixtures/oit/software/sprite-mask-",
                $mode,
                ".depth"
            ))
            .as_slice(),
        )
    };
}
type OitSpriteAsset = (
    &'static str,
    &'static [u8],
    &'static [u8],
    &'static [u8],
    &'static [u8],
);
pub const SPRITE_ALPHA_ASSETS: &[OitSpriteAsset] = &[
    sprite_alpha_asset!("weighted"),
    sprite_alpha_asset!("peeling"),
];
