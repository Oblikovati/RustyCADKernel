//! Live indexed-map iteration at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{Graphic3dCLight, Graphic3dLightSet, Graphic3dLightSetIterationFilter, LightSetState};
use rustycad_tkernel::standard::exceptions::no_such_object;
use rustycad_tkernel::standard::{Failure, Handle, SharedState};
use std::sync::Arc;

/// A live cursor; appends and swap-last removals are visible at its current index.
/// Filters run only on construction and `next`, as in the native iterator.
/// Martensite retains the storage so a cursor cannot outlive borrowed native memory.
/// Example: `let cursor = Graphic3dLightSetIterator::new(&set, filter);`.
#[derive(Clone, Default)]
pub struct Graphic3dLightSetIterator {
    source: Option<Arc<SharedState<LightSetState>>>,
    index: usize,
    filter: i32,
}

impl Graphic3dLightSetIterator {
    /// Starts at the first accepted indexed light; `Graphic3dLightSetIterator::new(&set, filter)`.
    pub fn new(set: &Graphic3dLightSet, filter: Graphic3dLightSetIterationFilter) -> Self {
        let mut cursor = Self {
            source: Some(Arc::clone(&set.state)),
            index: 1,
            filter: filter as i32,
        };
        cursor.skip_filtered();
        cursor
    }

    /// A null handle creates an empty cursor; `Graphic3dLightSetIterator::from_handle(&set, filter)`.
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

    /// Tests the current map extent without rechecking the filter; `cursor.more()`.
    pub fn more(&self) -> bool {
        self.source
            .as_ref()
            .is_some_and(|source| self.index <= source.read().lights.len())
    }

    /// Reads the live key or raises `Standard_NoSuchObject`; `cursor.value()?`.
    pub fn value(&self) -> Result<Handle<Graphic3dCLight>, Failure> {
        let Some(source) = &self.source else {
            return Err(self.exhausted(0));
        };
        let state = source.read();
        self.index
            .checked_sub(1)
            .and_then(|slot| state.lights.get(slot))
            .map(|(light, _)| light.clone())
            .ok_or_else(|| self.exhausted(state.lights.len()))
    }

    /// Advances once then skips currently excluded lights; `cursor.next()`.
    pub fn next(&mut self) {
        self.index = self.index.wrapping_add(1);
        self.skip_filtered();
    }

    /// Rechecks the current filter, exposing the native protected subclass hook.
    /// Normal reads leave a changed current light untouched until `next`.
    /// Example: `cursor.skip_filtered();`.
    pub fn skip_filtered(&mut self) {
        if self.filter == 0 {
            return;
        }
        while self.value().is_ok_and(|light| self.excludes(&light)) {
            self.index = self.index.wrapping_add(1);
        }
    }

    fn excludes(&self, light: &Graphic3dCLight) -> bool {
        use Graphic3dLightSetIterationFilter as F;
        let has = |flag: F| self.filter & flag as i32 != 0;
        (has(F::ExcludeAmbient) && light.light_type() == super::Graphic3dTypeOfLightSource::Ambient)
            || (has(F::ExcludeDisabled) && !light.is_enabled())
            || (has(F::ExcludeNoShadow) && !light.to_cast_shadows())
    }

    fn exhausted(&self, extent: usize) -> Failure {
        no_such_object(&format!(
            "NCollection_IndexedDataMap::Iterator::Key: index {}; expected 1..={extent}",
            self.index
        ))
    }
}
