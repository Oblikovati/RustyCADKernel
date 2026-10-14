#![allow(unsafe_code, dead_code)]
//! Test-owned Xlib boundary: resize and capture the actual mapped drawable, independently of Vulkan readback.
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::{aspect::*, xw::XwWindow};
use std::time::{Duration, Instant};
use x11_dl::xlib;
pub struct NativeWindow {
    api: xlib::Xlib,
    pub window: Handle<XwWindow>,
    display: Handle<AspectDisplayConnection>,
}
impl NativeWindow {
    /// Create a managed framebuffer target; e.g. NativeWindow::managed(1440,900).
    pub fn managed(width: i32, height: i32) -> Self {
        Self::create(width, height, false)
    }
    pub fn new(width: i32, height: i32) -> Self {
        Self::create(width, height, true)
    }
    fn create(width: i32, height: i32, unmanaged: bool) -> Self {
        let api = xlib::Xlib::open().expect("load test Xlib");
        unsafe {
            (api.XInitThreads)();
        }
        let display = Handle::new(AspectDisplayConnection::new().unwrap());
        let window = Self::create_client(&display, width, height);
        if unmanaged {
            Self::override_redirect(&api, &display, &window);
        }
        window.map();
        let native = Self {
            api,
            window,
            display,
        };
        native.sync();
        native
    }
    fn create_client(
        display: &Handle<AspectDisplayConnection>,
        width: i32,
        height: i32,
    ) -> Handle<XwWindow> {
        Handle::new(
            XwWindow::create(
                display,
                "Martensite Vulkan presentation",
                100,
                100,
                width,
                height,
            )
            .unwrap(),
        )
    }
    fn override_redirect(
        api: &xlib::Xlib,
        display: &Handle<AspectDisplayConnection>,
        window: &Handle<XwWindow>,
    ) {
        // Test geometry must not race the desktop window manager's placement/minimum size.
        let mut attributes = xlib::XSetWindowAttributes {
            override_redirect: 1,
            ..unsafe { std::mem::zeroed() }
        };
        unsafe {
            (api.XChangeWindowAttributes)(
                display.get_display_aspect() as *mut xlib::Display,
                window.native_handle(),
                xlib::CWOverrideRedirect,
                &mut attributes,
            );
        }
    }
    fn display_ptr(&self) -> *mut xlib::Display {
        self.display.get_display_aspect() as *mut xlib::Display
    }
    pub fn move_to(&self, x: i32, y: i32) {
        unsafe {
            (self.api.XMoveWindow)(self.display_ptr(), self.window.native_handle(), x, y);
        }
        self.sync();
    }
    pub fn sync(&self) {
        unsafe {
            (self.api.XSync)(self.display_ptr(), 0);
        }
    }
    pub fn unmap(&self) {
        // Override-redirect fixtures bypass the window manager's iconify protocol.
        unsafe {
            (self.api.XUnmapWindow)(self.display_ptr(), self.window.native_handle());
        }
        self.sync();
        assert!(!self.window.is_mapped(), "expected unmapped test drawable");
    }
    pub fn resize(&self, width: i32, height: i32) {
        assert!(
            width > 0 && height > 0,
            "received {width}x{height}; expected positive native extent"
        );
        unsafe {
            (self.api.XResizeWindow)(
                self.display_ptr(),
                self.window.native_handle(),
                width as u32,
                height as u32,
            );
        }
        self.sync();
        self.window.do_resize();
    }
    pub fn snapshot(&self) -> Vec<u8> {
        self.sync();
        let (width, height) = self.window.size();
        let image = unsafe {
            (self.api.XGetImage)(
                self.display_ptr(),
                self.window.native_handle(),
                0,
                0,
                width as u32,
                height as u32,
                !0,
                xlib::ZPixmap,
            )
        };
        assert!(
            !image.is_null(),
            "XGetImage returned null for {}",
            self.window.native_handle()
        );
        let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
        for y in 0..height {
            for x in 0..width {
                let pixel = unsafe { (self.api.XGetPixel)(image, x, y) };
                let masks = unsafe { [(*image).red_mask, (*image).green_mask, (*image).blue_mask] };
                rgba.extend(masks.map(|mask| channel(pixel, mask)));
                rgba.push(255);
            }
        }
        unsafe {
            (self.api.XDestroyImage)(image);
        }
        rgba
    }
    pub fn wait_for_pixels(&self, expected: &[u8]) -> Vec<u8> {
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            let actual = self.snapshot();
            if actual == expected {
                return actual;
            }
            assert!(
                Instant::now() < deadline,
                "native drawable differs in {} bytes; expected rendered RGBA pixels",
                actual.iter().zip(expected).filter(|(a, b)| a != b).count()
            );
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}
fn channel(pixel: u64, mask: u64) -> u8 {
    if mask == 0 {
        return 0;
    }
    let shift = mask.trailing_zeros();
    let maximum = mask >> shift;
    (((pixel & mask) >> shift) * 255 / maximum) as u8
}
#[cfg(test)]
mod tests {
    #[test]
    fn screen_masks_decode_channels_independently() {
        assert_eq!(super::channel(0xff8800, 0xff0000), 255);
        assert_eq!(super::channel(0xff8800, 0xff00), 136);
        assert_eq!(super::channel(0, 0), 0);
    }
}
