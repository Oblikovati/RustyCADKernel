//! Native count-zero pointee access and nested assignment must retain actual slots and counts.
#[path = "support/texture_retirement_fixture.rs"]
mod retirement_fixture;
use retirement_fixture::*;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{standard::*, tcollection::AsciiString};
use rustycad_tkservice::image::ImagePixMap;
use rustycad_tkv3d::ais::*;
use std::sync::Arc;
#[test]
fn texture_retirement_actual_getter_projects_count_zero_pointee() {
    let trace = Arc::new(SharedState::new(Vec::new()));
    run_projection_case("none", false, &trace);
    assert_eq!(trace.read()[0], "POINTEE none 1 0 3 2 73");
}
#[test]
fn texture_retirement_native_nested_matrix_matches_all_actual_records() {
    let trace = Arc::new(SharedState::new(Vec::new()));
    for mode in ["none", "same", "file", "pixmap"] {
        run_projection_case(mode, false, &trace);
        run_projection_case(mode, true, &trace);
    }
    assert_eq!(trace.read().join("\n"), include_str!("fixtures/ais_textured_shape/native-retirement-projection.txt").trim_end());
}
fn run_projection_case(mode: &'static str, outer_file: bool, trace: &Arc<SharedState<Vec<String>>>) {
    let owner = AisTexturedShape::new(&Shape::default());
    let next = Handle::new(ImagePixMap::default());
    let nested = Handle::new(ImagePixMap::default());
    let mut old = retirement_projection_pixels(&owner, &nested, mode, trace);
    owner.set_texture_pix_map(&old);
    old.nullify();
    if outer_file {
        owner.set_texture_file_name(&AsciiString::from("outer.raw")).unwrap();
    } else {
        owner.set_texture_pix_map(&next);
    }
    trace.write().push(retirement_after_row(&owner, &next, &nested, mode, outer_file));
}
fn retirement_after_row(owner: &AisTexturedShape, next: &Handle<ImagePixMap>, nested: &Handle<ImagePixMap>, mode: &str, outer_file: bool) -> String {
    let slot = owner.texture_pix_map();
    let file = owner.texture_file().read().unwrap();
    format!("AFTER {mode} {} {} {} {} {} {}", i32::from(outer_file),
        i32::from(slot.is_null()), i32::from(slot.object_address() == next.object_address()),
        next.ref_count(), nested.ref_count(), file.to_string_lossy()).trim_end().to_string()
}
