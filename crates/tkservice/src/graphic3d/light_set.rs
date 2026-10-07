//! `Graphic3d_LightSet`: the light sources of a view with the cached per-type counters,
//! cumulative ambient color and the shader keys of the enabled lights.
use super::{Graphic3dCLight, Graphic3dTypeOfLightSource};
use rustycad_tkernel::ncollection::Vec4;
use rustycad_tkernel::standard::exceptions::{out_of_range, program_error};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;

/// The key letter of each light type (`THE_LIGHT_KEY_LETTERS`).
const THE_LIGHT_KEY_LETTERS: [u8; 4] = *b"adps";

/// The filters of [`Graphic3dLightSetIterator`] (native `Graphic3d_LightSet::IterationFilter`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Graphic3dLightSetIterationFilter {
    /// `IterationFilter_None`: no filter.
    None = 0x0000,
    /// `IterationFilter_ExcludeAmbient`: skip ambient lights.
    ExcludeAmbient = 0x0002,
    /// `IterationFilter_ExcludeDisabled`: skip disabled lights.
    ExcludeDisabled = 0x0004,
    /// `IterationFilter_ExcludeNoShadow`: skip lights not casting shadows.
    ExcludeNoShadow = 0x0008,
    /// `IterationFilter_ExcludeDisabledAndAmbient`.
    ExcludeDisabledAndAmbient = 0x0006,
    /// `IterationFilter_ActiveShadowCasters`.
    ActiveShadowCasters = 0x000E,
}

/// The fields of a light set, shared by every handle to it.
#[derive(Clone)]
struct LightSetState {
    /// The lights in index order with the light revision cached by `UpdateRevision`.
    lights: Vec<(Handle<Graphic3dCLight>, usize)>,
    ambient: Vec4<f32>,
    key_enabled_long: AsciiString,
    key_enabled_short: AsciiString,
    light_types: [i32; 4],
    light_types_enabled: [i32; 4],
    nb_enabled: i32,
    nb_cast_shadows: i32,
    revision: usize,
    cache_revision: usize,
}

/// A set of lights; `let set = Graphic3dLightSet::default(); set.add(&light)?;`.
pub struct Graphic3dLightSet {
    transient: StandardTransient,
    state: SharedState<LightSetState>,
}
rustycad_tkernel::impl_transient!(
    Graphic3dLightSet,
    "Graphic3d_LightSet",
    StandardTransient,
    transient
);

