//! Actual public ToPixMap boundary uses the existing named recording backend; no GPU is mocked inline.
#[path = "support/recording_device.rs"]
mod recording_device;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::{aspect::AspectNeutralWindow, graphic3d::*, image::ImagePixMap};
use rustycad_tkv3d::v3d::*;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

struct LightDumpView {
    base: V3dView,
    redraws: Arc<AtomicUsize>,
}
rustycad_tkernel::impl_transient!(LightDumpView,"Martensite_LightDumpView",V3dView,base,
    interfaces:[dyn V3dViewSource]);
impl V3dViewSource for LightDumpView {
    fn view_base(&self) -> &V3dView {
        // Copying the view would change callback and implementation ownership.
        &self.base
    }
    fn redraw(&self) {
        self.redraws.fetch_add(1, Ordering::Relaxed);
        v3d_view_redraw(&self.base);
    }
}

fn windowed_dump_view(
    viewer: &Handle<V3dViewer>,
    redraws: &Arc<AtomicUsize>,
) -> Handle<dyn V3dViewSource> {
    let view = Handle::new(LightDumpView {
        base: V3dView::new_embedded(viewer, V3dTypeOfView::Orthographic).unwrap(),
        redraws: redraws.clone(),
    })
    .cast::<dyn V3dViewSource>();
    V3dView::bind_source(&view).unwrap();
    let window = Handle::new(AspectNeutralWindow::default());
    window.set_size(8, 8);
    view.set_window(&window.cast(), 0).unwrap();
    redraws.store(0, Ordering::Relaxed);
    view
}

fn capture_boundary(label: &str, buffer: Graphic3dBufferType, name: V3dLightName<'_>) -> String {
    let device = recording_device::RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend(device.clone(), 128).unwrap();
    let viewer = V3dViewer::new(&driver.cast()).unwrap();
    let redraws = Arc::new(AtomicUsize::new(0));
    let view = windowed_dump_view(&viewer, &redraws);
    let options = boundary_options(buffer, name);
    let result = view.to_pix_map(&ImagePixMap::default(), &options);
    assert_eq!(device.last_frame().extent(), [8, 8]);
    boundary_result(label, &result, &*view, &redraws)
}

fn boundary_options(
    buffer: Graphic3dBufferType,
    name: V3dLightName<'_>,
) -> V3dImageDumpOptions<'_> {
    V3dImageDumpOptions {
        width: 8,
        height: 8,
        buffer_type: buffer,
        light_name: name,
        target_z_layer_id: 42,
        is_single_layer: true,
        ..Default::default()
    }
}

fn boundary_result(
    label: &str,
    result: &Result<bool, rustycad_tkernel::standard::Failure>,
    view: &dyn V3dViewSource,
    redraws: &AtomicUsize,
) -> String {
    let outcome = result
        .as_ref()
        .err()
        .map_or("ok", |failure| failure.exception_type());
    let implementation = view.view_ref().observe();
    format!(
        "{label} {outcome} {} {} {}\n",
        redraws.load(Ordering::Relaxed),
        implementation.z_layer_target(),
        i32::from(implementation.z_layer_redraw_mode())
    )
}

#[test]
fn public_view_dump_null_and_bytes_native_boundary_replay() {
    let null = V3dLightName::null();
    let mut rows = capture_boundary("rgb-null", Graphic3dBufferType::Rgb, null);
    rows.push_str(&capture_boundary(
        "shadow-null",
        Graphic3dBufferType::ShadowMap,
        null,
    ));
    let bytes = c"\xff\x80";
    rows.push_str(&capture_boundary(
        "shadow-bytes",
        Graphic3dBufferType::ShadowMap,
        V3dLightName::from_c_str(bytes),
    ));
    assert_eq!(
        rows,
        include_str!("../../tkv3d/tests/fixtures/light-name-dump-boundary-native.txt")
    );
}
