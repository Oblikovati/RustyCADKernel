//! `Graphic3d_SequenceOfHClipPlane`: the clipping planes of an object or a view, each plane
//! at most once, with the flag that overrides the global planes.
use super::Graphic3dClipPlane;
use crate::json_dump::{field_dumped, field_integer};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::exceptions::{no_such_object, out_of_range};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use std::io;

#[derive(Clone, Default)]
struct ClipPlaneSequenceState {
    items: Vec<Handle<Graphic3dClipPlane>>,
    to_override_global: bool,
}

/// A sequence of distinct clip planes; `Graphic3dSequenceOfHClipPlane::default().append(plane)`.
#[derive(Default)]
pub struct Graphic3dSequenceOfHClipPlane {
    transient: StandardTransient,
    state: SharedState<ClipPlaneSequenceState>,
}
rustycad_tkernel::impl_transient!(
    Graphic3dSequenceOfHClipPlane,
    "Graphic3d_SequenceOfHClipPlane",
    StandardTransient,
    transient,
    interfaces: [dyn super::Graphic3dClipPlaneSequenceSource]
);

/// `Graphic3d_SequenceOfHClipPlane::Iterator`: a cursor over a snapshot of the planes.
/// Example: `let mut it = Graphic3dSequenceOfHClipPlaneIterator::new(&planes); while it.more() { it.next(); }`.
#[derive(Clone, Default)]
pub struct Graphic3dSequenceOfHClipPlaneIterator {
    items: Vec<Handle<Graphic3dClipPlane>>,
    index: usize,
}

impl Graphic3dSequenceOfHClipPlaneIterator {
    /// An iterator at the first plane of `planes`; `Graphic3dSequenceOfHClipPlaneIterator::new(&planes)`.
    pub fn new(planes: &Graphic3dSequenceOfHClipPlane) -> Self {
        Self {
            items: planes.items(),
            index: 0,
        }
    }
    /// An iterator over a handle; a null handle gives an empty iterator.
    /// Example: `Graphic3dSequenceOfHClipPlaneIterator::from_handle(&handle)`.
    pub fn from_handle(planes: &Handle<Graphic3dSequenceOfHClipPlane>) -> Self {
        planes.get().map(Self::new).unwrap_or_default()
    }
    /// Restarts on `planes`; `it.init(&planes)`.
    pub fn init(&mut self, planes: &Graphic3dSequenceOfHClipPlane) {
        *self = Self::new(planes);
    }
    /// Restarts on a handle (null empties the iterator); `it.init_handle(&handle)`.
    pub fn init_handle(&mut self, planes: &Handle<Graphic3dSequenceOfHClipPlane>) {
        *self = Self::from_handle(planes);
    }
    /// Whether a current plane exists; `it.more()`.
    pub fn more(&self) -> bool {
        self.index < self.items.len()
    }
    /// Moves to the next plane; `it.next()`.
    pub fn next(&mut self) {
        self.index += 1;
    }
    /// The current plane, null past the end; `it.value()`.
    pub fn value(&self) -> Handle<Graphic3dClipPlane> {
        self.items.get(self.index).cloned().unwrap_or_default()
    }
}

impl Graphic3dSequenceOfHClipPlane {
    /// The planes in order; `planes.items()`.
    pub fn items(&self) -> Vec<Handle<Graphic3dClipPlane>> {
        self.state.read().items.clone()
    }
    /// Whether these planes replace the global ones; `planes.to_override_global()`.
    pub fn to_override_global(&self) -> bool {
        self.state.read().to_override_global
    }
    /// Sets the override flag; `planes.set_override_global(true)`.
    pub fn set_override_global(&self, to_override: bool) {
        self.state.write().to_override_global = to_override;
    }
    /// Whether the sequence is empty; `planes.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.state.read().items.is_empty()
    }
    /// The number of planes; `planes.size()`.
    pub fn size(&self) -> i32 {
        i32::try_from(self.state.read().items.len()).unwrap_or(i32::MAX)
    }
    /// Appends a plane unless it is already present; `planes.append(plane)`.
    pub fn append(&self, plane: Handle<Graphic3dClipPlane>) -> bool {
        let mut state = self.state.write();
        if state.items.contains(&plane) {
            return false;
        }
        state.items.push(plane);
        true
    }
    /// Removes a plane; false when absent. Example: `planes.remove(&plane)`.
    pub fn remove(&self, plane: &Handle<Graphic3dClipPlane>) -> bool {
        let mut state = self.state.write();
        let Some(position) = state.items.iter().position(|item| item == plane) else {
            return false;
        };
        state.items.remove(position);
        true
    }
    /// Removes current and advances it; an end iterator is unchanged, as natively.
    /// Example: `planes.remove_at(&mut it)?` also accepts a default iterator.
    pub fn remove_at(&self, it: &mut Graphic3dSequenceOfHClipPlaneIterator) -> Result<(), Failure> {
        // OCCT 3d097a0 BaseSequence::Remove returns before touching null CurrentNode.
        if !it.more() {
            return Ok(());
        }
        let mut state = self.state.write();
        if it.index >= state.items.len() {
            return Err(out_of_range(&format!(
                "Graphic3d_SequenceOfHClipPlane::Remove() - iterator position {} of {} planes; expected a current plane",
                it.index,
                state.items.len()
            )));
        }
        state.items.remove(it.index);
        it.items.remove(it.index);
        Ok(())
    }
    /// Removes every plane; `planes.clear()`.
    pub fn clear(&self) {
        self.state.write().items.clear();
    }
    /// The first plane; an empty sequence raises `Standard_NoSuchObject`. `planes.first()?`.
    pub fn first(&self) -> Result<Handle<Graphic3dClipPlane>, Failure> {
        self.state.read().items.first().cloned().ok_or_else(|| {
            no_such_object(
                "NCollection_Sequence::First - empty sequence; expected at least one plane",
            )
        })
    }
    /// `DumpJson`: the override flag, then each plane; `planes.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let state = self.state.read().clone();
        field_integer(
            stream,
            "myToOverrideGlobal",
            i32::from(state.to_override_global),
        )?;
        for plane in &state.items {
            field_dumped(stream, depth, "aClipPlane.get()", &|o, d| {
                plane.dump_json(o, d)
            })?;
        }
        Ok(())
    }
}
