//! `StdPrs_BRepFont::Impl::GlyphCache`: a bounded FIFO map from resolved glyphs to their
//! shared planar regions, with the reverse map from region identity to glyph.
use crate::brep_font::BRepFontPlanarRegion;
use rustycad_tkservice::font::{FontGlyphOutlineGlyph, FontGlyphOutlineGlyphHasher};
use std::collections::HashMap;
use std::sync::Arc;

/// `THE_CAPACITY`: the number of cached glyph regions.
const CACHE_CAPACITY: usize = 256;
/// `THE_MAX_REGION_COMPLEXITY`: the largest pool total (and pool size) worth caching.
const MAX_REGION_COMPLEXITY: usize = 16384;

/// A glyph as a hash map key, hashed and compared by `Font_GlyphOutline::Glyph::Hasher`.
#[derive(Clone, Debug, PartialEq, Eq)]
struct GlyphKey(FontGlyphOutlineGlyph);

impl std::hash::Hash for GlyphKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(FontGlyphOutlineGlyphHasher.hash(&self.0));
    }
}

/// The identity of a shared region (the native `const BRepFont_PlanarRegion*` key).
pub(super) fn region_address(region: &Arc<BRepFontPlanarRegion>) -> usize {
    Arc::as_ptr(region) as usize
}

/// The glyph region cache of one font implementation.
/// Example: `cache.bind_region(&glyph, &region); cache.find_region(&glyph)`.
pub(super) struct StdPrsBRepFontGlyphCache {
    // The entries in insertion order; the native IndexedDataMap index is position + 1.
    entries: Vec<(FontGlyphOutlineGlyph, Arc<BRepFontPlanarRegion>)>,
    positions: HashMap<GlyphKey, usize>,
    glyphs_by_region: HashMap<usize, FontGlyphOutlineGlyph>,
    // `myNextIndex`: the one-based index the next binding replaces once full.
    next_index: usize,
}

impl Default for StdPrsBRepFontGlyphCache {
    fn default() -> Self {
        Self {
            entries: Vec::with_capacity(CACHE_CAPACITY),
            positions: HashMap::with_capacity(CACHE_CAPACITY),
            glyphs_by_region: HashMap::new(),
            next_index: 1,
        }
    }
}

impl StdPrsBRepFontGlyphCache {
    /// `FindRegion`; `cache.find_region(&glyph)`.
    pub(super) fn find_region(
        &self,
        glyph: &FontGlyphOutlineGlyph,
    ) -> Option<Arc<BRepFontPlanarRegion>> {
        let position = *self.positions.get(&GlyphKey(glyph.clone()))?;
        Some(self.entries[position].1.clone())
    }

    /// `FindGlyph`: the glyph a cached region was loaded for; `cache.find_glyph(&region)`.
    pub(super) fn find_glyph(
        &self,
        region: &Arc<BRepFontPlanarRegion>,
    ) -> Option<FontGlyphOutlineGlyph> {
        self.glyphs_by_region.get(&region_address(region)).cloned()
    }

    /// `BindRegion`: caches a new, small enough region; once full, the oldest binding is
    /// replaced in FIFO order. Example: `cache.bind_region(&glyph, &region)`.
    pub(super) fn bind_region(
        &mut self,
        glyph: &FontGlyphOutlineGlyph,
        region: &Arc<BRepFontPlanarRegion>,
    ) {
        let key = GlyphKey(glyph.clone());
        if self.positions.contains_key(&key) || !is_cacheable(region) {
            return;
        }
        if self.entries.len() < CACHE_CAPACITY {
            self.positions.insert(key, self.entries.len());
            self.entries.push((glyph.clone(), region.clone()));
        } else {
            self.substitute(key, region);
        }
        self.glyphs_by_region
            .insert(region_address(region), glyph.clone());
    }

    fn substitute(&mut self, key: GlyphKey, region: &Arc<BRepFontPlanarRegion>) {
        let position = self.next_index - 1;
        let (old_glyph, old_region) = &self.entries[position];
        self.glyphs_by_region.remove(&region_address(old_region));
        self.positions.remove(&GlyphKey(old_glyph.clone()));
        self.entries[position] = (key.0.clone(), region.clone());
        self.positions.insert(key, position);
        self.next_index = if self.next_index < CACHE_CAPACITY {
            self.next_index + 1
        } else {
            1
        };
    }

    /// `Clear`; `cache.clear()`.
    pub(super) fn clear(&mut self) {
        self.entries.clear();
        self.positions.clear();
        self.glyphs_by_region.clear();
        self.next_index = 1;
    }
}

/// `isCacheable`: no pool and no pool total exceeds the complexity limit.
fn is_cacheable(region: &BRepFontPlanarRegion) -> bool {
    let sizes = [
        region.vertices().len(),
        region.curves().len(),
        region.uses().len(),
        region.loops().len(),
        region.hole_loops().len(),
        region.regions().len(),
    ];
    sizes.iter().all(|&size| size <= MAX_REGION_COMPLEXITY)
        && sizes.iter().sum::<usize>() <= MAX_REGION_COMPLEXITY
}
