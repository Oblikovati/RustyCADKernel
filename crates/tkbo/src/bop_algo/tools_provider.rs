//! Argument/tool lists shared by BOPAlgo tools-based algorithms.
use super::BOPAlgoBuilder;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{impl_transient, ncollection::List};
use std::ops::{Deref, DerefMut};

/// The source-pinned tool storage state of `BOPAlgo_ToolsProvider`.
///
/// ```
/// use rustycad_tkbo::bop_algo::BOPAlgoToolsProvider;
/// assert!(BOPAlgoToolsProvider::new().tools().is_empty());
/// ```
pub struct BOPAlgoToolsProvider {
    builder: BOPAlgoBuilder,
    tools: List<Shape>,
    tool_fence: rustycad_tkbrep::toptools::MapOfShape,
}

impl_transient!(
    BOPAlgoToolsProvider,
    "BOPAlgo_ToolsProvider",
    BOPAlgoBuilder,
    builder
);

impl Default for BOPAlgoToolsProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl BOPAlgoToolsProvider {
    /// Creates a tool provider with empty input state; `BOPAlgoToolsProvider::new()`.
    pub fn new() -> Self {
        Self {
            builder: BOPAlgoBuilder::new(),
            tools: List::default(),
            tool_fence: rustycad_tkbrep::toptools::MapOfShape::default(),
        }
    }

    /// Adds a unique tool shape identity; `provider.add_tool(shape)`.
    pub fn add_tool(&mut self, shape: &Shape) {
        if self.tool_fence.add(shape.clone()) {
            self.tools.append(shape.clone());
        }
    }

    /// Replaces listed tools while retaining the native fence until `clear`; `provider.set_tools(&tools)`.
    pub fn set_tools(&mut self, shapes: &List<Shape>) {
        self.tools.clear();
        for shape in shapes.iter() {
            self.add_tool(shape);
        }
    }

    /// Borrows the unique tool list; `provider.tools().len()`.
    pub fn tools(&self) -> &List<Shape> {
        &self.tools
    }

    /// Clears builder and tool maps; `provider.clear()`.
    pub fn clear(&mut self) {
        self.builder.clear();
        self.tools.clear();
        self.tool_fence.clear(false);
    }
}

impl Deref for BOPAlgoToolsProvider {
    type Target = BOPAlgoBuilder;

    fn deref(&self) -> &Self::Target {
        &self.builder
    }
}

impl DerefMut for BOPAlgoToolsProvider {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.builder
    }
}
