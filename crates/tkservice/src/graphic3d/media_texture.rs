//! `Graphic3d_MediaTexture` and `Graphic3d_MediaTextureSet`: the textures of a video player,
//! as the build without FFmpeg compiles them (frames stay empty, so no image is produced).
use super::media_texture_shaders::yuv_shader_program;
use super::texture_2d::Graphic3dTexture2DBase;
use super::{
    Graphic3dShaderProgram, Graphic3dTexture2DSource, Graphic3dTextureMap, Graphic3dTextureRoot,
    Graphic3dTextureRootBase, Graphic3dTextureRootMembers as _, Graphic3dTextureSet,
    Graphic3dTextureUnit, Graphic3dTypeOfTexture, Graphic3dTypeOfTextureFilter,
};
use crate::image::{ImagePixMap, ImageSupportedFormats};
use crate::media::{MediaFrame, MediaIFrameQueue, MediaPlayerContext};
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// `CallbackOnUpdate_t` with its user pointer: called when a new frame is ready.
pub type Graphic3dMediaTextureSetCallback = Arc<dyn Fn() + Send + Sync>;

/// The front/back state behind `myMutex`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Graphic3dMediaFrameSwap {
    /// `myFront`: the index of the front frame.
    pub front: usize,
    /// `myToPresentFrame`: a filled back frame waits to be presented.
    pub to_present_frame: bool,
}

/// The mutex the textures and the set share (`std::mutex& myMutex`).
pub type Graphic3dMediaFrameLock = Arc<Mutex<Graphic3dMediaFrameSwap>>;

fn lock(mutex: &Graphic3dMediaFrameLock) -> MutexGuard<'_, Graphic3dMediaFrameSwap> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// One plane of a video frame as a 2D texture; made by [`Graphic3dMediaTextureSet`].
pub struct Graphic3dMediaTexture {
    transient: StandardTransient,
    base: Graphic3dTexture2DBase,
    /// `myMutex`.
    mutex: Graphic3dMediaFrameLock,
    /// `myFrame`.
    frame: SharedState<Handle<MediaFrame>>,
    /// `myPlane`.
    plane: i32,
    /// `myPixMapWrapper`.
    pix_map_wrapper: SharedState<Handle<ImagePixMap>>,
}
impl Graphic3dTextureRoot for Graphic3dMediaTexture {
    fn texture_root_base(&self) -> &Graphic3dTextureRootBase {
        &self.base.root
    }
    /// `GetImage`: no image without FFmpeg (the frame is null, locked or empty).
    fn get_image(&self, supported: &Handle<ImageSupportedFormats>) -> Handle<ImagePixMap> {
        let _ = supported;
        let _guard = lock(&self.mutex);
        let frame = self.frame.read().clone();
        let Some(frame) = frame
            .get()
            .filter(|f| !f.is_locked() && !f.is_empty() && f.size_x() >= 1 && f.size_y() >= 1)
        else {
            return Handle::default();
        };
        let _ = frame;
        let mut wrapper = self.pix_map_wrapper.write();
        if wrapper.is_null() {
            *wrapper = Handle::new(ImagePixMap::default());
        }
        Handle::default()
    }
}
impl Graphic3dTextureMap for Graphic3dMediaTexture {}
impl Graphic3dTexture2DSource for Graphic3dMediaTexture {
    fn texture_2d_base(&self) -> &Graphic3dTexture2DBase {
        &self.base
    }
}
rustycad_tkernel::impl_transient!(Graphic3dMediaTexture, "Graphic3d_MediaTexture", dyn Graphic3dTexture2DSource, transient,
    interfaces: [dyn Graphic3dTextureRoot, dyn Graphic3dTextureMap, dyn Graphic3dTexture2DSource]);

/// The texture unit of a plane 0..3 (`Graphic3d_TextureUnit_0 + thePlane`).
fn plane_unit(plane: usize) -> Graphic3dTextureUnit {
    [
        Graphic3dTextureUnit::Unit0,
        Graphic3dTextureUnit::Unit1,
        Graphic3dTextureUnit::Unit2,
        Graphic3dTextureUnit::Unit3,
    ][plane.min(3)]
}

impl Graphic3dMediaTexture {
    /// The private constructor: a texture of `plane` (0..3) without modulation and repeat,
    /// bilinear, on the plane's texture unit.
    pub(crate) fn new(mutex: &Graphic3dMediaFrameLock, plane: usize) -> Self {
        let base = Graphic3dTexture2DBase::from_file(
            &AsciiString::default(),
            Graphic3dTypeOfTexture::TOT_2D,
        );
        if let Some(params) = base.root.params.get() {
            params.set_modulate(false);
            params.set_repeat(false);
            params.set_filter(Graphic3dTypeOfTextureFilter::Bilinear);
            params.set_texture_unit(plane_unit(plane));
        }
        Self {
            transient: StandardTransient::default(),
            base,
            mutex: mutex.clone(),
            frame: SharedState::new(Handle::default()),
            plane: plane as i32,
            pix_map_wrapper: SharedState::new(Handle::default()),
        }
    }
    /// `Frame`; `texture.frame()`.
    pub fn frame(&self) -> Handle<MediaFrame> {
        self.frame.read().clone()
    }
    /// `SetFrame`; `texture.set_frame(&frame)`.
    pub fn set_frame(&self, frame: &Handle<MediaFrame>) {
        *self.frame.write() = frame.clone();
    }
    /// `GenerateNewId`; `texture.generate_new_id()`.
    pub fn generate_new_id(&self) {
        self.base.root.generate_id();
    }
    /// `myPlane`; `texture.plane()`.
    pub fn plane(&self) -> i32 {
        self.plane
    }
}

