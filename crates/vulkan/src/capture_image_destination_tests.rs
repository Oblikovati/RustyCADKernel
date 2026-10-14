//! Completed capture writes use hand-authored pixel expectations and real derived image hooks.
use crate::capture_image_destination_fixture::{
    DestinationMode, capture_destination_probe, counting_capture_destination,
};
use crate::capture_owned_fixture::{
    OWNED_COLOR_CASES, OwnedCaptureCase, owned_depth_rows, owned_frame_capture,
};
use crate::{
    CaptureImageError, CaptureImageFailure, CaptureImageLayout, CaptureImageOperation, FrameCapture,
};
use rustycad_tkservice::{graphic3d::Graphic3dBufferType, image::ImagePixMap};
use std::sync::atomic::Ordering;

fn assert_capture_storage(image: &ImagePixMap, first: &[u8], second: &[u8], top_down: bool) {
    let first = [first, &[167; 3]].concat();
    let second = [second, &[167; 3]].concat();
    let expected = if top_down {
        [first, second].concat()
    } else {
        [second, first].concat()
    };
    assert_eq!(image.data().data.data(), expected);
}
fn write_owned_color_destination(case: &OwnedCaptureCase, top_down: bool) {
    let destination = capture_destination_probe(DestinationMode::Valid);
    destination.base.set_top_down(top_down);
    owned_frame_capture()
        .write_image(&destination, case.kind, case.first.len() + 3)
        .unwrap();
    assert_capture_storage(&destination.base, case.first, case.second, top_down);
    assert_eq!(destination.base.format(), case.format);
    assert_eq!(destination.allocations.load(Ordering::Relaxed), 1);
    assert_eq!(destination.clears.load(Ordering::Relaxed), 1);
}

#[test]
fn derived_color_destinations_preserve_padding_and_both_row_orders() {
    for case in &OWNED_COLOR_CASES {
        for top_down in [false, true] {
            write_owned_color_destination(case, top_down);
        }
    }
}

#[test]
fn derived_depth_destination_uses_owned_ieee_rows_in_both_orders() {
    let rows = owned_depth_rows();
    for top_down in [false, true] {
        let destination = capture_destination_probe(DestinationMode::Valid);
        destination.base.set_top_down(top_down);
        owned_frame_capture()
            .write_image(&destination, Graphic3dBufferType::Depth, 11)
            .unwrap();
        assert_capture_storage(&destination.base, &rows[0], &rows[1], top_down);
    }
}

#[test]
fn concrete_destination_write_keeps_completed_capture_unchanged() {
    let capture = owned_frame_capture();
    let original = capture.clone();
    let image = ImagePixMap::default();
    capture
        .write_image(&image, Graphic3dBufferType::Rgba, 0)
        .unwrap();
    assert_eq!(image.data().data.row(0).unwrap(), &[1, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!(
        image.data().data.row(1).unwrap(),
        &[9, 10, 11, 12, 13, 14, 15, 16]
    );
    assert_eq!(
        (capture.width, capture.height),
        (original.width, original.height)
    );
    assert_eq!(capture.rgba, original.rgba);
    assert_eq!(capture.depth, original.depth);
}

fn require_capture_send_sync<T: Send + Sync>() {}

#[test]
fn capture_writer_context_types_keep_send_sync_bounds() {
    require_capture_send_sync::<FrameCapture>();
    require_capture_send_sync::<CaptureImageError>();
    require_capture_send_sync::<CaptureImageFailure>();
    require_capture_send_sync::<CaptureImageLayout>();
    require_capture_send_sync::<CaptureImageOperation>();
}

#[test]
fn valid_counting_destination_observes_projection_allocation_and_type_callbacks() {
    use rustycad_tkernel::standard::Transient;
    let destination = counting_capture_destination();
    owned_frame_capture()
        .write_image(&destination, Graphic3dBufferType::Rgba, 0)
        .unwrap();
    assert_eq!(destination.callbacks.load(Ordering::Relaxed), 3);
    let _ = destination.transient_base();
    assert_eq!(destination.callbacks.load(Ordering::Relaxed), 4);
    assert_eq!(
        destination.inner.base.data().data.row(0).unwrap(),
        &[1, 2, 3, 4, 5, 6, 7, 8]
    );
}
