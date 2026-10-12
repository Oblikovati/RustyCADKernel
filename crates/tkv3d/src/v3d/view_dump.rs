//! The image dump members of `V3d_View` (`Dump`, `ToPixMap`) and `DumpJson`.
//!
//! Native quirk adapted: a tiled dump wraps a sub-rectangle of the target image natively
//! (`Image_PixMap::InitWrapper` over its bytes); the port dumps each tile into its own image
//! and copies its rows into the target rectangle, which gives the same bytes.
use super::{V3dImageDumpOptions, V3dStereoDumpOptions, V3dView};
use rustycad_tkernel::message::{Gravity, Message};
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkservice::aspect::AspectWindowMembers as _;
use rustycad_tkservice::graphic3d::{
    GRAPHIC3D_ZLAYER_ID_BOT_OSD, Graphic3dBufferType, Graphic3dCView, Graphic3dCViewMembers as _,
    Graphic3dCamera, Graphic3dCameraProjection, Graphic3dCameraTile, Graphic3dTypeOfLimit,
};
use rustycad_tkservice::image::{ImageAlienPixMap, ImageFormat, ImagePixMap};
use rustycad_tkservice::json_dump::{
    class_begin, field_dumped, field_integer, field_pointer, field_real,
};
use std::io;

/// The image format of a buffer type (`ToPixMap`).
fn buffer_format(buffer_type: Graphic3dBufferType) -> ImageFormat {
    match buffer_type {
        Graphic3dBufferType::Rgb => ImageFormat::Rgb,
        Graphic3dBufferType::Rgba => ImageFormat::Rgba,
        Graphic3dBufferType::Depth | Graphic3dBufferType::ShadowMap => ImageFormat::GrayF,
        Graphic3dBufferType::RgbRayTraceHdrLeft => ImageFormat::Rgbf,
        Graphic3dBufferType::Red => ImageFormat::Gray,
    }
}

/// Copies the rows of a tile image into a target image at a byte row and column offset.
fn copy_tile(target: &ImagePixMap, tile: &ImagePixMap, bottom: usize, left: usize) {
    let source = tile.data().data.data().to_vec();
    let (rows, row_bytes) = {
        let state = tile.data();
        (state.data.size_y, state.data.size_x * state.data.size_bpp)
    };
    let tile_stride = tile.data().data.size_row_bytes;
    let mut state = target.change_data();
    let (stride, bpp) = (state.data.size_row_bytes, state.data.size_bpp);
    let bytes = state.data.change_data();
    for row in 0..rows {
        let start = (bottom + row) * stride + left * bpp;
        let Some(dst) = bytes.get_mut(start..start + row_bytes) else {
            continue;
        };
        if let Some(src) = source.get(row * tile_stride..row * tile_stride + row_bytes) {
            dst.copy_from_slice(src);
        }
    }
}

/// A dump buffer, its viewport, whether the dump is tiled and the previous buffer viewport.
type DumpBuffer = (Handle<StandardTransient>, Vec2<i32>, bool, Vec2<i32>);

