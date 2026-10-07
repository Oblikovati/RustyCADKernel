//! The native window-system handles of the Aspect package (`Aspect_Drawable.hxx`,
//! `Aspect_Handle.hxx`, `Aspect_Display.hxx`, `Aspect_FBConfig.hxx`,
//! `Aspect_RenderingContext.hxx`).
//!
//! The pointers of the native typedefs (`Display*`, `GLXFBConfig`, `GLXContext`) are kept
//! as their addresses: the classes only store and compare them, and the platform wrapper
//! (Xlib, the Vulkan surface) turns them back into pointers.

/// A window or pixmap id: `Window` under X11, an `HDC` address on Windows (`Aspect_Drawable`).
#[cfg(not(windows))]
pub type AspectDrawable = std::ffi::c_ulong;
/// A window or pixmap id: `Window` under X11, an `HDC` address on Windows (`Aspect_Drawable`).
#[cfg(windows)]
pub type AspectDrawable = usize;

/// A native handle: an X11 id, a `HANDLE` address on Windows (`Aspect_Handle`).
#[cfg(not(windows))]
pub type AspectHandle = std::ffi::c_ulong;
/// A native handle: an X11 id, a `HANDLE` address on Windows (`Aspect_Handle`).
#[cfg(windows)]
pub type AspectHandle = usize;

/// The address of the native display (`Display*` under X11) (`Aspect_Display`).
pub type AspectDisplay = usize;

/// The address of a GLX frame-buffer configuration, 0 for none (`Aspect_FBConfig`).
pub type AspectFbConfig = usize;

/// The address of a native rendering context, 0 for none (`Aspect_RenderingContext`).
pub type AspectRenderingContext = usize;
