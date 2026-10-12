//! Public native option fields must remain directly mutable, without ToPixMap validation.
use rustycad_tkservice::graphic3d::Graphic3dBufferType;
use rustycad_tkv3d::v3d::{V3dImageDumpOptions, V3dStereoDumpOptions};

const DUMP_BUFFER_VARIANTS: [Graphic3dBufferType; 6] = [
    Graphic3dBufferType::Rgb,
    Graphic3dBufferType::Rgba,
    Graphic3dBufferType::Depth,
    Graphic3dBufferType::RgbRayTraceHdrLeft,
    Graphic3dBufferType::Red,
    Graphic3dBufferType::ShadowMap,
];
const DUMP_STEREO_VARIANTS: [V3dStereoDumpOptions; 4] = [
    V3dStereoDumpOptions::Mono,
    V3dStereoDumpOptions::LeftEye,
    V3dStereoDumpOptions::RightEye,
    V3dStereoDumpOptions::Blended,
];

fn emit_options(label: &str, options: &V3dImageDumpOptions) -> String {
    format!(
        "{label} {} {} {} {} {} {} {} {}\n",
        options.width,
        options.height,
        options.buffer_type as i32,
        options.stereo_options as i32,
        options.tile_size,
        i32::from(options.to_adjust_aspect),
        options.target_z_layer_id,
        i32::from(options.is_single_layer)
    )
}

fn assign_options(options: &mut V3dImageDumpOptions, index: i32) {
    options.width = -index;
    options.height = 640 + index;
    options.buffer_type = DUMP_BUFFER_VARIANTS[index as usize % DUMP_BUFFER_VARIANTS.len()];
    options.stereo_options = DUMP_STEREO_VARIANTS[index as usize % DUMP_STEREO_VARIANTS.len()];
    options.tile_size = -20 + index;
    options.to_adjust_aspect = index % 2 != 0;
    options.target_z_layer_id = -100 + index;
    options.is_single_layer = index % 2 == 0;
}

#[test]
fn native_public_value_fields_replay() {
    let mut options = V3dImageDumpOptions::default();
    assert!(options.light_name.is_empty());
    let mut actual = emit_options("default", &options);
    for index in 0..12 {
        assign_options(&mut options, index);
        actual.push_str(&emit_options("assigned", &options));
    }
    let copy = options.clone();
    assign_options(&mut options, 42);
    actual.push_str(&emit_options("copy", &copy));
    actual.push_str(&emit_options("original", &options));
    assert_eq!(
        actual,
        include_str!("fixtures/image-dump-options-native.txt")
    );
}
