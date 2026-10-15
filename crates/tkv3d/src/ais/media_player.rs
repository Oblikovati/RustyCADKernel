//! `AIS_MediaPlayer`: a 2D overlay rectangle textured with the frames of a video player.
use super::*;
use crate::prs_mgr::*;
use crate::select_mgr::*;
use crate::select3d::Select3DSensitivePrimitiveArray;
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::{
    Failure, Handle, HandleReference, SharedState, StandardTransient,
};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkmath::toploc::Location;
use rustycad_tkservice::aspect::{
    AspectInteriorStyle, AspectTypeOfLine, AspectTypeOfTriedronPosition,
};
use rustycad_tkservice::graphic3d::*;
use rustycad_tkservice::media::MediaPlayerContext;

/// `createRectangleArray`: the two triangles of a rectangle with flipped vertical texels.
fn create_rectangle_array(
    lower: Vec2<i32>,
    upper: Vec2<i32>,
    flags: Graphic3dArrayFlags,
) -> Result<Handle<Graphic3dArrayOfTriangles>, Failure> {
    let array = Graphic3dArrayOfTriangles::new(4, 6, flags)?;
    let corner = |x: i32, y: i32| Point3::new(f64::from(x), f64::from(y), 0.0);
    array.add_vertex_pnt_texel(&corner(lower.x(), lower.y()), &Point2::new(0.0, 1.0))?;
    array.add_vertex_pnt_texel(&corner(lower.x(), upper.y()), &Point2::new(0.0, 0.0))?;
    array.add_vertex_pnt_texel(&corner(upper.x(), upper.y()), &Point2::new(1.0, 0.0))?;
    array.add_vertex_pnt_texel(&corner(upper.x(), lower.y()), &Point2::new(1.0, 1.0))?;
    array.add_edges3(1, 2, 3)?;
    array.add_edges3(1, 3, 4)?;
    Ok(Handle::new(array))
}

/// The protected fields of `AIS_MediaPlayer`.
#[derive(Clone)]
pub struct AisMediaPlayerFields {
    /// `myFramePair`: the frame textures and the player.
    pub frame_pair: Handle<Graphic3dMediaTextureSet>,
    /// `myFrameAspect`: the unlit textured aspect of the frame.
    pub frame_aspect: Handle<Graphic3dAspectFillArea3d>,
    /// `myFrameBottomLeft`.
    pub frame_bottom_left: Vec2<i32>,
    /// `myFrameSize`.
    pub frame_size: Vec2<i32>,
    /// `myToClosePlayer`.
    pub to_close_player: bool,
}

/// `AIS_MediaPlayer`: draws the current video frame in the lower left corner overlay layer.
/// Example: `let player = AisMediaPlayer::new()?; player.open_input(&path, false);`.
pub struct AisMediaPlayer {
    transient: StandardTransient,
    base: AisInteractiveObjectBase,
    /// The protected fields.
    pub fields: SharedState<AisMediaPlayerFields>,
}
rustycad_tkernel::impl_transient!(AisMediaPlayer, "AIS_MediaPlayer", dyn AisInteractiveObject, transient,
    interfaces: [dyn AisInteractiveObject, dyn SelectMgrSelectableObject, dyn PrsMgrPresentableObject]);