impl V3dView {
    /// `Dump`: dumps the window size into an image file; `view.dump("view.ppm", &Graphic3dBufferType::Rgb)?`.
    pub fn dump(&self, file: &str, buffer_type: &Graphic3dBufferType) -> Result<bool, Failure> {
        let (width, height) = self.window().get().map_or((0, 0), |w| w.size());
        let image = ImageAlienPixMap::default();
        let options = V3dImageDumpOptions {
            width,
            height,
            buffer_type: *buffer_type,
            ..Default::default()
        };
        if !self.to_pix_map(&image, &options)? {
            return Ok(false);
        }
        image.save(&AsciiString::from(file))
    }
    /// `ToPixMap(image, width, height, ...)`: the dump with explicit options.
    /// Example: `view.to_pix_map_sized(&image, 64, 32, Graphic3dBufferType::Rgb, true)?`.
    pub fn to_pix_map_sized(
        &self,
        image: &ImagePixMap,
        width: i32,
        height: i32,
        buffer_type: Graphic3dBufferType,
        to_adjust_aspect: bool,
    ) -> Result<bool, Failure> {
        let options = V3dImageDumpOptions {
            width,
            height,
            buffer_type,
            to_adjust_aspect,
            ..Default::default()
        };
        self.to_pix_map(image, &options)
    }
    /// `ToPixMap(image, params)`: renders the view into an image, through an offscreen buffer
    /// (tiled when larger than the driver limits). Example: `view.to_pix_map(&image, &options)?`.
    pub fn to_pix_map(
        &self,
        image: &ImagePixMap,
        params: &V3dImageDumpOptions,
    ) -> Result<bool, Failure> {
        if !self.prepare_dump_image(image, params) {
            return Ok(false);
        }
        let target = Vec2::new(image.width() as i32, image.height() as i32);
        let cview = self.view();
        let previous_fbo = cview.fbo();
        let Some((fbo, viewport, is_tiling, previous_viewport)) =
            self.choose_dump_buffer(&cview, &previous_fbo, target, params.tile_size)
        else {
            return Ok(false);
        };
        let store_mapping = Graphic3dCamera::default();
        let camera = self.camera();
        store_mapping.copy(&camera);
        apply_stereo_options(&camera, params.stereo_options);
        if params.to_adjust_aspect {
            camera.set_aspect(f64::from(target.x) / f64::from(target.y));
        }
        cview.set_z_layer_target(params.target_z_layer_id);
        cview.set_z_layer_redraw_mode(params.is_single_layer);
        let previous_immediate = cview.set_immediate_mode_draw_to_front(false);
        let success = if is_tiling {
            self.dump_tiles(image, params, &fbo, viewport, target)?
        } else {
            self.dump_single(image, params, &fbo, target)?
        };
        cview.set_immediate_mode_draw_to_front(previous_immediate);
        camera.copy(&store_mapping);
        let mut fbo = fbo;
        if fbo != previous_fbo {
            cview.fbo_release(&mut fbo);
        } else if !previous_fbo.is_null() {
            cview.fbo_change_viewport(&previous_fbo, previous_viewport.x, previous_viewport.y);
        }
        cview.set_fbo(&previous_fbo);
        cview.set_z_layer_target(GRAPHIC3D_ZLAYER_ID_BOT_OSD);
        cview.set_z_layer_redraw_mode(false);
        Ok(success)
    }
    /// Allocates the image of the requested size; false (with a message) when impossible.
    fn prepare_dump_image(&self, image: &ImagePixMap, params: &V3dImageDumpOptions) -> bool {
        if params.width != 0 && params.height != 0 {
            let (w, h) = (params.width as usize, params.height as usize);
            let resize = image.is_empty() || image.width() != w || image.height() != h;
            if resize && !image.init_zero(buffer_format(params.buffer_type), w, h, 0, 0) {
                let text = format!(
                    "Fail to allocate an image {}x{} for view dump",
                    params.width, params.height
                );
                Message::send_text(&AsciiString::from(text.as_str()), Gravity::Fail);
                return false;
            }
        }
        if image.is_empty() {
            Message::send_text(
                &AsciiString::from("V3d_View::ToPixMap() has been called without image dimensions"),
                Gravity::Fail,
            );
            return false;
        }
        true
    }
    /// The offscreen buffer of a dump, its viewport, whether the dump is tiled and the
    /// previous buffer viewport; `None` when the tile size exceeds the hardware limits.
    fn choose_dump_buffer(
        &self,
        cview: &Handle<dyn Graphic3dCView>,
        previous: &Handle<StandardTransient>,
        target: Vec2<i32>,
        tile_size: i32,
    ) -> Option<DumpBuffer> {
        let mut viewport = target;
        let mut is_tiling = false;
        if tile_size > 0 && (viewport.x > tile_size || viewport.y > tile_size) {
            viewport = Vec2::new(viewport.x.min(tile_size), viewport.y.min(tile_size));
            is_tiling = true;
        }
        let mut fbo = Handle::default();
        let mut previous_viewport = Vec2::new(0, 0);
        if !previous.is_null() {
            let [w, h, max_w, max_h] = cview.fbo_get_dimensions(previous);
            previous_viewport = Vec2::new(w, h);
            if viewport.x <= max_w && viewport.y <= max_h {
                fbo = previous.clone();
            }
        }
        if fbo.is_null() {
            let driver = self.viewer().get().map(|v| v.driver()).unwrap_or_default();
            let limit = |kind| driver.get().map_or(0, |d| d.inquire_limit(kind));
            let (max_x, max_y) = (
                limit(Graphic3dTypeOfLimit::MaxViewDumpSizeX),
                limit(Graphic3dTypeOfLimit::MaxViewDumpSizeY),
            );
            if tile_size > max_x || tile_size > max_y {
                let text = format!(
                    "Image dump can not be performed - specified tile size ({tile_size}) exceeds hardware limits ({max_x}x{max_y})"
                );
                Message::send_text(&AsciiString::from(text.as_str()), Gravity::Fail);
                return None;
            }
            if viewport.x > max_x || viewport.y > max_y {
                if limit(Graphic3dTypeOfLimit::IsWorkaroundFBO) != 0 {
                    Message::send_text(
                        &AsciiString::from(
                            "Warning, workaround for Intel driver problem with empty FBO for images with big width is applied",
                        ),
                        Gravity::Warning,
                    );
                }
                let text = format!(
                    "Info, tiling image dump is used, image size ({}x{}) exceeds hardware limits ({max_x}x{max_y})",
                    viewport.x, viewport.y
                );
                Message::send_text(&AsciiString::from(text.as_str()), Gravity::Info);
                viewport = Vec2::new(viewport.x.min(max_x), viewport.y.min(max_y));
                is_tiling = true;
            }
            fbo = cview.fbo_create(viewport.x, viewport.y);
        }
        cview.set_fbo(&fbo);
        if fbo.is_null() {
            let (w, h) = self.window().get().map_or((0, 0), |w| w.size());
            if viewport.x != w || viewport.y != h {
                is_tiling = true;
            }
            viewport = Vec2::new(w, h);
            Message::send_text(
                &AsciiString::from(
                    "Warning, on screen buffer is used for image dump - content might be invalid",
                ),
                Gravity::Warning,
            );
        }
        Some((fbo, viewport, is_tiling, previous_viewport))
    }
    /// The untiled dump: one redraw and one buffer (or shadow map) dump.
    fn dump_single(
        &self,
        image: &ImagePixMap,
        params: &V3dImageDumpOptions,
        fbo: &Handle<StandardTransient>,
        target: Vec2<i32>,
    ) -> Result<bool, Failure> {
        let cview = self.view();
        if !fbo.is_null() {
            cview.fbo_change_viewport(fbo, target.x, target.y);
        }
        self.redraw();
        if params.buffer_type == Graphic3dBufferType::ShadowMap {
            return Self::dump_shadow_map(&*cview, image, params.light_name);
        }
        Ok(cview.buffer_dump(image, &params.buffer_type))
    }
    fn dump_shadow_map(
        cview: &dyn Graphic3dCView,
        image: &ImagePixMap,
        name: super::V3dLightName<'_>,
    ) -> Result<bool, Failure> {
        let consumed = name.to_ascii_string()?;
        if cview.shadow_map_dump(image, &consumed) {
            return Ok(true);
        }
        Message::send_text(
            &AsciiString::from("OpenGl_View::BufferDump() failed to dump shadowmap"),
            Gravity::Fail,
        );
        Ok(false)
    }
    /// The tiled dump: one redraw and one buffer dump per tile.
    fn dump_tiles(
        &self,
        image: &ImagePixMap,
        params: &V3dImageDumpOptions,
        fbo: &Handle<StandardTransient>,
        viewport: Vec2<i32>,
        target: Vec2<i32>,
    ) -> Result<bool, Failure> {
        let (cview, camera) = (self.view(), self.camera());
        let mut offset_y = 0;
        while offset_y < target.y {
            let mut offset_x = 0;
            while offset_x < target.x {
                let uncropped = Graphic3dCameraTile {
                    offset: Vec2::new(offset_x, offset_y),
                    total_size: target,
                    tile_size: viewport,
                    ..Default::default()
                };
                offset_x += viewport.x;
                let tile = uncropped.cropped();
                if tile.tile_size.x < 1 || tile.tile_size.y < 1 {
                    continue;
                }
                let left = tile.offset.x;
                let bottom = if image.is_top_down() {
                    target.y - (tile.offset.y + tile.tile_size.y)
                } else {
                    tile.offset.y
                };
                if fbo.is_null() {
                    camera.set_tile(&uncropped);
                } else {
                    camera.set_tile(&tile);
                    cview.fbo_change_viewport(fbo, tile.tile_size.x, tile.tile_size.y);
                }
                self.redraw();
                let tile_image = ImagePixMap::default();
                tile_image.set_top_down(image.is_top_down());
                if !tile_image.init_zero(
                    image.format(),
                    tile.tile_size.x as usize,
                    tile.tile_size.y as usize,
                    0,
                    0,
                ) || !cview.buffer_dump(&tile_image, &params.buffer_type)
                {
                    return Ok(false);
                }
                copy_tile(image, &tile_image, bottom as usize, left as usize);
            }
            offset_y += viewport.y;
        }
        Ok(true)
    }
}

