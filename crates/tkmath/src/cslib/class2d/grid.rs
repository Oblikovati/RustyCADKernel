//! The lazily built, thread-safe 32x32 classification cache of CSLib_Class2d.
mod raster;

use super::Class2dResult;
use super::normalize::is_safe_for_direct_arithmetic;
use super::ray::NormalizedPolygon;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};

/// Cells per side of the normalized [0,1]^2 domain.
pub(super) const GRID_SIZE: i32 = 32;
/// Smaller polygons stay on the exact scan.
pub(super) const GRID_MIN_POINTS: usize = 24;
/// Queries needed before the grid is built.
const GRID_BUILD_QUERY_COUNT: usize = 64;

/// Classification of one grid cell (native `GridCell`, a `signed char`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i8)]
pub(super) enum GridCell {
    Outside = -1,
    Boundary = 0,
    Inside = 1,
    Unvisited = 2,
}

/// Native `GridState` values stored in the atomic.
const NOT_BUILT: u8 = 0;
const BUILDING: u8 = 1;
const READY: u8 = 2;
const DISABLED: u8 = 3;

/// Grid, state and query counter.
///
/// The cells are published through a `OnceLock` before the state becomes Ready
/// (release), so readers that acquire Ready always find them; OCCT relies on the
/// same release/acquire pair around a plain array.
#[derive(Debug, Default)]
pub(super) struct GridCache {
    cells: OnceLock<Box<[GridCell]>>,
    state: AtomicU8,
    query_count: AtomicUsize,
}

impl GridCache {
    /// Copy-constructor semantics: a Ready grid and Disabled state are copied; a
    /// grid under construction is not, and its query count restarts at zero.
    pub(super) fn copied(&self) -> Self {
        let state: u8 = self.state.load(Ordering::Acquire);
        let copy: Self = Self::with_query_count(self.transferable_count(state));
        match state {
            READY => copy.publish_cells(self.cells.get().cloned()),
            DISABLED => copy.state.store(DISABLED, Ordering::Relaxed),
            _ => {}
        }
        copy
    }

    /// Move semantics: like [`Self::copied`] but steals the grid and disables `self`.
    pub(super) fn taken(&mut self) -> Self {
        let state: u8 = self.state.load(Ordering::Acquire);
        let moved: Self = Self::with_query_count(self.transferable_count(state));
        match state {
            READY => moved.publish_cells(self.cells.take()),
            DISABLED => moved.state.store(DISABLED, Ordering::Relaxed),
            _ => {}
        }
        self.state.store(DISABLED, Ordering::Release);
        moved
    }

    fn with_query_count(count: usize) -> Self {
        Self {
            cells: OnceLock::new(),
            state: AtomicU8::new(NOT_BUILT),
            query_count: AtomicUsize::new(count),
        }
    }

    fn transferable_count(&self, state: u8) -> usize {
        if state == BUILDING {
            return 0;
        }
        self.query_count.load(Ordering::Relaxed)
    }

    fn publish_cells(&self, cells: Option<Box<[GridCell]>>) {
        let Some(cells) = cells else {
            return;
        };
        // A fresh or builder-owned lock: nothing else can have set it.
        let _ = self.cells.set(cells);
        self.state.store(READY, Ordering::Release);
    }

    /// Counts a query, builds the grid at the threshold and returns the cached
    /// Inside/Outside answer for a boundary-free cell, if any.
    pub(super) fn lookup(
        &self,
        polygon: &NormalizedPolygon,
        tol: [f64; 2],
        x: f64,
        y: f64,
    ) -> Option<Class2dResult> {
        let mut state: u8 = self.state.load(Ordering::Acquire);
        if state == NOT_BUILT && polygon.count() >= GRID_MIN_POINTS {
            let queries: usize = self
                .query_count
                .fetch_add(1, Ordering::Relaxed)
                .wrapping_add(1);
            if queries >= GRID_BUILD_QUERY_COUNT {
                self.build(polygon, tol);
                state = self.state.load(Ordering::Acquire);
            }
        }
        if state != READY || !is_in_unit_square(x, y) {
            return None;
        }
        self.cached(x, y)
    }

    fn cached(&self, x: f64, y: f64) -> Option<Class2dResult> {
        let cells: &[GridCell] = self.cells.get()?;
        let ix: i32 = ((x * f64::from(GRID_SIZE)) as i32).clamp(0, GRID_SIZE - 1);
        let iy: i32 = ((y * f64::from(GRID_SIZE)) as i32).clamp(0, GRID_SIZE - 1);
        match cells[(iy * GRID_SIZE + ix) as usize] {
            GridCell::Inside => Some(Class2dResult::Inside),
            GridCell::Outside => Some(Class2dResult::Outside),
            // Boundary cells fall through to exact classification.
            GridCell::Boundary | GridCell::Unvisited => None,
        }
    }

    // buildGridCache: exactly one thread wins NotBuilt -> Building; the others keep
    // using the exact path until the state is published.
    fn build(&self, polygon: &NormalizedPolygon, tol: [f64; 2]) {
        if polygon.count() < GRID_MIN_POINTS {
            return;
        }
        let claimed =
            self.state
                .compare_exchange(NOT_BUILT, BUILDING, Ordering::AcqRel, Ordering::Acquire);
        if claimed.is_err() {
            return;
        }
        match raster::classify_cells(polygon, tol) {
            Err(raster::AllocationFailure) => {
                // Transient allocation failure: allow a later rebuild, as natively.
                self.query_count.store(0, Ordering::Relaxed);
                self.state.store(NOT_BUILT, Ordering::Release);
            }
            Ok(None) => self.state.store(DISABLED, Ordering::Release),
            Ok(Some(cells)) => self.publish_cells(Some(cells)),
        }
    }
}

fn is_in_unit_square(x: f64, y: f64) -> bool {
    is_safe_for_direct_arithmetic(x)
        && is_safe_for_direct_arithmetic(y)
        && (0.0..=1.0).contains(&x)
        && (0.0..=1.0).contains(&y)
}
