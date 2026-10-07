//! `BOPAlgo_BuilderArea`: the state shared by the face and solid builders.
use super::Options;
use crate::int_tools::Context;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::{IndexedMap, List};
use rustycad_tkernel::standard::Handle;

/// The input shapes, the loops and areas built from them, the shapes to avoid and the
/// internal-shape option of an area builder.
///
/// ```
/// use rustycad_tkbo::bop_algo::BuilderArea;
/// let mut area = BuilderArea::new();
/// area.set_avoid_internal_shapes(true);
/// assert!(area.is_avoid_internal_shapes() && area.areas().is_empty());
/// ```
pub struct BuilderArea {
    pub(super) options: Options,
    pub(super) context: Handle<Context>,
    pub(super) shapes: List<Shape>,
    pub(super) loops: List<Shape>,
    pub(super) loops_internal: List<Shape>,
    pub(super) areas: List<Shape>,
    /// `myShapesToAvoid`: keyed by `TopoDS_Shape::IsEqual` (orientation counts).
    pub(super) shapes_to_avoid: IndexedMap<Shape>,
    pub(super) avoid_internal_shapes: bool,
}

impl Default for BuilderArea {
    fn default() -> Self {
        Self::new()
    }
}

impl BuilderArea {
    /// Empty state with internal shapes kept (`BOPAlgo_BuilderArea()`); `BuilderArea::new()`.
    pub fn new() -> Self {
        Self {
            options: Options::new(),
            context: Handle::default(),
            shapes: List::default(),
            loops: List::default(),
            loops_internal: List::default(),
            areas: List::default(),
            shapes_to_avoid: IndexedMap::new(100),
            avoid_internal_shapes: false,
        }
    }

    /// Shares a geometric context (`SetContext`); `area.set_context(context)`.
    pub fn set_context(&mut self, context: Handle<Context>) {
        self.context = context;
    }

    /// The geometric context; `area.context()`.
    pub fn context(&self) -> &Handle<Context> {
        &self.context
    }

    /// The input shapes (`Shapes`); `area.shapes().len()`.
    pub fn shapes(&self) -> &List<Shape> {
        &self.shapes
    }

    /// Sets the input shapes (`SetShapes`); `area.set_shapes(&edges)`.
    pub fn set_shapes(&mut self, shapes: &List<Shape>) {
        self.shapes = shapes.clone();
    }

    /// The loops built from the shapes (`Loops`); `area.loops().len()`.
    pub fn loops(&self) -> &List<Shape> {
        &self.loops
    }

    /// The built areas (`Areas`); `area.areas().len()`.
    pub fn areas(&self) -> &List<Shape> {
        &self.areas
    }

    /// Whether internal shapes are left out of the areas (`SetAvoidInternalShapes`).
    /// `area.set_avoid_internal_shapes(true)`.
    pub fn set_avoid_internal_shapes(&mut self, avoid: bool) {
        self.avoid_internal_shapes = avoid;
    }

    /// Whether internal shapes are left out (`IsAvoidInternalShapes`).
    /// `area.is_avoid_internal_shapes()`.
    pub fn is_avoid_internal_shapes(&self) -> bool {
        self.avoid_internal_shapes
    }

    /// The report and run options; `area.options().has_errors()`.
    pub fn options(&self) -> &Options {
        &self.options
    }

    /// Mutable report and run options; `area.options_mut().set_run_parallel(true)`.
    pub fn options_mut(&mut self) -> &mut Options {
        &mut self.options
    }
}