/// The frame queue part of the set, shared with the player thread.
struct MediaFrameExchange {
    lock: Graphic3dMediaFrameLock,
    frame_pair: [Handle<MediaFrame>; 2],
    callback: SharedState<Option<Graphic3dMediaTextureSetCallback>>,
}

impl MediaFrameExchange {
    fn notify(&self) {
        let callback = self.callback.read().clone();
        if let Some(callback) = callback {
            callback();
        }
    }
}

impl MediaIFrameQueue for MediaFrameExchange {
    /// `LockFrame`: the back frame when no frame waits to be presented and it is free.
    fn lock_frame(&self) -> Handle<MediaFrame> {
        {
            let state = lock(&self.lock);
            if !state.to_present_frame {
                let frame = self.frame_pair[if state.front == 0 { 1 } else { 0 }].clone();
                if frame.get().is_none_or(|f| f.is_locked()) {
                    return Handle::default();
                }
                if let Some(value) = frame.get() {
                    value.set_locked(true);
                }
                return frame;
            }
        }
        self.notify();
        Handle::default()
    }
    /// `ReleaseFrame`: unlocks the frame, marks it to present and calls back.
    fn release_frame(&self, frame: &Handle<MediaFrame>) {
        {
            let mut state = lock(&self.lock);
            if let Some(frame) = frame.get() {
                frame.set_locked(false);
            }
            state.to_present_frame = true;
        }
        self.notify();
    }
}

/// The fields of a media texture set.
#[derive(Clone)]
pub struct Graphic3dMediaTextureSetFields {
    /// `myPlayerCtx`.
    pub player_ctx: Handle<MediaPlayerContext>,
    /// `myInput`.
    pub input: AsciiString,
    /// `myFrameSize`.
    pub frame_size: Vec2<i32>,
    /// `myProgress`.
    pub progress: f64,
    /// `myDuration`.
    pub duration: f64,
    /// `myIsPlanarYUV`.
    pub is_planar_yuv: bool,
    /// `myIsFullRangeYUV`.
    pub is_full_range_yuv: bool,
}

/// The four plane textures of a video and its player; `Graphic3dMediaTextureSet::new()?`.
pub struct Graphic3dMediaTextureSet {
    set: Graphic3dTextureSet,
    exchange: Arc<MediaFrameExchange>,
    /// `myShaderYUV`.
    shader_yuv: Handle<Graphic3dShaderProgram>,
    /// `myShaderYUVJ`.
    shader_yuvj: Handle<Graphic3dShaderProgram>,
    /// The fields, shared by every handle.
    pub fields: SharedState<Graphic3dMediaTextureSetFields>,
}
rustycad_tkernel::impl_transient!(
    Graphic3dMediaTextureSet,
    "Graphic3d_MediaTextureSet",
    Graphic3dTextureSet,
    set
);

impl std::ops::Deref for Graphic3dMediaTextureSet {
    type Target = Graphic3dTextureSet;
    /// The inherited `Graphic3d_TextureSet` members.
    fn deref(&self) -> &Graphic3dTextureSet {
        &self.set
    }
}

