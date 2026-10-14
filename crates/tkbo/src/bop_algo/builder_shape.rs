//! `BOPAlgo_BuilderShape`: the result shape and the history of the shape builders.
use super::Options;
use rustycad_tkbrep::brep_tools::History;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::Map;
use rustycad_tkernel::standard::{Failure, Handle};

/// The result of an algorithm building a shape, with the history of its arguments.
///
/// ```
/// use rustycad_tkbo::bop_algo::BuilderShape;
/// let mut base = BuilderShape::new();
/// assert!(base.shape().is_null() && base.has_history());
/// assert!(base.history().is_some());
/// ```
pub struct BuilderShape {
    pub(super) options: Options,
    pub(super) shape: Shape,
    /// `myMapShape`: every sub-shape of the result (the cached map of the history).
    pub(super) map_shape: Map<Shape, ShapeMapHasher>,
    pub(super) fill_history: bool,
    pub(super) history: Handle<History>,
}

impl Default for BuilderShape {
    fn default() -> Self {
        Self::new()
    }
}

impl BuilderShape {
    /// A null result with history filling on (`BOPAlgo_BuilderShape()`); `BuilderShape::new()`.
    pub fn new() -> Self {
        Self {
            options: Options::new(),
            shape: Shape::default(),
            map_shape: Map::with_hasher(ShapeMapHasher, 1),
            fill_history: true,
            history: Handle::default(),
        }
    }

    /// The result of the operation (`Shape`); `base.shape()`.
    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    /// The splits of `shape` kept in the result (`Modified`); `base.modified(&edge)?`.
    pub fn modified(&self, shape: &Shape) -> Result<Vec<Shape>, Failure> {
        match self.filled_history() {
            Some(history) => history.modified(shape),
            None => Ok(Vec::new()),
        }
    }

    /// The shapes generated from `shape` kept in the result (`Generated`).
    /// `base.generated(&face)?`.
    pub fn generated(&self, shape: &Shape) -> Result<Vec<Shape>, Failure> {
        match self.filled_history() {
            Some(history) => history.generated(shape),
            None => Ok(Vec::new()),
        }
    }

    /// Whether `shape` has no trace in the result (`IsDeleted`); `base.is_deleted(&face)?`.
    pub fn is_deleted(&self, shape: &Shape) -> Result<bool, Failure> {
        match self.filled_history() {
            Some(history) => history.is_removed(shape),
            None => Ok(false),
        }
    }

    /// Whether some argument shape was modified (`HasModified`); `base.has_modified()`.
    pub fn has_modified(&self) -> bool {
        self.filled_history().is_some_and(History::has_modified)
    }

    /// Whether some argument shape generated new shapes (`HasGenerated`).
    /// `base.has_generated()`.
    pub fn has_generated(&self) -> bool {
        self.filled_history().is_some_and(History::has_generated)
    }

    /// Whether some argument shape was deleted (`HasDeleted`); `base.has_deleted()`.
    pub fn has_deleted(&self) -> bool {
        self.filled_history().is_some_and(History::has_removed)
    }

    /// The history, created empty on demand while history filling is on; `None` when it is
    /// off (`History`); `base.history()`.
    pub fn history(&mut self) -> Option<Handle<History>> {
        if !self.fill_history {
            return None;
        }
        if self.history.is_null() {
            self.history = Handle::new(History::new());
        }
        Some(self.history.clone())
    }

    /// Turns history filling on or off (`SetToFillHistory`); `base.set_to_fill_history(false)`.
    pub fn set_to_fill_history(&mut self, fill: bool) {
        self.fill_history = fill;
    }

    /// Whether history filling is on (`HasHistory`); `base.has_history()`.
    pub fn has_history(&self) -> bool {
        self.fill_history
    }

    /// The report and run options; `base.options().has_errors()`.
    pub fn options(&self) -> &Options {
        &self.options
    }

    /// Mutable report and run options; `base.options_mut().set_fuzzy_value(1e-5)`.
    pub fn options_mut(&mut self) -> &mut Options {
        &mut self.options
    }

    /// Clears the report, the history and the result map (`Clear`); `base.clear()`.
    pub fn clear(&mut self) {
        self.options.clear();
        self.history = Handle::default();
        self.map_shape.clear(false);
    }

    /// Observe availability without creating an empty history; `base.has_filled_history()`.
    pub(crate) fn has_filled_history(&self) -> bool {
        self.filled_history().is_some()
    }

    fn filled_history(&self) -> Option<&History> {
        if self.fill_history {
            self.history.get()
        } else {
            None
        }
    }
}

/// The queries a `BRepTools_History` is built from (the `TheAlgo` of its template
/// constructor and `Merge`), for `BOPAlgo_Builder` and the algorithms built on it.
impl rustycad_tkbrep::brep_tools::HistoryAlgorithm for BuilderShape {
    fn is_deleted(&mut self, shape: &Shape) -> Result<bool, Failure> {
        BuilderShape::is_deleted(self, shape)
    }

    fn modified(&mut self, shape: &Shape) -> Result<Vec<Shape>, Failure> {
        BuilderShape::modified(self, shape)
    }

    fn generated(&mut self, shape: &Shape) -> Result<Vec<Shape>, Failure> {
        BuilderShape::generated(self, shape)
    }
}
