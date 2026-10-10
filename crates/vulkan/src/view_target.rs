//! Logical framebuffer targets let V3d's dump lifecycle select offscreen dimensions.
use crate::{RenderError, error::rejected};
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient};
use rustycad_tkservice::graphic3d::Graphic3dCView;
pub(crate) struct CaptureTarget {
    transient: StandardTransient,
    extent: SharedState<[i32; 2]>,
    capacity: [i32; 2],
}
rustycad_tkernel::impl_transient!(
    CaptureTarget,
    "Martensite_CaptureTarget",
    StandardTransient,
    transient
);
impl CaptureTarget {
    pub fn create(width: i32, height: i32, limit: u32) -> Handle<StandardTransient> {
        if width <= 0 || height <= 0 || width as u32 > limit || height as u32 > limit {
            return Handle::default();
        }
        Handle::new(Self {
            transient: StandardTransient::default(),
            extent: SharedState::new([width, height]),
            capacity: [width, height],
        })
        .cast()
    }
    pub fn dimensions(&self) -> [i32; 4] {
        let [w, h] = *self.extent.read();
        [w, h, self.capacity[0], self.capacity[1]]
    }
    pub fn resize(&self, width: i32, height: i32) {
        if width > 0 && height > 0 && width <= self.capacity[0] && height <= self.capacity[1] {
            *self.extent.write() = [width, height];
        }
    }
}
pub(crate) fn capture_extent(
    view: &dyn Graphic3dCView,
    limit: u32,
) -> Result<[u32; 2], RenderError> {
    let dimensions = target_dimensions(view);
    let Some((width, height)) = dimensions else {
        return Err(rejected(
            "missing window or foreign framebuffer",
            "a Martensite capture target",
        ));
    };
    if width <= 0 || height <= 0 || width as u32 > limit || height as u32 > limit {
        return Err(rejected(
            (width, height),
            &format!("positive extent at most {limit}"),
        ));
    }
    Ok([width as u32, height as u32])
}

fn target_dimensions(view: &dyn Graphic3dCView) -> Option<(i32, i32)> {
    let fbo = view.fbo();
    if fbo.is_null() {
        return view.window().get().map(|window| window.size());
    }
    fbo.cast::<CaptureTarget>().get().map(|target| {
        let size = target.dimensions();
        (size[0], size[1])
    })
}
