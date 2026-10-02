//! Configuration and argument storage shared by General Fuse and derived operations.
use super::algo::BRepAlgoAPIAlgo;
use crate::bop_algo::Glue;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::List;
use std::ops::{Deref, DerefMut};

/// The public data and option surface of `BRepAlgoAPI_BuilderAlgo`.
/// Geometry building awaits the port of `BOPAlgo_PaveFiller` and `BOPAlgo_Builder`.
pub struct BRepAlgoAPIBuilderAlgo {
    algo: BRepAlgoAPIAlgo,
    arguments: List<Shape>,
    non_destructive: bool,
    glue: Glue,
    check_inverted: bool,
    fill_history: bool,
}

impl BRepAlgoAPIBuilderAlgo {
    /// Creates an empty General Fuse builder with OCCT defaults.
    /// Example: `let builder = BRepAlgoAPIBuilderAlgo::new(); assert!(builder.arguments().is_empty());`.
    pub fn new() -> Self {
        Self {
            algo: BRepAlgoAPIAlgo::new(),
            arguments: List::default(),
            non_destructive: false,
            glue: Glue::Off,
            check_inverted: true,
            fill_history: true,
        }
    }

    /// Replaces the arguments by a copy of `arguments`; `builder.set_arguments(&list)`.
    pub fn set_arguments(&mut self, arguments: &List<Shape>) {
        self.arguments = arguments.clone();
    }

    /// Returns the configured argument list; `builder.arguments().len()`.
    pub fn arguments(&self) -> &List<Shape> {
        &self.arguments
    }

    /// Sets whether argument shapes must be kept unchanged; `builder.set_non_destructive(true)`.
    pub fn set_non_destructive(&mut self, enabled: bool) {
        self.non_destructive = enabled;
    }

    /// Reads the non-destructive flag; `builder.non_destructive()`.
    pub fn non_destructive(&self) -> bool {
        self.non_destructive
    }

    /// Selects the coincident-shape assumption; `builder.set_glue(Glue::Full)`.
    pub fn set_glue(&mut self, glue: Glue) {
        self.glue = glue;
    }

    /// Returns the selected gluing mode; `builder.glue()`.
    pub fn glue(&self) -> Glue {
        self.glue
    }

    /// Sets validation of inverted input solids; `builder.set_check_inverted(false)`.
    pub fn set_check_inverted(&mut self, enabled: bool) {
        self.check_inverted = enabled;
    }

    /// Reads validation of inverted input solids; `builder.check_inverted()`.
    pub fn check_inverted(&self) -> bool {
        self.check_inverted
    }

    /// Enables or disables modification history; `builder.set_to_fill_history(false)`.
    pub fn set_to_fill_history(&mut self, enabled: bool) {
        self.fill_history = enabled;
    }

    /// Returns whether history will be collected on a future build; `builder.has_history()`.
    pub fn has_history(&self) -> bool {
        self.fill_history
    }
}

impl Default for BRepAlgoAPIBuilderAlgo {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for BRepAlgoAPIBuilderAlgo {
    type Target = BRepAlgoAPIAlgo;

    fn deref(&self) -> &Self::Target {
        &self.algo
    }
}

impl DerefMut for BRepAlgoAPIBuilderAlgo {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.algo
    }
}
