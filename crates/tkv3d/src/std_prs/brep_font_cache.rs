//! `StdPrs_BRepFontCache`: a bounded FIFO cache of fonts by request (name, aspect, size and
//! strict level); each lookup returns a new font handle sharing the cached implementation.
use super::brep_font::StdPrsBRepFont;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Handle, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkservice::font::{FontFontAspect, FontStrictLevel};
use std::collections::HashMap;
use std::sync::RwLock;

/// `StdPrs_BRepFontCache::Request`, compared exactly (`RequestHasher`); the size by its bits
/// apart from the zero sign, as native `==` on positive finite sizes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct StdPrsBRepFontRequestKey {
    font_name: Vec<u8>,
    size_bits: u64,
    font_aspect: i32,
    strict_level: i32,
}

#[derive(Default)]
struct StdPrsBRepFontCacheState {
    // Insertion order; the native IndexedDataMap index is position + 1.
    entries: Vec<(StdPrsBRepFontRequestKey, Handle<StdPrsBRepFont>)>,
    positions: HashMap<StdPrsBRepFontRequestKey, usize>,
    next_index: usize,
}

/// `StdPrs_BRepFontCache`. Example: `let cache = StdPrsBRepFontCache::new(16); cache.find_font(&name, aspect, 12.0, FontStrictLevel::Any)`.
pub struct StdPrsBRepFontCache {
    transient: StandardTransient,
    capacity: usize,
    state: RwLock<StdPrsBRepFontCacheState>,
}
rustycad_tkernel::impl_transient!(
    StdPrsBRepFontCache,
    "StdPrs_BRepFontCache",
    StandardTransient,
    transient
);

impl Default for StdPrsBRepFontCache {
    fn default() -> Self {
        Self::new(16)
    }
}

impl StdPrsBRepFontCache {
    /// `StdPrs_BRepFontCache(theCapacity = 16)`; a zero capacity caches nothing.
    /// Example: `StdPrsBRepFontCache::new(4)`.
    pub fn new(capacity: usize) -> Self {
        let state = StdPrsBRepFontCacheState {
            entries: Vec::with_capacity(capacity),
            positions: HashMap::with_capacity(capacity),
            next_index: 1,
        };
        Self {
            transient: StandardTransient::default(),
            capacity,
            state: RwLock::new(state),
        }
    }

    /// `FindFont`: a new handle sharing the cached font of the request, creating and caching
    /// it on a miss (replacing the oldest entry when full); null for an invalid size, a zero
    /// capacity or an unavailable font.
    /// Example: `cache.find_font(&"Arial".into(), FontFontAspect::Regular, 12.0, FontStrictLevel::Any)`.
    pub fn find_font(
        &self,
        name: &AsciiString,
        aspect: FontFontAspect,
        size: f64,
        strict_level: FontStrictLevel,
    ) -> Handle<StdPrsBRepFont> {
        if self.capacity == 0 || size.is_nan() || size <= 0.0 || precision::is_infinite(size) {
            return Handle::default();
        }
        let key = StdPrsBRepFontRequestKey {
            font_name: name.as_bytes().to_vec(),
            size_bits: size.to_bits(),
            font_aspect: aspect as i32,
            strict_level: strict_level as i32,
        };
        if let Some(found) = self.lookup(&key) {
            return found;
        }
        let mut state = self
            .state
            .write()
            .unwrap_or_else(|poison| poison.into_inner());
        if let Some(&position) = state.positions.get(&key) {
            return shared_copy(&state.entries[position].1);
        }
        let created = StdPrsBRepFont::find_and_create(name, aspect, size, strict_level);
        if created.is_null() {
            return Handle::default();
        }
        self.insert(&mut state, key, created.clone());
        shared_copy(&created)
    }

    fn lookup(&self, key: &StdPrsBRepFontRequestKey) -> Option<Handle<StdPrsBRepFont>> {
        let state = self
            .state
            .read()
            .unwrap_or_else(|poison| poison.into_inner());
        let position = *state.positions.get(key)?;
        Some(shared_copy(&state.entries[position].1))
    }

    fn insert(
        &self,
        state: &mut StdPrsBRepFontCacheState,
        key: StdPrsBRepFontRequestKey,
        font: Handle<StdPrsBRepFont>,
    ) {
        if state.entries.len() < self.capacity {
            state.positions.insert(key.clone(), state.entries.len());
            state.entries.push((key, font));
            return;
        }
        let position = state.next_index - 1;
        let old_key = state.entries[position].0.clone();
        state.positions.remove(&old_key);
        state.positions.insert(key.clone(), position);
        state.entries[position] = (key, font);
        state.next_index = if state.next_index < self.capacity {
            state.next_index + 1
        } else {
            1
        };
    }

    /// `Clear`; fonts handed out stay valid. Example: `cache.clear()`.
    pub fn clear(&self) {
        let mut state = self
            .state
            .write()
            .unwrap_or_else(|poison| poison.into_inner());
        state.entries.clear();
        state.positions.clear();
        state.next_index = 1;
    }

    /// `Capacity`; `cache.capacity()`.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// `Size`: the cached requests; `cache.size()`.
    pub fn size(&self) -> usize {
        self.state
            .read()
            .unwrap_or_else(|poison| poison.into_inner())
            .entries
            .len()
    }
}

/// `new StdPrs_BRepFont(anEntry->Font->myImpl)`.
fn shared_copy(font: &Handle<StdPrsBRepFont>) -> Handle<StdPrsBRepFont> {
    let implementation = font
        .get()
        .expect("cached fonts are never null")
        .implementation();
    Handle::new(StdPrsBRepFont::from_impl(implementation))
}
