//! Packed native image-format fixtures bypass codecs and retain actual component storage.
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::image::{ImageFormat, ImagePixMap};
macro_rules! pixels {
    ($name:literal) => {
        include_bytes!(concat!("../fixtures/surface_textures/", $name, ".pixels")).as_slice()
    };
}
struct SurfaceFormatFixture {
    name: &'static str,
    format: ImageFormat,
    bytes: &'static [u8],
}
macro_rules! image_fixture {
    ($name:literal,$format:ident) => {
        SurfaceFormatFixture {
            name: $name,
            format: ImageFormat::$format,
            bytes: pixels!($name),
        }
    };
}
const FORMATS: &[SurfaceFormatFixture] = &[
    image_fixture!("cutoff", Rgbaf),
    image_fixture!("gray", Gray),
    image_fixture!("alpha", Alpha),
    image_fixture!("rgb", Rgb),
    image_fixture!("bgr", Bgr),
    image_fixture!("rgb32", Rgb32),
    image_fixture!("bgr32", Bgr32),
    image_fixture!("rgba", Rgba),
    image_fixture!("bgra", Bgra),
    image_fixture!("gray-f", GrayF),
    image_fixture!("alpha-f", AlphaF),
    image_fixture!("rg-f", Rgf),
    image_fixture!("rgb-f", Rgbf),
    image_fixture!("bgr-f", Bgrf),
    image_fixture!("rgba-f", Rgbaf),
    image_fixture!("bgra-f", Bgraf),
    image_fixture!("gray-half", GrayFHalf),
    image_fixture!("rg-half", RgfHalf),
    image_fixture!("rgba-half", RgbafHalf),
    image_fixture!("gray16", Gray16),
];
/// Load packed native channels without decoding; `surface_format_image("format-rgba")`.
pub fn surface_format_image(mode: &str) -> Option<Handle<ImagePixMap>> {
    let name = if mode == "mask-exact" {
        "cutoff"
    } else {
        mode.strip_prefix("format-")?
    };
    let fixture = FORMATS
        .iter()
        .find(|fixture| fixture.name == name)
        .unwrap_or_else(|| {
            panic!("received image format={name:?}; expected native sampled 2D fixture format")
        });
    let image = Handle::new(ImagePixMap::default());
    assert!(image.init_wrapper(
        fixture.format,
        Some(fixture.bytes.to_vec()),
        2,
        2,
        fixture.bytes.len() / 2
    ));
    image.set_top_down(true);
    Some(image)
}
