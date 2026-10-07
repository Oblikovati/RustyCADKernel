//! The `BOPAlgo_Builder*` the BRepAlgoAPI classes drive: any General Fuse algorithm run on
//! the results of a pave filler.
use super::{BOPAlgoBOP, BOPAlgoBuilder, BOPAlgoPaveFiller, BOPAlgoSection};
use std::ops::{Deref, DerefMut};

/// A General Fuse algorithm (the builder, the Boolean operation, the section, the splitter)
/// seen through its `BOPAlgo_Builder` base.
///
/// ```
/// use rustycad_tkbo::bop_algo::{BOPAlgoBuilder, GeneralFuseAlgorithm};
/// let builder = BOPAlgoBuilder::new();
/// assert!(builder.general_fuse().arguments().is_empty());
/// ```
pub trait GeneralFuseAlgorithm {
    /// The `BOPAlgo_Builder` base; `algorithm.general_fuse().images()`.
    fn general_fuse(&self) -> &BOPAlgoBuilder;
    /// The mutable `BOPAlgo_Builder` base; `algorithm.general_fuse_mut().clear()`.
    fn general_fuse_mut(&mut self) -> &mut BOPAlgoBuilder;
    /// Builds the result from the results of `filler` (`PerformWithFiller`).
    fn perform_with_filler(&mut self, filler: &mut BOPAlgoPaveFiller);
}

impl GeneralFuseAlgorithm for BOPAlgoBuilder {
    fn general_fuse(&self) -> &BOPAlgoBuilder {
        self
    }
    fn general_fuse_mut(&mut self) -> &mut BOPAlgoBuilder {
        self
    }
    fn perform_with_filler(&mut self, filler: &mut BOPAlgoPaveFiller) {
        BOPAlgoBuilder::perform_with_filler(self, filler);
    }
}

impl GeneralFuseAlgorithm for BOPAlgoBOP {
    fn general_fuse(&self) -> &BOPAlgoBuilder {
        self.deref().deref()
    }
    fn general_fuse_mut(&mut self) -> &mut BOPAlgoBuilder {
        self.deref_mut().deref_mut()
    }
    fn perform_with_filler(&mut self, filler: &mut BOPAlgoPaveFiller) {
        BOPAlgoBOP::perform_with_filler(self, filler);
    }
}

impl GeneralFuseAlgorithm for BOPAlgoSection {
    fn general_fuse(&self) -> &BOPAlgoBuilder {
        self.deref()
    }
    fn general_fuse_mut(&mut self) -> &mut BOPAlgoBuilder {
        self.deref_mut()
    }
    fn perform_with_filler(&mut self, filler: &mut BOPAlgoPaveFiller) {
        BOPAlgoSection::perform_with_filler(self, filler);
    }
}