/// The projection of a stereo camera for a dump.
fn apply_stereo_options(camera: &Graphic3dCamera, options: V3dStereoDumpOptions) {
    if !camera.is_stereo() {
        return;
    }
    match options {
        V3dStereoDumpOptions::Mono => {
            camera.set_projection_type(Graphic3dCameraProjection::Perspective)
        }
        V3dStereoDumpOptions::LeftEye => {
            camera.set_projection_type(Graphic3dCameraProjection::MonoLeftEye)
        }
        V3dStereoDumpOptions::RightEye => {
            camera.set_projection_type(Graphic3dCameraProjection::MonoRightEye)
        }
        V3dStereoDumpOptions::Blended => {}
    }
}

impl V3dView {
    /// `DumpJson`: the start camera, the default camera, the driver view, the lights, the
    /// window, the rotation and zoom state, the trihedron, the grid and the screen axes.
    /// Example: `view.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "V3d_View")?;
        let s = self.state.read().clone();
        field_real(stream, "myOldMouseX", s.old_mouse_x)?;
        field_real(stream, "myOldMouseY", s.old_mouse_y)?;
        field_dumped(stream, depth, "&myCamStartOpUp", &|o, _| {
            s.cam_start_op_up.dump_json(o)
        })?;
        field_dumped(stream, depth, "&myCamStartOpDir", &|o, _| {
            s.cam_start_op_dir.dump_json(o)
        })?;
        field_dumped(stream, depth, "&myCamStartOpEye", &|o, _| {
            s.cam_start_op_eye.dump_json(o)
        })?;
        field_dumped(stream, depth, "&myCamStartOpCenter", &|o, _| {
            s.cam_start_op_center.dump_json(o)
        })?;
        let default_camera = self.default_camera();
        field_dumped(stream, depth, "myDefaultCamera.get()", &|o, d| {
            default_camera.dump_json(o, d)
        })?;
        let cview = self.view();
        field_dumped(stream, depth, "myView.get()", &|o, d| cview.dump_json(o, d))?;
        field_integer(stream, "myImmediateUpdate", i32::from(s.immediate_update))?;
        field_integer(
            stream,
            "myIsInvalidatedImmediate",
            i32::from(s.is_invalidated_immediate),
        )?;
        field_pointer(stream, "MyViewer", s.viewer.object_address() as usize)?;
        for light in &s.active_lights {
            field_pointer(stream, "anActiveLight", light.object_address() as usize)?;
        }
        field_dumped(stream, depth, "&myDefaultViewAxis", &|o, _| {
            s.default_view_axis.dump_json(o)
        })?;
        field_dumped(stream, depth, "&myDefaultViewPoint", &|o, _| {
            s.default_view_point.dump_json(o)
        })?;
        if let Some(window) = s.window.get() {
            field_dumped(stream, depth, "MyWindow.get()", &|o, d| {
                window.dump_json(o, d)
            })?;
        }
        self.dump_interaction(stream, depth, &s)
    }
    fn dump_interaction(
        &self,
        stream: &mut DumpStream,
        depth: i32,
        s: &super::view::V3dViewState,
    ) -> io::Result<()> {
        field_integer(stream, "sx", s.sx)?;
        field_integer(stream, "sy", s.sy)?;
        field_real(stream, "rx", s.rx)?;
        field_real(stream, "ry", s.ry)?;
        field_dumped(stream, depth, "&myRotateGravity", &|o, _| {
            s.rotate_gravity.dump_json(o)
        })?;
        field_integer(stream, "myComputedMode", i32::from(s.computed_mode))?;
        field_integer(stream, "SwitchSetFront", i32::from(s.switch_set_front))?;
        field_integer(stream, "myZRotation", i32::from(s.z_rotation))?;
        field_integer(stream, "MyZoomAtPointX", s.zoom_at_point_x)?;
        field_integer(stream, "MyZoomAtPointY", s.zoom_at_point_y)?;
        if let Some(trihedron) = s.trihedron.get() {
            field_dumped(stream, depth, "myTrihedron.get()", &|o, d| {
                trihedron.dump_json(o, d)
            })?;
        }
        if let Some(grid) = s.grid.get() {
            field_dumped(stream, depth, "MyGrid.get()", &|o, d| grid.dump_json(o, d))?;
        }
        field_dumped(stream, depth, "&MyPlane", &|o, _| s.plane.dump_json(o))?;
        if let Some(structure) = s.grid_echo_structure.get() {
            field_dumped(stream, depth, "MyGridEchoStructure.get()", &|o, d| {
                structure.dump_json(o, d)
            })?;
        }
        if let Some(group) = s.grid_echo_group.get() {
            field_dumped(stream, depth, "MyGridEchoGroup.get()", &|o, d| {
                group.dump_json(o, d)
            })?;
        }
        field_dumped(stream, depth, "&myXscreenAxis", &|o, _| {
            s.x_screen_axis.dump_json(o)
        })?;
        field_dumped(stream, depth, "&myYscreenAxis", &|o, _| {
            s.y_screen_axis.dump_json(o)
        })?;
        field_dumped(stream, depth, "&myZscreenAxis", &|o, _| {
            s.z_screen_axis.dump_json(o)
        })?;
        field_dumped(stream, depth, "&myViewAxis", &|o, _| {
            s.view_axis.dump_json(o)
        })?;
        let vertex = s.gravity_reference_point;
        field_dumped(stream, depth, "&myGravityReferencePoint", &|o, d| {
            vertex.dump_json(o, d)
        })?;
        field_integer(stream, "myAutoZFitIsOn", i32::from(s.auto_z_fit_is_on))?;
        field_real(stream, "myAutoZFitScaleFactor", s.auto_z_fit_scale_factor)
    }
}
