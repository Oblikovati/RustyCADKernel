//! Attributed native buffers are immutable inputs; replay never loads OCCT.
#[allow(dead_code)]
#[path = "oit_scene.rs"]
mod oit_scene;
#[allow(dead_code)]
#[path = "viewer_scene.rs"]
mod scene;
#[path = "ordered_stencil_scene.rs"]
mod stencil_scene;
#[allow(dead_code)]
#[path = "surface_texture_scene.rs"]
mod surface_texture_scene;
#[allow(dead_code)]
#[path = "../../examples/support/view_capture.rs"]
mod view_capture;
#[path = "view_setup.rs"]
mod view_setup;
use surface_texture_scene as texture_scene;
#[allow(dead_code)]
#[path = "capping_texture_scene.rs"]
mod capping_texture_scene;
use capping_texture_scene::section_geometry as capping_scene;
#[path = "seeded_stencil_backend.rs"]
mod seeded_backend;
use martensite_vulkan::{Graphic3dRenderDriver, VulkanOptions};
use rustycad_tkservice::graphic3d::Graphic3dBufferType;

pub struct StencilNativeCase<'a> {
    pub mode: &'a str,
    pub method: &'a str,
    pub seed: u8,
    expected: [u32; 8],
    ordinal: usize,
}
/// Iterate the original87 accepted workloads; `for case in native_cases() { ... }`.
pub fn native_cases() -> impl Iterator<Item = StencilNativeCase<'static>> {
    include_str!("../fixtures/ordered_stencil/native-cases.txt")
        .lines()
        .enumerate()
        .map(|(ordinal, line)| parse_native_case(ordinal, line))
}
fn parse_native_case(ordinal: usize, line: &str) -> StencilNativeCase<'_> {
    let fields: Vec<_> = line.split_whitespace().collect();
    assert_eq!(
        fields.len(),
        11,
        "received {line:?}; expected mode method seed and eight state fields"
    );
    StencilNativeCase {
        mode: fields[0],
        method: fields[1],
        seed: fields[2].parse().unwrap(),
        expected: std::array::from_fn(|i| fields[i + 3].parse().unwrap()),
        ordinal,
    }
}
/// Replay actual retained geometry and every native gate; `replay_case(&case, options, false)`.
pub fn replay_case(
    case: &StencilNativeCase<'_>,
    options: VulkanOptions,
    measure: bool,
) -> Vec<f64> {
    let backend = seeded_backend::SeededStencilBackend::new(options, case.seed).unwrap();
    let identity = backend.driver_identity();
    eprintln!("MARTENSITE_DEVICE_IDENTITY={identity:?}");
    let driver = Graphic3dRenderDriver::with_backend(backend, 128).unwrap();
    let fixture = stencil_scene::OrderedStencilScene::new(&driver, case.mode, case.method);
    fixture.initial_redraw(case.mode).unwrap();
    assert_native_stencil_capture(case, &fixture, &identity.device_name);
    let samples = measure_stencil_redraws(&fixture, measure);
    assert_native_stencil_capture(case, &fixture, &identity.device_name);
    fixture.finish();
    samples
}
fn measure_stencil_redraws(
    fixture: &stencil_scene::OrderedStencilScene,
    measure: bool,
) -> Vec<f64> {
    if !measure {
        return Vec::new();
    }
    (0..120)
        .filter_map(|index| {
            let start = std::time::Instant::now();
            fixture.fixture.scene.render_view().try_redraw().unwrap();
            (index >= 20).then(|| start.elapsed().as_secs_f64() * 1000.)
        })
        .collect()
}
fn assert_native_stencil_capture(
    case: &StencilNativeCase<'_>,
    fixture: &stencil_scene::OrderedStencilScene,
    device: &str,
) {
    let expected = native_stencil_profile(device);
    let start = case.ordinal * 32768;
    let full = case.mode.starts_with("layer-");
    let view = &fixture.fixture.scene.view;
    let rgba = view_capture::capture_buffer(view, 64, 64, full, Graphic3dBufferType::Rgba).unwrap();
    let depth =
        view_capture::capture_buffer(view, 64, 64, full, Graphic3dBufferType::Depth).unwrap();
    assert_stencil_color(&rgba, &expected[start..start + 16384], case);
    assert_stencil_depth(&depth, &expected[start + 16384..start + 32768]);
    let actual = fixture.fixture.scene.render_view().stencil_context_state();
    assert_eq!(actual.native_state_fields(), case.expected);
}
fn assert_stencil_color(actual: &[u8], expected: &[u8], case: &StencilNativeCase<'_>) {
    assert_eq!(
        actual, expected,
        "{} {} seed{}",
        case.mode, case.method, case.seed
    );
}

fn native_stencil_profile(device: &str) -> &'static [u8] {
    if device.contains("llvmpipe") {
        return include_bytes!("../fixtures/ordered_stencil/software-native.bin");
    }
    assert!(
        device.contains("AMD"),
        "received device={device:?}; expected proved AMD or llvmpipe"
    );
    include_bytes!("../fixtures/ordered_stencil/host-native.bin")
}
fn assert_stencil_depth(actual: &[u8], expected: &[u8]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual
        .as_chunks::<4>()
        .0
        .iter()
        .zip(expected.as_chunks::<4>().0)
    {
        let actual = f32::from_le_bytes(*actual);
        let expected = f32::from_le_bytes(*expected);
        assert!(
            (actual - expected).abs() <= 1e-6,
            "received depth={actual}; expected {expected} within1e-6"
        );
    }
}
