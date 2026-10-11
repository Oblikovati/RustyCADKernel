//! Device-independent validation rejects malformed submissions before Vulkan I/O.
use martensite_vulkan::*;
#[derive(Default)]
struct RecordingBackend {
    frames: usize,
}
impl RenderBackend for RecordingBackend {
    fn render(&mut self, frame: &RenderFrame) -> Result<FrameCapture, RenderError> {
        assert_eq!(frame.extent(), [1, 1]);
        assert_eq!(frame.clear_color(), [0.; 4]);
        assert!(frame.draws().is_empty());
        self.frames += 1;
        Ok(FrameCapture {
            width: 1,
            height: 1,
            rgba: vec![1, 2, 3, 255],
            depth: vec![1.],
        })
    }
}
#[test]
fn malformed_geometry_reports_the_offending_input() {
    let vertices = vec![RenderVertex::new([0.; 3], [1.; 4]); 3];
    for indices in [vec![0, 1], vec![0, 1, 9]] {
        let error = TriangleDraw::new(vertices.clone(), indices, identity_clip()).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("expected triangle index triples")
        );
    }
    let mut invalid = vertices;
    invalid[0].position[1] = f32::NAN;
    assert!(TriangleDraw::new(invalid, vec![0, 1, 2], identity_clip()).is_err());
}
#[test]
fn invalid_extents_and_clear_colors_fail_before_backend_calls() {
    for (width, height) in [(0, 1), (1, 0), (u32::MAX, 2)] {
        assert!(RenderFrame::new(width, height, [0.; 4], vec![]).is_err());
    }
    assert!(RenderFrame::new(1, 1, [f32::INFINITY; 4], vec![]).is_err());
}
#[test]
fn named_recording_backend_uses_the_same_capture_contract() {
    let frame = RenderFrame::new(1, 1, [0.; 4], vec![]).unwrap();
    let mut backend = RecordingBackend::default();
    let capture = backend.render(&frame).unwrap();
    assert_eq!(backend.frames, 1);
    assert_eq!(capture.pixel(0, 0), Some([1, 2, 3, 255]));
    assert_eq!(capture.pixel(1, 0), None);
    assert_eq!(capture.pixel(0, 1), None);
}
#[test]
fn surface_alpha_is_validated_and_invalid_transforms_are_rejected() {
    let mut vertices = vec![RenderVertex::new([0.; 3], [1.; 4]); 3];
    vertices[0].color[3] = 0.5;
    let blended = TriangleDraw::new(vertices.clone(), vec![0, 1, 2], identity_clip()).unwrap();
    assert!(blended.is_surface_blended());
    vertices[0].color[3] = 1.1;
    assert!(TriangleDraw::new(vertices.clone(), vec![0, 1, 2], identity_clip()).is_err());
    vertices[0].color[3] = 1.;
    let mut matrix = identity_clip();
    matrix[3] = f32::INFINITY;
    assert!(TriangleDraw::new(vertices, vec![0, 1, 2], matrix).is_err());
}

#[test]
fn retained_transform_is_available_to_injected_backends() {
    let matrix = identity_clip();
    let draw = TriangleDraw::new(vec![], vec![], matrix).unwrap();
    assert_eq!(draw.clip_transform(), &matrix);
}

#[test]
fn injected_backends_can_report_input_errors() {
    let error = RenderError::rejected(0, "positive extent");
    assert_eq!(error.to_string(), "received 0; expected positive extent");
}

#[test]
fn ordered_layers_retain_draw_ranges_and_empty_frame_clear() {
    let draw = TriangleDraw::new(vec![], vec![], identity_clip()).unwrap();
    let frame = RenderFrame::with_layers(
        1,
        1,
        [0.; 4],
        vec![
            RenderLayer::new(vec![draw.clone()], true, false, true),
            RenderLayer::new(vec![draw.clone(), draw], false, true, false),
        ],
    )
    .unwrap();
    assert_eq!(frame.layers()[0].draw_range(), 0..1);
    assert_eq!(frame.layers()[1].draw_range(), 1..3);
    assert_eq!(frame.layers()[1].depth_policy(), (false, true, false));
    let empty = RenderFrame::with_layers(1, 1, [0.; 4], vec![]).unwrap();
    assert_eq!(empty.layers()[0].depth_policy(), (true, true, true));
    assert_eq!(empty.layers()[0].draw_range(), 0..0);
}
#[test]
fn polygon_offset_survives_layer_flattening_and_transparent_deferral() {
    let opaque = PrimitiveDraw::new(vec![], vec![], identity_clip()).unwrap();
    let transparent = PrimitiveDraw::new(
        vec![RenderVertex::new([0.; 3], [1., 1., 1., 0.5]); 3],
        vec![0, 1, 2],
        identity_clip(),
    )
    .unwrap();
    let offset = RenderPolygonOffset::new(1, 2., 3.).unwrap();
    let layer =
        RenderLayer::new(vec![transparent, opaque], true, true, false).with_polygon_offset(offset);
    let frame = RenderFrame::with_layers(1, 1, [0.; 4], vec![layer]).unwrap();
    assert_eq!(frame.layers().len(), 2);
    for layer in frame.layers() {
        assert_eq!(layer.polygon_offset(), offset);
    }
}
