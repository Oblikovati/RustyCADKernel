//! Renderer validation gates run on the pinned layer, never on whichever package the host installed.
use martensite_vulkan::{VERIFIED_VALIDATION_LAYER, active_validation_layer};

// Regression for the toolkit SIGSEGV (2026-10-15): with Ubuntu's layer 1.3.275 the parallel toolkit
// target crashed inside libVkLayer_khronos_validation.so in about one of ten runs on lavapipe and AMD,
// and reported a false MSAA depth-resolve hazard; the pinned 1.4.313 layer passed 80 parallel repeats.
#[test]
fn toolkit_validation_uses_the_verified_layer_dependency() {
    let active = active_validation_layer().unwrap();
    assert!(
        active >= Some(VERIFIED_VALIDATION_LAYER),
        "active validation layer {active:?}; expected at least {VERIFIED_VALIDATION_LAYER:?} \
         selected by the cargo runner tools/with_vulkan_validation.py (.cargo/config.toml)"
    );
}