impl Default for Graphic3dLightSet {
    fn default() -> Self {
        let state = LightSetState {
            lights: Vec::new(),
            ambient: Vec4::new(0.0, 0.0, 0.0, 0.0),
            key_enabled_long: AsciiString::default(),
            key_enabled_short: AsciiString::default(),
            light_types: [0; 4],
            light_types_enabled: [0; 4],
            nb_enabled: 0,
            nb_cast_shadows: 0,
            revision: 1,
            cache_revision: 0,
        };
        Self {
            transient: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
}

impl Graphic3dLightSet {
    /// The lower index (1); `set.lower()`.
    pub fn lower(&self) -> i32 {
        1
    }
    /// The upper index; `set.upper()`.
    pub fn upper(&self) -> i32 {
        self.extent()
    }
    /// Whether the set is empty; `set.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.state.read().lights.is_empty()
    }
    /// The number of lights; `set.extent()`.
    pub fn extent(&self) -> i32 {
        i32::try_from(self.state.read().lights.len()).unwrap_or(i32::MAX)
    }
    /// The light at a 1-based index; an index outside `1..=Extent` raises `Standard_OutOfRange`.
    /// Example: `set.value(1)?`.
    pub fn value(&self, index: i32) -> Result<Handle<Graphic3dCLight>, Failure> {
        let state = self.state.read();
        let slot = usize::try_from(index - 1).ok();
        slot.and_then(|i| state.lights.get(i))
            .map(|(light, _)| light.clone())
            .ok_or_else(|| {
                out_of_range(&format!(
                    "NCollection_IndexedDataMap::FindKey - index {index}; expected 1..{}",
                    state.lights.len()
                ))
            })
    }
    /// The lights in index order; `set.lights()`.
    pub fn lights(&self) -> Vec<Handle<Graphic3dCLight>> {
        self.state
            .read()
            .lights
            .iter()
            .map(|(light, _)| light.clone())
            .collect()
    }
    /// Whether the light is in the set; `set.contains(&light)`.
    pub fn contains(&self, light: &Handle<Graphic3dCLight>) -> bool {
        self.state
            .read()
            .lights
            .iter()
            .any(|(item, _)| item == light)
    }
    /// Adds a light (false when already present); a null light raises `Standard_ProgramError`.
    /// Example: `set.add(&light)?`.
    pub fn add(&self, light: &Handle<Graphic3dCLight>) -> Result<bool, Failure> {
        let Some(source) = light.get() else {
            return Err(program_error(
                "Graphic3d_LightSet::Add(), NULL argument; expected a light",
            ));
        };
        if self.contains(light) {
            return Ok(false);
        }
        let (kind, revision) = (source.light_type(), source.revision());
        let mut state = self.state.write();
        state.lights.push((light.clone(), revision));
        state.light_types[kind as usize] += 1;
        state.revision = state.revision.wrapping_add(1);
        Ok(true)
    }
    /// Removes a light (the last light takes its index, as `RemoveFromIndex` does); false
    /// when absent. Example: `set.remove(&light)`.
    pub fn remove(&self, light: &Handle<Graphic3dCLight>) -> bool {
        let mut state = self.state.write();
        let Some(position) = state.lights.iter().position(|(item, _)| item == light) else {
            return false;
        };
        state.revision = state.revision.wrapping_add(1);
        let (removed, _) = state.lights.swap_remove(position);
        if let Some(removed) = removed.get() {
            state.light_types[removed.light_type() as usize] -= 1;
        }
        true
    }
    /// The number of lights of a type; `set.nb_lights_of_type(Graphic3dTypeOfLightSource::Spot)`.
    pub fn nb_lights_of_type(&self, kind: Graphic3dTypeOfLightSource) -> i32 {
        self.state.read().light_types[kind as usize]
    }
}

impl Graphic3dLightSet {
    /// The set revision; `set.revision()`.
    pub fn revision(&self) -> usize {
        self.state.read().revision
    }
    /// The enabled non-ambient lights at the last update; `set.nb_enabled()`.
    pub fn nb_enabled(&self) -> i32 {
        self.state.read().nb_enabled
    }
    /// The enabled lights of a type at the last update; `set.nb_enabled_lights_of_type(kind)`.
    pub fn nb_enabled_lights_of_type(&self, kind: Graphic3dTypeOfLightSource) -> i32 {
        self.state.read().light_types_enabled[kind as usize]
    }
    /// The enabled shadow casters at the last update; `set.nb_cast_shadows()`.
    pub fn nb_cast_shadows(&self) -> i32 {
        self.state.read().nb_cast_shadows
    }
    /// The cumulative ambient color (alpha 1) at the last update; `set.ambient_color()`.
    pub fn ambient_color(&self) -> Vec4<f32> {
        self.state.read().ambient
    }
    /// One letter per enabled non-ambient light (upper case for shadow casters).
    /// Example: `set.key_enabled_long()`.
    pub fn key_enabled_long(&self) -> AsciiString {
        self.state.read().key_enabled_long.clone()
    }
    /// One letter per enabled non-ambient light type (`d`, `p`, `s`); `set.key_enabled_short()`.
    pub fn key_enabled_short(&self) -> AsciiString {
        self.state.read().key_enabled_short.clone()
    }
    /// `UpdateRevision`: bumps the revision when a light changed since the last update, then
    /// recomputes the cached counters, ambient color and keys. Example: `set.update_revision()`.
    pub fn update_revision(&self) -> usize {
        let mut state = self.state.write();
        if state.cache_revision == state.revision
            && state
                .lights
                .iter()
                .any(|(light, cached)| *cached != light.revision())
        {
            state.revision = state.revision.wrapping_add(1);
        }
        if state.cache_revision != state.revision {
            recompute_cache(&mut state);
        }
        state.revision
    }
}

/// The cache recomputation of `UpdateRevision`.
fn recompute_cache(state: &mut LightSetState) {
    use Graphic3dTypeOfLightSource as T;
    state.cache_revision = state.revision;
    state.nb_cast_shadows = 0;
    state.ambient = Vec4::new(0.0, 0.0, 0.0, 0.0);
    state.light_types_enabled = [0; 4];
    let mut key_long = Vec::new();
    let mut lights = std::mem::take(&mut state.lights);
    for (light, cached) in &mut lights {
        *cached = light.revision();
        if !light.is_enabled() {
            continue;
        }
        let kind = light.light_type();
        state.light_types_enabled[kind as usize] += 1;
        if kind == T::Ambient {
            state.ambient += light.packed_color() * light.intensity();
            continue;
        }
        let letter = THE_LIGHT_KEY_LETTERS[kind as usize];
        let casts = light.to_cast_shadows();
        state.nb_cast_shadows += i32::from(casts);
        key_long.push(if casts {
            letter.to_ascii_uppercase()
        } else {
            letter
        });
    }
    state.lights = lights;
    state.ambient.w = 1.0;
    let enabled = state.light_types_enabled;
    state.nb_enabled = enabled[1] + enabled[2] + enabled[3];
    state.key_enabled_long = AsciiString::from(String::from_utf8_lossy(&key_long).as_ref());
    let short: Vec<u8> = (1..4)
        .filter(|&kind| enabled[kind] > 0)
        .map(|kind| THE_LIGHT_KEY_LETTERS[kind])
        .collect();
    state.key_enabled_short = AsciiString::from(String::from_utf8_lossy(&short).as_ref());
}

/// `Graphic3d_LightSet::Iterator`: the lights of a set (snapshot of the handles), skipping
/// those excluded by the filter when it reaches them.
/// Example: `let it = Graphic3dLightSetIterator::new(&set, Graphic3dLightSetIterationFilter::ExcludeAmbient);`.
#[derive(Clone, Default)]
pub struct Graphic3dLightSetIterator {
    items: Vec<Handle<Graphic3dCLight>>,
    index: usize,
    filter: i32,
}

impl Graphic3dLightSetIterator {
    /// An iterator over `set` with `filter`; `Graphic3dLightSetIterator::new(&set, filter)`.
    pub fn new(set: &Graphic3dLightSet, filter: Graphic3dLightSetIterationFilter) -> Self {
        let mut it = Self {
            items: set.lights(),
            index: 0,
            filter: filter as i32,
        };
        it.skip_filtered();
        it
    }
    /// An iterator over a handle; a null set gives an empty iterator.
    /// Example: `Graphic3dLightSetIterator::from_handle(&set, filter)`.
    pub fn from_handle(
        set: &Handle<Graphic3dLightSet>,
        filter: Graphic3dLightSetIterationFilter,
    ) -> Self {
        match set.get() {
            Some(set) => Self::new(set, filter),
            None => Self {
                filter: filter as i32,
                ..Self::default()
            },
        }
    }
    /// Whether a current light exists; `it.more()`.
    pub fn more(&self) -> bool {
        self.index < self.items.len()
    }
    /// The current light, null past the end; `it.value()`.
    pub fn value(&self) -> Handle<Graphic3dCLight> {
        self.items.get(self.index).cloned().unwrap_or_default()
    }
    /// Moves to the next light that passes the filter; `it.next()`.
    pub fn next(&mut self) {
        self.index += 1;
        self.skip_filtered();
    }
    /// `skipFiltered`: advances past the lights that the filter excludes.
    fn skip_filtered(&mut self) {
        while self
            .items
            .get(self.index)
            .is_some_and(|light| self.excludes(light))
        {
            self.index += 1;
        }
    }
    fn excludes(&self, light: &Graphic3dCLight) -> bool {
        use Graphic3dLightSetIterationFilter as F;
        let has = |flag: F| self.filter & flag as i32 != 0;
        (has(F::ExcludeAmbient) && light.light_type() == Graphic3dTypeOfLightSource::Ambient)
            || (has(F::ExcludeDisabled) && !light.is_enabled())
            || (has(F::ExcludeNoShadow) && !light.to_cast_shadows())
    }
}