impl AisMediaPlayer {
    /// `AIS_MediaPlayer()`: a 2D lower-left persistence in the top OSD layer, infinite, with a
    /// white unlit textured frame aspect over the frame textures and a 1 x 1 frame.
    /// Example: `AisMediaPlayer::new()?`.
    pub fn new() -> Result<Handle<Self>, Failure> {
        let frame_pair = Handle::new(Graphic3dMediaTextureSet::new()?);
        let material = Graphic3dMaterialAspect::default();
        let aspect = Graphic3dAspectFillArea3d::new(
            AspectInteriorStyle::Solid,
            Color::named(NamedColor::WHITE),
            Color::named(NamedColor::BLACK),
            AspectTypeOfLine::Solid,
            1.0,
            material.clone(),
            material,
        )?;
        aspect.set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
        aspect.set_texture_map_on(true);
        aspect.set_texture_set(frame_pair.clone().cast());
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            base: AisInteractiveObjectBase::new(PrsMgrTypeOfPresentation3d::AllView),
            fields: SharedState::new(AisMediaPlayerFields {
                frame_pair,
                frame_aspect: Handle::new(aspect),
                frame_bottom_left: Vec2::new(0, 0),
                frame_size: Vec2::new(1, 1),
                to_close_player: false,
            }),
        });
        prs_mgr_presentable_bind(&object.cast());
        let persistence = Graphic3dTransformPers::with_corner(
            Graphic3dTransModeFlags::TMF_2D,
            AspectTypeOfTriedronPosition::LEFT_LOWER,
            Vec2::new(0, 0),
        )?;
        object.set_transform_persistence(&Handle::new(persistence).into_cast());
        object.set_z_layer(GRAPHIC3D_ZLAYER_ID_TOP_OSD);
        object.set_infinite_state(true);
        Ok(object)
    }
    /// `SetCallback`: the function called when a frame is ready; `player.set_callback(Some(f))`.
    pub fn set_callback(&self, callback: Option<Graphic3dMediaTextureSetCallback>) {
        self.fields.read().frame_pair.set_callback(callback);
    }
    /// `OpenInput`: nothing for an empty path without player; then the aspects are
    /// synchronized. Example: `player.open_input(&path, false)`.
    pub fn open_input(&self, path: &AsciiString, to_wait: bool) {
        let frame_pair = self.fields.read().frame_pair.clone();
        if frame_pair.player_context().is_null() && path.is_empty() {
            return;
        }
        frame_pair.open_input(path, to_wait);
        self.synchronize_aspects();
    }
    /// `PlayerContext`; `player.player_context()`.
    pub fn player_context(&self) -> Handle<MediaPlayerContext> {
        self.fields.read().frame_pair.player_context()
    }
    /// `PlayPause`: toggles the player when there is one; `player.play_pause()`.
    pub fn play_pause(&self) {
        let player = self.player_context();
        if let Some(player) = player.get() {
            player.play_pause();
        }
    }
    /// `SetClosePlayer`: the next `PresentFrame` removes the player; `player.set_close_player()`.
    pub fn set_close_player(&self) {
        self.fields.write().to_close_player = true;
        self.fields.read().frame_pair.notify();
    }
    /// `Duration`; `player.duration()`.
    pub fn duration(&self) -> f64 {
        self.fields.read().frame_pair.duration()
    }
    /// `PresentFrame`: removes a player asked to close (pausing it), else swaps the frames and
    /// fits the frame into the area; true when a redraw is needed.
    /// Example: `player.present_frame(Vec2::new(0, 0), Vec2::new(640, 480))?`.
    pub fn present_frame(
        &self,
        left_corner: Vec2<i32>,
        max_size: Vec2<i32>,
    ) -> Result<bool, Failure> {
        if self.fields.read().to_close_player {
            self.fields.write().to_close_player = false;
            if !self.has_interactive_context() {
                return Ok(false);
            }
            let player = self.player_context();
            if let Some(player) = player.get() {
                player.pause();
            }
            let context = self.get_context();
            let this: Handle<dyn AisInteractiveObject> = self.presentable_base().this().cast();
            context.remove(&this, false)?;
            context.current_viewer().invalidate();
            return Ok(true);
        }
        let frame_pair = self.fields.read().frame_pair.clone();
        if frame_pair.player_context().is_null() {
            return Ok(false);
        }
        let mut redraw = frame_pair.swap_frames();
        redraw = self.update_size(left_corner, max_size)? || redraw;
        if redraw {
            self.fields
                .read()
                .frame_aspect
                .set_shader_program(frame_pair.shader_program());
            self.synchronize_aspects();
        }
        Ok(redraw)
    }
    /// `updateSize` (protected): the frame fitted with its aspect ratio and centered in the
    /// area (the whole area while the frame size is unknown and the frame is tiny); the
    /// object is redisplayed when it moved. Example: `player.update_size(corner, size)?`.
    pub fn update_size(
        &self,
        left_corner: Vec2<i32>,
        max_size: Vec2<i32>,
    ) -> Result<bool, Failure> {
        let frame_size = self.fields.read().frame_pair.frame_size();
        let (current_size, current_pos) = {
            let fields = self.fields.read();
            (fields.frame_size, fields.frame_bottom_left)
        };
        let mut new_pos = left_corner;
        let mut new_size = current_size;
        if frame_size.x() > 0 && frame_size.y() > 0 {
            let aspect = f64::from(max_size.x()) / f64::from(max_size.y());
            let fit_aspect = f64::from(frame_size.x()) / f64::from(frame_size.y());
            new_size = frame_size;
            if fit_aspect >= aspect {
                *new_size.y_mut() = (f64::from(frame_size.x()) / fit_aspect) as i32;
            } else {
                *new_size.x_mut() = (f64::from(frame_size.y()) * fit_aspect) as i32;
            }
            for coord in 0..2 {
                if new_size[coord] > max_size[coord] {
                    let scale = f64::from(max_size[coord]) / f64::from(new_size[coord]);
                    *new_size.x_mut() = (f64::from(new_size.x()) * scale) as i32;
                    *new_size.y_mut() = (f64::from(new_size.y()) * scale) as i32;
                }
            }
            new_pos = left_corner + max_size / 2 - new_size / 2;
        } else if current_size.x() < 2 || current_size.y() < 2 {
            new_size = max_size;
        }
        if current_size == new_size && current_pos == new_pos {
            return Ok(false);
        }
        {
            let mut fields = self.fields.write();
            fields.frame_size = new_size;
            fields.frame_bottom_left = new_pos;
        }
        if self.has_interactive_context() {
            self.set_to_update(-1);
            let context = self.get_context();
            let this: Handle<dyn AisInteractiveObject> = self.presentable_base().this().cast();
            context.redisplay(&this, false, false)?;
            context.current_viewer().invalidate();
        }
        Ok(true)
    }
}

