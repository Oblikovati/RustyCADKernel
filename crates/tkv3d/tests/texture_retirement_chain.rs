//! Every nested EndScope boundary must retain callback-published native ownership loss.
#[path = "support/texture_retirement_chain_fixture.rs"]
mod chain_fixture;
use chain_fixture::*;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{standard::*, tcollection::AsciiString};
use rustycad_tkservice::image::ImagePixMap;
use rustycad_tkv3d::ais::*;
use std::sync::Arc;
#[test]
fn texture_retirement_two_destructors_preserve_all_native_lost_references() {
    let trace = Arc::new(SharedState::new(Vec::new()));
    run_texture_chain(false, &trace);
    run_texture_chain(true, &trace);
    assert_eq!(trace.read().join("\n"), include_str!("fixtures/ais_textured_shape/native-retirement-chain.txt").trim_end());
}
fn run_texture_chain(outer_file: bool, trace: &Arc<SharedState<Vec<String>>>) {
    let owner = AisTexturedShape::new(&Shape::default());
    let control = Arc::new(SharedState::new(TextureChainControl::default()));
    seed_texture_chain(&owner, &control, trace);
    let mut old = texture_chain_pixels(&owner, &control, trace, 1);
    let next = Handle::new(ImagePixMap::default());
    owner.set_texture_pix_map(&old);
    old.nullify();
    if outer_file {
        owner.set_texture_file_name(&AsciiString::from("outer.raw")).unwrap();
    } else {
        owner.set_texture_pix_map(&next);
    }
    record_texture_chain_after(&owner, &control, &next, trace, outer_file);
}
fn seed_texture_chain(owner: &Handle<AisTexturedShape>, control: &Arc<SharedState<TextureChainControl>>, trace: &Arc<SharedState<Vec<String>>>) {
    let nested = texture_chain_pixels(owner, control, trace, 2);
    let final_pixels = texture_chain_pixels(owner, control, trace, 3);
    let mut fields = control.write();
    fields.last = final_pixels.downgrade();
    fields.nested = nested;
    fields.final_pixels = final_pixels;
}
fn record_texture_chain_after(owner: &AisTexturedShape, control: &SharedState<TextureChainControl>, next: &Handle<ImagePixMap>, trace: &SharedState<Vec<String>>, outer_file: bool) {
    let count = control.read().last.observe().ref_count();
    let file = owner.texture_file().read().unwrap();
    trace.write().push(format!("AFTER {} {} {} {count} {}", i32::from(outer_file),
        i32::from(owner.texture_pix_map().is_null()), next.ref_count(), file.to_string_lossy()).trim_end().to_string());
}
