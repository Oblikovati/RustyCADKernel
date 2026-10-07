//! `Graphic3d_CameraTile` (a sub-area of the view to render) and
//! `Graphic3d_WorldViewProjState` (the change counters of a camera).
use crate::json_dump::{field_dumped, field_integer, field_pointer, field_unsigned};
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkernel::standard::dump::{DumpStream, dump_vector};
use std::io;

/// A tile of the view area; `Graphic3dCameraTile::default().is_valid() == false`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Graphic3dCameraTile {
    /// `TotalSize`: the view area in pixels.
    pub total_size: Vec2<i32>,
    /// `TileSize`: the tile in pixels.
    pub tile_size: Vec2<i32>,
    /// `Offset`: the tile corner, lower-left (or upper-left with `is_top_down`).
    pub offset: Vec2<i32>,
    /// `IsTopDown`: the offset is measured from the top.
    pub is_top_down: bool,
}

impl Default for Graphic3dCameraTile {
    /// Zero sizes and offset, lower-left origin.
    fn default() -> Self {
        let zero = Vec2::new(0, 0);
        Self {
            total_size: zero,
            tile_size: zero,
            offset: zero,
            is_top_down: false,
        }
    }
}

impl Graphic3dCameraTile {
    /// Whether both sizes are positive; `tile.is_valid()`.
    pub fn is_valid(&self) -> bool {
        self.total_size.x > 0
            && self.total_size.y > 0
            && self.tile_size.x > 0
            && self.tile_size.y > 0
    }
    /// The offset from the lower-left corner; `tile.offset_lower_left()`.
    pub fn offset_lower_left(&self) -> Vec2<i32> {
        let y = if self.is_top_down {
            self.total_size.y - self.offset.y - 1
        } else {
            self.offset.y
        };
        Vec2::new(self.offset.x, y)
    }
    /// The tile clipped to the view area (an invalid tile is unchanged); `tile.cropped()`.
    pub fn cropped(&self) -> Self {
        let mut tile = *self;
        if !self.is_valid() {
            return tile;
        }
        tile.offset = Vec2::new(self.offset.x.max(0), self.offset.y.max(0));
        let x = (self.offset.x + self.tile_size.x).min(self.total_size.x);
        let y = (self.offset.y + self.tile_size.y).min(self.total_size.y);
        tile.tile_size = Vec2::new(x - self.offset.x, y - self.offset.y);
        tile
    }
    /// `operator==`: sizes and lower-left offsets equal; `tile.is_same(&other)`.
    pub fn is_same(&self, other: &Self) -> bool {
        let (a, b) = (self.offset_lower_left(), other.offset_lower_left());
        self.total_size == other.total_size && self.tile_size == other.tile_size && a == b
    }
    /// `DumpJson`. Native quirk: the integer vectors go through the double variadic dump
    /// macro (undefined natively); here they print as numbers. Example: `tile.dump_json(&mut s, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        for (field, value) in [
            ("&TotalSize", self.total_size),
            ("&TileSize", self.tile_size),
            ("&Offset", self.offset),
        ] {
            field_dumped(stream, depth, field, &|s, _| {
                dump_vector(s, "Vec2", &[f64::from(value.x), f64::from(value.y)])
            })?;
        }
        field_integer(stream, "IsTopDown", i32::from(self.is_top_down))
    }
}

/// The projection and world-view counters of a camera, with the camera identity.
///
/// Example: `Graphic3dWorldViewProjState::new(1, 2, 0).is_valid()`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Graphic3dWorldViewProjState {
    is_valid: bool,
    /// `myCamera` as an address (0 for none).
    camera: usize,
    projection_state: usize,
    world_view_state: usize,
}

impl Graphic3dWorldViewProjState {
    /// `Graphic3d_WorldViewProjState(projection, worldView, camera)`; `Graphic3dWorldViewProjState::new(1, 2, 0)`.
    pub fn new(projection_state: usize, world_view_state: usize, camera: usize) -> Self {
        Self {
            is_valid: true,
            camera,
            projection_state,
            world_view_state,
        }
    }
    /// Whether the state was initialised; `state.is_valid()`.
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }
    /// Back to the invalid state of the default constructor; `state.reset()`.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    /// `Initialize(projection, worldView, camera)`; `state.initialize(1, 2, 0)`.
    pub fn initialize(&mut self, projection_state: usize, world_view_state: usize, camera: usize) {
        *self = Self::new(projection_state, world_view_state, camera);
    }
    /// `Initialize(camera)`: zero counters; `state.initialize_camera(0)`.
    pub fn initialize_camera(&mut self, camera: usize) {
        *self = Self::new(0, 0, camera);
    }
    /// The projection counter; `state.projection_state()`.
    pub fn projection_state(&self) -> usize {
        self.projection_state
    }
    /// Sets the projection counter (`ProjectionState()` reference); `state.set_projection_state(3)`.
    pub fn set_projection_state(&mut self, value: usize) {
        self.projection_state = value;
    }
    /// The world-view counter; `state.world_view_state()`.
    pub fn world_view_state(&self) -> usize {
        self.world_view_state
    }
    /// Sets the world-view counter (`WorldViewState()` reference); `state.set_world_view_state(3)`.
    pub fn set_world_view_state(&mut self, value: usize) {
        self.world_view_state = value;
    }
    /// Whether validity, camera or projection counter differ; `state.is_projection_changed(&other)`.
    pub fn is_projection_changed(&self, other: &Self) -> bool {
        self.is_valid != other.is_valid
            || self.camera != other.camera
            || self.projection_state != other.projection_state
    }
    /// Whether validity, camera or world-view counter differ; `state.is_world_view_changed(&other)`.
    pub fn is_world_view_changed(&self, other: &Self) -> bool {
        self.is_valid != other.is_valid
            || self.camera != other.camera
            || self.world_view_state != other.world_view_state
    }
    /// Whether anything differs; `state.is_changed(&other)`.
    pub fn is_changed(&self, other: &Self) -> bool {
        self != other
    }
    /// `DumpJson`; `state.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, _depth: i32) -> io::Result<()> {
        field_integer(stream, "myIsValid", i32::from(self.is_valid))?;
        field_pointer(stream, "myCamera", self.camera)?;
        field_unsigned(stream, "myProjectionState", self.projection_state as u64)?;
        field_unsigned(stream, "myWorldViewState", self.world_view_state as u64)
    }
}
