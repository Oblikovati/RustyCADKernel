//! `Graphic3d_BoundBuffer`: the bounds (sub-group vertex or edge counts) of a primitive array
//! and their optional colors, in one allocation (colors first, then the counts).
use super::Graphic3dBufferValue;
use crate::json_dump::{base_class, class_begin, field_integer, field_pointer};
use rustycad_tkernel::ncollection::{BaseAllocator, Buffer, Vec4};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient};
use std::io;

/// The fields of `Graphic3d_BoundBuffer`.
pub struct Graphic3dBoundBufferState {
    /// The `NCollection_Buffer` part: colors (`Vec4<f32>` each), then bound counts (`int` each).
    pub data: Buffer,
    /// Whether the buffer has colors (`Colors` not null).
    pub has_colors: bool,
    /// `NbBounds`.
    pub nb_bounds: i32,
    /// `NbMaxBounds`.
    pub nb_max_bounds: i32,
}

/// A bound buffer; `let bounds = Graphic3dBoundBuffer::new(allocator)?; bounds.init(2, true)?;`.
pub struct Graphic3dBoundBuffer {
    transient: StandardTransient,
    state: SharedState<Graphic3dBoundBufferState>,
}
rustycad_tkernel::impl_transient!(
    Graphic3dBoundBuffer,
    "Graphic3d_BoundBuffer",
    Buffer,
    transient
);

impl Graphic3dBoundBuffer {
    /// An empty bound buffer over an allocator; `Graphic3dBoundBuffer::new(allocator)?`.
    pub fn new(allocator: BaseAllocator) -> Result<Self, Failure> {
        let state = Graphic3dBoundBufferState {
            data: Buffer::new(Some(allocator), 0)?,
            has_colors: false,
            nb_bounds: 0,
            nb_max_bounds: 0,
        };
        Ok(Self {
            transient: StandardTransient::default(),
            state: SharedState::new(state),
        })
    }
    /// The fields; `bounds.state().nb_bounds`.
    pub fn state(&self) -> std::sync::RwLockReadGuard<'_, Graphic3dBoundBufferState> {
        self.state.read()
    }
    /// `Init`: room for `count` bounds (and colors); false below 1 bound or on a failed
    /// allocation. Example: `bounds.init(2, true)?`.
    pub fn init(&self, count: i32, has_colors: bool) -> Result<bool, Failure> {
        let mut state = self.state.write();
        (state.has_colors, state.nb_bounds, state.nb_max_bounds) = (false, 0, 0);
        state.data.free()?;
        if count < 1 {
            return Ok(false);
        }
        let colors = if has_colors { 16 * count as usize } else { 0 };
        if !state.data.allocate(colors + 4 * count as usize)? {
            state.data.free()?;
            return Ok(false);
        }
        (state.has_colors, state.nb_bounds, state.nb_max_bounds) = (has_colors, count, count);
        Ok(true)
    }
    /// The byte offset of the bound counts.
    fn bounds_offset(state: &Graphic3dBoundBufferState) -> usize {
        if state.has_colors {
            16 * state.nb_max_bounds.max(0) as usize
        } else {
            0
        }
    }
    /// `Bounds[index]`; `None` past the allocation (undefined natively). `bounds.bound(0)`.
    pub fn bound(&self, index: i32) -> Option<i32> {
        let state = self.state.read();
        let offset = Self::bounds_offset(&state).checked_add(4 * usize::try_from(index).ok()?)?;
        state.data.as_slice().get(offset..offset + 4).map(i32::read)
    }
    /// `Bounds[index] = value`; false past the allocation. `bounds.set_bound(0, 3)`.
    pub fn set_bound(&self, index: i32, value: i32) -> bool {
        let mut state = self.state.write();
        let Some(offset) = usize::try_from(index)
            .ok()
            .map(|i| Self::bounds_offset(&state) + 4 * i)
        else {
            return false;
        };
        state
            .data
            .as_mut_slice()
            .get_mut(offset..offset + 4)
            .map(|bytes| value.write(bytes))
            .is_some()
    }
    /// `Colors[index]`; `None` without colors or past them. `bounds.color(0)`.
    pub fn color(&self, index: i32) -> Option<Vec4<f32>> {
        let state = self.state.read();
        let slot = usize::try_from(index)
            .ok()
            .filter(|i| state.has_colors && (*i as i32) < state.nb_max_bounds)?;
        state
            .data
            .as_slice()
            .get(16 * slot..16 * slot + 16)
            .map(Vec4::<f32>::read)
    }
    /// `Colors[index] = value`; false without colors or past them. `bounds.set_color(0, rgba)`.
    pub fn set_color(&self, index: i32, value: Vec4<f32>) -> bool {
        let mut state = self.state.write();
        let Some(slot) = usize::try_from(index)
            .ok()
            .filter(|i| state.has_colors && (*i as i32) < state.nb_max_bounds)
        else {
            return false;
        };
        state
            .data
            .as_mut_slice()
            .get_mut(16 * slot..16 * slot + 16)
            .map(|bytes| value.write(bytes))
            .is_some()
    }
    /// Sets the bound count in use (the public `NbBounds` field); `bounds.set_nb_bounds(1)`.
    pub fn set_nb_bounds(&self, count: i32) {
        self.state.write().nb_bounds = count;
    }
    /// `DumpJson`: class name, buffer, color and bound addresses and counts.
    /// Example: `bounds.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "Graphic3d_BoundBuffer")?;
        let state = self.state.read();
        base_class(stream, depth, &|s, d| state.data.dump_json(s, d))?;
        let base = state.data.data_ptr() as usize;
        field_pointer(stream, "Colors", if state.has_colors { base } else { 0 })?;
        field_pointer(
            stream,
            "Bounds",
            if base == 0 {
                0
            } else {
                base + Self::bounds_offset(&state)
            },
        )?;
        field_integer(stream, "NbBounds", state.nb_bounds)?;
        field_integer(stream, "NbMaxBounds", state.nb_max_bounds)
    }
}
