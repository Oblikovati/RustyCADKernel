//! Wasm_Window (built without Emscripten) and the display-free parts of Xw_Window, the twin of
//! tools/oracle/tkservice_probe_window.cxx.
use super::common::{Reader, end, flag, int, real, text};
use super::tried;
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkservice::aspect::{AspectWindow, AspectWindowMembers};
use rustycad_tkservice::wasm::WasmWindow;
use rustycad_tkservice::xw::XwWindow;

fn geometry(window: &WasmWindow, is_virtual: bool, size: [i32; 2]) {
    real(window.ratio());
    window.set_size_backing(Vec2::new(size[0], size[1]));
    real(window.ratio());
    let (w, h) = window.size();
    int(if is_virtual { w } else { 0 });
    int(if is_virtual { h } else { 0 });
    let (x1, y1, x2, y2) = window.position();
    int(x1);
    int(y1);
    int(if is_virtual { x2 } else { 0 });
    int(if is_virtual { y2 } else { 0 });
}

/// `sv_window`: a canvas window's geometry and conversions, then the Xw key translations
/// and constructor checks.
pub fn window_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (is_virtual, ratio) = (r.flag(), r.real());
    let size = [r.integer(), r.integer()];
    let (log_w, log_h) = (r.real(), r.real());
    let (buttons, key) = (r.integer(), r.integer());
    let window = WasmWindow::new(&AsciiString::from("#canvas"), true);
    window.set_virtual(is_virtual);
    text(window.canvas_id().as_bytes());
    flag(window.is_mapped());
    window.unmap();
    flag(window.is_mapped());
    window.map();
    flag(window.is_mapped());
    flag(window.do_mapping());
    int(window.do_resize() as i32);
    geometry(&window, is_virtual, size);
    window.set_device_pixel_ratio(ratio);
    real(window.device_pixel_ratio());
    window.set_size_logical(Vec2::new(log_w, log_h));
    real(window.ratio());
    int(window.native_handle() as i32);
    int(window.native_parent_handle() as i32);
    flag(window.native_fb_config() == 0);
    int(WasmWindow::mouse_buttons_from_native(buttons as u16) as i32);
    int(WasmWindow::virtual_key_from_native(key) as i32);
    for _ in 0..r.integer() {
        int(XwWindow::virtual_key_from_native(r.real() as u64) as i32);
    }
    tried(XwWindow::create(&Handle::default(), "t", 0, 0, size[0], size[1]).map(|_| ()));
    tried(XwWindow::wrap(&Handle::default(), key as u64, 0).map(|_| ()));
    end();
}