impl AisInteractiveObject for AisMediaPlayer {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.base
    }
}

impl PrsMgrPresentableObject for AisMediaPlayer {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.base.presentable
    }
    /// `AcceptDisplayMode`: mode 0 only.
    fn accept_display_mode(&self, mode: i32) -> bool {
        mode == 0
    }
    /// `Compute`: the infinite state, then in mode 0 the textured frame rectangle in a new group.
    fn compute(
        &self,
        _: &dyn HandleReference<PrsMgrPresentationManager>,
        prs: &dyn HandleReference<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        let prs = prs.observe();
        prs.set_infinite_state(self.is_infinite());
        if mode != 0 {
            return Ok(());
        }
        let fields = self.fields.read().clone();
        let triangles = create_rectangle_array(
            fields.frame_bottom_left,
            fields.frame_bottom_left + fields.frame_size,
            GRAPHIC3D_ARRAY_FLAGS_VERTEX_TEXEL,
        )?;
        let group = prs.new_group()?;
        group.set_group_primitives_aspect(&fields.frame_aspect.cast());
        group.add_primitive_array(&triangles.cast(), true);
        Ok(())
    }
    crate::ais_presentable_overrides!();
}

impl SelectMgrSelectableObject for AisMediaPlayer {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.base.selectable
    }
    /// `ComputeSelection`: in mode 0, the triangles of the frame rectangle (owner priority 5).
    fn compute_selection(
        &self,
        selection: &dyn HandleReference<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure> {
        if mode != 0 {
            return Ok(());
        }
        let fields = self.fields.read().clone();
        let triangles = create_rectangle_array(
            fields.frame_bottom_left,
            fields.frame_bottom_left + fields.frame_size,
            0,
        )?;
        let this: Handle<dyn SelectMgrSelectableObject> = self.presentable_base().this().cast();
        let owner = SelectMgrEntityOwner::with_selectable(&this, 5).into_cast();
        let sensitive = Select3DSensitivePrimitiveArray::new(&owner)?;
        sensitive.init_triangulation_all(
            &triangles.attributes(),
            &triangles.indices(),
            &Location::default(),
            true,
            1,
        )?;
        selection.observe().add(&Handle::new(sensitive).cast())
    }
}
