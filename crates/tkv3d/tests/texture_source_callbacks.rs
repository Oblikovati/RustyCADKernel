//! Real pixmap destructor callbacks preserve native retiring pointer and incoming ownership order.
#[path = "support/texture_source_callback_fixture.rs"]
mod callback_fixture;
use callback_fixture::*;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{standard::*, tcollection::AsciiString};
use rustycad_tkservice::image::ImagePixMap;
use rustycad_tkv3d::ais::*;
use std::sync::Arc;
#[test]
fn texture_source_filename_destructor_getter_matches_native_retiring_slot() {
    let trace = Arc::new(SharedState::new(Vec::new()));
    run_retiring_texture_case("file-sole", false, false, &trace);
    assert_eq!(
        trace.read()[0],
        "MARTENSITE_TEXTURE_SLOT file-sole 1 0 0 -1 0 prior.raw"
    );
}
fn run_retiring_texture_case(
    label: &str,
    replace: bool,
    keep_owner: bool,
    trace: &Arc<SharedState<Vec<String>>>,
) {
    let owner = AisTexturedShape::new(&Shape::default());
    let incoming = texture_incoming_pixels(replace);
    let mut pixels = callback_texture_pixels(&owner, &incoming, label, trace);
    owner.set_texture_pix_map(&pixels);
    seed_callback_texture_file(&owner);
    let alias = owner.texture_pix_map();
    let slot_address = alias.reference_address();
    if !keep_owner {
        pixels.nullify();
    }
    replace_callback_texture_source(&owner, &incoming, replace);
    record_texture_after(&owner, &incoming, label, slot_address, trace);
    pixels.nullify();
}
fn texture_incoming_pixels(replace: bool) -> Handle<ImagePixMap> {
    if replace {
        return Handle::new(ImagePixMap::default());
    }
    Handle::default()
}
fn replace_callback_texture_source(
    owner: &AisTexturedShape,
    incoming: &Handle<ImagePixMap>,
    replace: bool,
) {
    if replace {
        owner.set_texture_pix_map(incoming);
        return;
    }
    owner
        .set_texture_file_name(&AsciiString::from("next.raw"))
        .unwrap();
}
fn record_texture_after(
    owner: &AisTexturedShape,
    incoming: &Handle<ImagePixMap>,
    label: &str,
    slot_address: *const (),
    trace: &Arc<SharedState<Vec<String>>>,
) {
    let alias = owner.texture_pix_map();
    let same = alias.reference_address() == slot_address;
    let count = texture_incoming_count(incoming);
    let prefix = format!(
        "MARTENSITE_TEXTURE_AFTER {label} {} {} {count}",
        i32::from(same),
        i32::from(alias.is_null())
    );
    trace.write().push(texture_state_suffix(owner, prefix));
}
fn run_same_texture_slot(trace: &Arc<SharedState<Vec<String>>>) {
    let owner = AisTexturedShape::new(&Shape::default());
    let mut pixels = callback_texture_pixels(&owner, &Handle::default(), "same-slot", trace);
    owner.set_texture_pix_map(&pixels);
    pixels.nullify();
    seed_callback_texture_file(&owner);
    let alias = owner.texture_pix_map();
    let address = alias.object_address();
    owner.set_texture_pix_map(&alias);
    let prefix = format!(
        "MARTENSITE_TEXTURE_SAME {} {}",
        i32::from(alias.object_address() == address),
        alias.ref_count()
    );
    trace.write().push(texture_state_suffix(&owner, prefix));
    owner
        .set_texture_file_name(&AsciiString::from("cleanup.raw"))
        .unwrap();
}
#[test]
fn texture_source_actual_callbacks_replay_all_native_retirement_records() {
    let trace = Arc::new(SharedState::new(Vec::new()));
    run_retiring_texture_case("file-sole", false, false, &trace);
    run_retiring_texture_case("pixmap-sole", true, false, &trace);
    run_retiring_texture_case("file-retained", false, true, &trace);
    run_retiring_texture_case("pixmap-retained", true, true, &trace);
    run_same_texture_slot(&trace);
    run_incoming_texture_slot_mutation(&trace);
    let actual = trace.read().join("\n") + "\n";
    let native = include_str!("fixtures/ais_textured_shape/native-source-callbacks.txt");
    assert_eq!(actual, native);
}

fn texture_incoming_count(incoming: &Handle<ImagePixMap>) -> i32 {
    if incoming.is_null() {
        return -1;
    }
    incoming.ref_count()
}

fn run_incoming_texture_slot_mutation(trace: &Arc<SharedState<Vec<String>>>) {
    let owner = AisTexturedShape::new(&Shape::default());
    let input = AisTexturedShape::new(&Shape::default());
    let original = Handle::new(ImagePixMap::default());
    let next = Handle::new(ImagePixMap::default());
    input.set_texture_pix_map(&original);
    install_mutating_texture_pixels(&owner, &input, &original, &next, trace);
    let alias = input.texture_pix_map();
    owner.set_texture_pix_map(&alias);
    let same = owner.texture_pix_map().object_address() == original.object_address();
    let changed = alias.object_address() == next.object_address();
    trace.write().push(format!(
        "MARTENSITE_TEXTURE_CAPTURED {} {} {} {}",
        i32::from(same),
        original.ref_count(),
        i32::from(changed),
        next.ref_count()
    ));
}
fn install_mutating_texture_pixels(
    owner: &Handle<AisTexturedShape>,
    input: &Handle<AisTexturedShape>,
    original: &Handle<ImagePixMap>,
    next: &Handle<ImagePixMap>,
    trace: &Arc<SharedState<Vec<String>>>,
) {
    let input = input.downgrade();
    let next = next.downgrade();
    let callback = move || input.observe().set_texture_pix_map(&*next.observe());
    let pixels = Handle::new(TextureSlotCallbackPixels {
        base: ImagePixMap::default(),
        owner: owner.downgrade(),
        incoming: original.downgrade(),
        label: "incoming-slot".to_string(),
        trace: trace.clone(),
        callback: Some(Box::new(callback)),
    });
    owner.set_texture_pix_map(&pixels.cast());
}
