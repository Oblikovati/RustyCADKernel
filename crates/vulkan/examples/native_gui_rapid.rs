//! Actual native framebuffer→image capture after rapid GUI replacement, independent of X11 readback.
#[path = "support/device_options.rs"]
mod device_options;
#[path = "support/framebuffer_capture_image.rs"]
mod framebuffer_capture_image;
#[path = "../tests/support/gui_packets.rs"]
mod gui_packets;
#[path = "support/gui_timing_samples.rs"]
mod gui_timing_samples;
#[path = "support/native_window.rs"]
mod native_window;
#[path = "support/rapid_gui_framebuffer.rs"]
mod rapid_gui_framebuffer;
use martensite_vulkan::*;
use rustycad_tkernel::quantity::{Color, ColorSpace};
use rustycad_tkv3d::v3d::V3dViewer;
use std::path::PathBuf;
const EXTENT: [u32; 2] = [1440, 900];
fn main() {
    let output = capture_directory();
    let measure = performance_requested(&std::env::args().skip(2).collect::<Vec<_>>());
    let driver = capture_driver(measure);
    let native = native_window::NativeWindow::managed(1440, 900);
    let viewer = V3dViewer::new(&driver.cast()).unwrap();
    let view = viewer.create_view().unwrap();
    view.set_immediate_update(false);
    view.set_window(&native.window.cast(), 0).unwrap();
    let render = view.get().unwrap().view().cast::<Graphic3dRenderView>();
    verify_background(view.get().unwrap(), &render, &output);
    rapid_gui(&render, &output);
    restore_gui_after_clear(&render, &output);
    measure_steady_redraw(&render, measure);
    view.remove();
    assert!(render.capture().is_err(), "removed view retained a capture");
    check_validation(&driver);
}
fn capture_driver(measure: bool) -> rustycad_tkernel::standard::Handle<Graphic3dRenderDriver> {
    let mut options = device_options::validation_options();
    options.validation = !measure;
    Graphic3dRenderDriver::new(options).unwrap()
}
fn performance_requested(arguments: &[String]) -> bool {
    match arguments {
        [] => false,
        [flag] if flag == "--performance" => true,
        _ => panic!("arguments={arguments:?}; expected no timing arguments or --performance"),
    }
}
fn measure_steady_redraw(render: &Graphic3dRenderView, measure: bool) {
    if !measure {
        return;
    }
    let samples = gui_timing_samples::collect_samples(20, 100, |_| {
        let start = std::time::Instant::now();
        render.try_redraw().unwrap();
        start.elapsed().as_secs_f64() * 1000.
    });
    let capture = render.capture().unwrap();
    rapid_gui_framebuffer::verify_capture(&capture, [0, 255, 0, 255], Some([0, 0, 255, 255]));
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"rapid3gui-1440x900-steady-redraw\",\"extent\":[1440,900],\"warmup\":20,\"validation\":false,\"baseline_accepted\":false,\"samples_ms\":{samples:?}}}"
    );
}

fn restore_gui_after_clear(render: &Graphic3dRenderView, output: &std::path::Path) {
    replace_gui_packets(render);
    render.try_redraw().unwrap();
    rapid_gui_framebuffer::verify_capture(
        &render.capture().unwrap(),
        [0, 255, 0, 255],
        Some([0, 0, 255, 255]),
    );
    rapid_gui_framebuffer::save_capture(output, "restored", &render.capture().unwrap());
}

fn capture_directory() -> PathBuf {
    let output = PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("expected owned capture directory argument"),
    );
    std::fs::create_dir_all(&output).unwrap();
    output
}
fn check_validation(driver: &Graphic3dRenderDriver) {
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    println!("MARTENSITE_DEVICE_IDENTITY={:?}", driver.driver_identity());
}

fn verify_background(
    view: &dyn rustycad_tkv3d::v3d::V3dViewSource,
    render: &Graphic3dRenderView,
    output: &std::path::Path,
) {
    for (name, color, bytes) in [
        ("blue", [0., 0., 1.], [0, 0, 255, 255]),
        ("green", [0., 1., 0.], [0, 255, 0, 255]),
    ] {
        view.set_background_color(
            Color::new(color[0], color[1], color[2], ColorSpace::Rgb).unwrap(),
        );
        render.try_redraw().unwrap();
        let capture = render.capture().unwrap();
        rapid_gui_framebuffer::verify_capture(&capture, bytes, None);
        rapid_gui_framebuffer::save_capture(output, name, &capture);
    }
}
fn rapid_gui(render: &Graphic3dRenderView, output: &std::path::Path) {
    replace_gui_packets(render);
    // Preserve the consumer's three draws plus verification draw (rapid.rs ae2c8ea8).
    // Every completed frame must be correct; later draws cannot conceal a stale first frame.
    for index in 0..4 {
        render.try_redraw().unwrap();
        let capture = render.capture().unwrap();
        rapid_gui_framebuffer::verify_capture(&capture, [0, 255, 0, 255], Some([0, 0, 255, 255]));
        rapid_gui_framebuffer::save_capture(output, &format!("rapid3gui-{index}"), &capture);
    }
    render.clear_gui_layer().unwrap();
    assert!(
        render.capture().is_err(),
        "clear retained the preceding GUI capture"
    );
    render.try_redraw().unwrap();
    let capture = render.capture().unwrap();
    rapid_gui_framebuffer::verify_capture(&capture, [0, 255, 0, 255], None);
    rapid_gui_framebuffer::save_capture(output, "cleared", &capture);
}
fn replace_gui_packets(render: &Graphic3dRenderView) {
    // Preserve rapid.rs ae2c8ea8: all three packet replacements precede drawing.
    for color in [[1., 0., 0., 0.5], [0., 1., 0., 0.5], [0., 0., 1., 1.]] {
        render
            .attach_gui_layer(
                EXTENT,
                RenderLayer::new(vec![rectangle(color)], false, false, false),
            )
            .unwrap();
        assert!(
            render.capture().is_err(),
            "replacement must invalidate previous capture"
        );
    }
}

fn rectangle(color: [f32; 4]) -> PrimitiveDraw {
    let vertices = gui_rectangle_vertices(color);
    let image = RasterImage::new([1, 1], vec![[1.; 4]]).unwrap();
    let style =
        TexturedStyle::new(image, vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.]], 0.).unwrap();
    PrimitiveDraw::textured(
        vertices,
        vec![0, 1, 2, 0, 2, 3],
        gui_packets::pixel_clip(EXTENT),
        style,
    )
    .unwrap()
}

fn gui_rectangle_vertices(color: [f32; 4]) -> Vec<RenderVertex> {
    [
        [8., 8., 0.],
        [308., 8., 0.],
        [308., 118., 0.],
        [8., 118., 0.],
    ]
    .map(|point| RenderVertex::new(point, color))
    .to_vec()
}

#[cfg(test)]
mod tests {
    #[test]
    fn ordinary_acceptance_never_requests_timing() {
        assert!(!super::performance_requested(&[]));
        assert!(super::performance_requested(&["--performance".into()]));
        assert!(
            std::panic::catch_unwind(|| super::performance_requested(&["--unknown".into()]))
                .is_err()
        );
    }
}