impl Graphic3dMediaTextureSet {
    /// Four plane textures, two frames and the YUV shader programs (MPEG and full range);
    /// `Graphic3dMediaTextureSet::new()?`.
    pub fn new() -> Result<Self, Failure> {
        let set = Graphic3dTextureSet::with_size(4)?;
        let mutex: Graphic3dMediaFrameLock =
            Arc::new(Mutex::new(Graphic3dMediaFrameSwap::default()));
        for plane in 0..4usize {
            let texture: Handle<dyn Graphic3dTextureMap> =
                Handle::new(Graphic3dMediaTexture::new(&mutex, plane)).cast();
            set.set_value(set.lower() + plane as i32, texture)?;
        }
        let exchange = MediaFrameExchange {
            lock: mutex,
            frame_pair: [
                Handle::new(MediaFrame::default()),
                Handle::new(MediaFrame::default()),
            ],
            callback: SharedState::new(None),
        };
        let fields = Graphic3dMediaTextureSetFields {
            player_ctx: Handle::default(),
            input: AsciiString::default(),
            frame_size: Vec2::new(0, 0),
            progress: 0.0,
            duration: 0.0,
            is_planar_yuv: false,
            is_full_range_yuv: true,
        };
        Ok(Self {
            set,
            exchange: Arc::new(exchange),
            shader_yuv: yuv_shader_program(false),
            shader_yuvj: yuv_shader_program(true),
            fields: SharedState::new(fields),
        })
    }
    /// `SetCallback`: the function called when a frame is ready; `set.set_callback(Some(callback))`.
    pub fn set_callback(&self, callback: Option<Graphic3dMediaTextureSetCallback>) {
        *self.exchange.callback.write() = callback;
    }
    /// `Notify`: calls the callback; `set.notify()`.
    pub fn notify(&self) {
        self.exchange.notify();
    }
    /// `Input`; `set.input()`.
    pub fn input(&self) -> AsciiString {
        self.fields.read().input.clone()
    }
    /// `OpenInput`: starts the player on a path (an empty path without player only clears
    /// the input); `set.open_input(&path, false)`.
    pub fn open_input(&self, path: &AsciiString, to_wait: bool) {
        let mut player = self.fields.read().player_ctx.clone();
        if player.is_null() {
            if path.is_empty() {
                self.fields.write().input = AsciiString::default();
                return;
            }
            let queue: Arc<dyn MediaIFrameQueue> = self.exchange.clone();
            player = Handle::new(MediaPlayerContext::new(Some(queue)));
            self.fields.write().player_ctx = player.clone();
        }
        {
            let mut fields = self.fields.write();
            fields.progress = 0.0;
            fields.duration = 0.0;
        }
        if let Some(player) = player.get() {
            player.set_input(path, to_wait);
        }
        self.fields.write().input = path.clone();
    }
    /// `PlayerContext`; `set.player_context()`.
    pub fn player_context(&self) -> Handle<MediaPlayerContext> {
        self.fields.read().player_ctx.clone()
    }
    /// `SwapFrames`: presents the filled back frame (updating the playback state, the frame
    /// size, the YUV range and the textures); false when there is nothing to present.
    /// Example: `set.swap_frames()`.
    pub fn swap_frames(&self) -> bool {
        let player = self.player_context();
        let Some(player) = player.get() else {
            return false;
        };
        let (_, progress, duration) = player.playback_state();
        {
            let mut fields = self.fields.write();
            fields.progress = progress;
            fields.duration = duration;
        }
        let mut state = lock(&self.exchange.lock);
        if !state.to_present_frame {
            return false;
        }
        state.to_present_frame = false;
        state.front = if state.front == 0 { 1 } else { 0 };
        let front = self.exchange.frame_pair[state.front].clone();
        self.present(&front);
        true
    }
    /// Takes the size and range of the new front frame and gives it to every media texture.
    fn present(&self, front: &Handle<MediaFrame>) {
        if let Some(frame) = front.get() {
            let mut fields = self.fields.write();
            fields.frame_size = frame.size();
            fields.is_full_range_yuv = frame.is_full_range_yuv();
        }
        for texture in self.set.textures() {
            if let Some(texture) = texture.cast::<Graphic3dMediaTexture>().get() {
                texture.set_frame(front);
                texture.update_revision();
            }
        }
    }
    /// `FrameSize`; `set.frame_size()`.
    pub fn frame_size(&self) -> Vec2<i32> {
        self.fields.read().frame_size
    }
    /// `ShaderProgram`: the YUV program (full or MPEG range) for planar YUV frames, else null;
    /// `set.shader_program()`.
    pub fn shader_program(&self) -> Handle<Graphic3dShaderProgram> {
        let fields = self.fields.read();
        match (fields.is_planar_yuv, fields.is_full_range_yuv) {
            (true, true) => self.shader_yuvj.clone(),
            (true, false) => self.shader_yuv.clone(),
            _ => Handle::default(),
        }
    }
    /// `myShaderYUV`: the reduced-range YUV program; `set.shader_yuv()`.
    pub fn shader_yuv(&self) -> Handle<Graphic3dShaderProgram> {
        self.shader_yuv.clone()
    }
    /// `myShaderYUVJ`: the full-range YUV program; `set.shader_yuvj()`.
    pub fn shader_yuvj(&self) -> Handle<Graphic3dShaderProgram> {
        self.shader_yuvj.clone()
    }
    /// `IsPlanarYUV`; `set.is_planar_yuv()`.
    pub fn is_planar_yuv(&self) -> bool {
        self.fields.read().is_planar_yuv
    }
    /// `IsFullRangeYUV`; `set.is_full_range_yuv()`.
    pub fn is_full_range_yuv(&self) -> bool {
        self.fields.read().is_full_range_yuv
    }
    /// `Duration`; `set.duration()`.
    pub fn duration(&self) -> f64 {
        self.fields.read().duration
    }
    /// `Progress`; `set.progress()`.
    pub fn progress(&self) -> f64 {
        self.fields.read().progress
    }
    /// The frame queue of the set (`LockFrame`/`ReleaseFrame`, private natively, are the
    /// `Media_IFrameQueue` the player fills); `set.frame_queue().lock_frame()`.
    pub fn frame_queue(&self) -> Arc<dyn MediaIFrameQueue> {
        self.exchange.clone()
    }
}
