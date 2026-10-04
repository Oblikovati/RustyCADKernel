//! Argument and result state inherited by General Fuse and Boolean builders.
use super::{Glue, Options};
use rustycad_tkbrep::{topods::Shape, toptools::MapOfShape};
use rustycad_tkernel::ncollection::List;

/// The source-pinned argument and option state of `BOPAlgo_Builder`.
/// Boolean execution is supplied by `BOPAlgo_PaveFiller` and is not part of this state slice.
///
/// ```
/// use rustycad_tkbo::bop_algo::BOPAlgoBuilder;
/// let builder = BOPAlgoBuilder::new();
/// assert!(builder.arguments().is_empty());
/// ```
pub struct BOPAlgoBuilder {
    options: Options,
    shape: Shape,
    arguments: List<Shape>,
    argument_fence: MapOfShape,
    non_destructive: bool,
    glue: Glue,
    check_inverted: bool,
    fill_history: bool,
}

impl Default for BOPAlgoBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl BOPAlgoBuilder {
    /// Creates builder state with OCCT defaults; `BOPAlgoBuilder::new()`.
    pub fn new() -> Self {
        Self {
            options: Options::new(),
            shape: Shape::default(),
            arguments: List::default(),
            argument_fence: MapOfShape::default(),
            non_destructive: false,
            glue: Glue::Off,
            check_inverted: true,
            fill_history: true,
        }
    }

    /// Adds a unique shape identity; reversed occurrences are duplicates; `builder.add_argument(shape)`.
    pub fn add_argument(&mut self, shape: &Shape) {
        if self.argument_fence.add(shape.clone()) {
            self.arguments.append(shape.clone());
        }
    }

    /// Replaces listed arguments while preserving the native fence until `clear`; `builder.set_arguments(&shapes)`.
    pub fn set_arguments(&mut self, shapes: &List<Shape>) {
        self.arguments.clear();
        for shape in shapes.iter() {
            self.add_argument(shape);
        }
    }

    /// Borrows the unique argument list; `builder.arguments().len()`.
    pub fn arguments(&self) -> &List<Shape> {
        &self.arguments
    }

    /// Clears report, result, argument and shape maps while retaining configuration; `builder.clear()`.
    pub fn clear(&mut self) {
        self.options.clear();
        self.shape = Shape::default();
        self.arguments.clear();
        self.argument_fence.clear(false);
    }

    /// Returns the current result shape, null before an execution algorithm is connected.
    /// Example: `assert!(BOPAlgoBuilder::new().shape().is_null())`.
    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    /// Reads non-destructive mode; `builder.non_destructive()`.
    pub fn non_destructive(&self) -> bool {
        self.non_destructive
    }

    /// Sets non-destructive mode; `builder.set_non_destructive(true)`.
    pub fn set_non_destructive(&mut self, enabled: bool) {
        self.non_destructive = enabled;
    }

    /// Reads glue mode; `builder.glue()`.
    pub fn glue(&self) -> Glue {
        self.glue
    }

    /// Sets glue mode; `builder.set_glue(Glue::Shift)`.
    pub fn set_glue(&mut self, glue: Glue) {
        self.glue = glue;
    }

    /// Reads inverted-solid checking mode; `builder.check_inverted()`.
    pub fn check_inverted(&self) -> bool {
        self.check_inverted
    }

    /// Sets inverted-solid checking mode; `builder.set_check_inverted(false)`.
    pub fn set_check_inverted(&mut self, enabled: bool) {
        self.check_inverted = enabled;
    }

    /// Reads history collection mode; `builder.has_history()`.
    pub fn has_history(&self) -> bool {
        self.fill_history
    }

    /// Sets history collection mode; `builder.set_to_fill_history(false)`.
    pub fn set_to_fill_history(&mut self, enabled: bool) {
        self.fill_history = enabled;
    }

    /// Borrows the configured algorithm options; `builder.options().fuzzy_value()`.
    pub fn options(&self) -> &Options {
        &self.options
    }

    /// Mutably borrows algorithm options; `builder.options_mut().set_fuzzy_value(0.1)`.
    pub fn options_mut(&mut self) -> &mut Options {
        &mut self.options
    }
}
