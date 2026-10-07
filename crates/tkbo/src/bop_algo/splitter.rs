//! `BOPAlgo_Splitter`: splits the arguments by the tools; the splits of the tools stay out
//! of the result.
use super::alerts::alert;
use super::builder::perform::{perform_on, perform_with};
use super::{
    AlertTooFewArguments, BOPAlgoBuilder, BOPAlgoPaveFiller, BOPAlgoToolsProvider, BuilderHooks,
    GeneralFuseAlgorithm,
};
use crate::bop_ds::DataStructure;
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::ops::{Deref, DerefMut};

/// The splitter: the General Fuse of the arguments and the tools keeping only the splits of
/// the arguments.
///
/// ```
/// use rustycad_tkbo::bop_algo::BOPAlgoSplitter;
/// let splitter = BOPAlgoSplitter::new();
/// assert!(splitter.tools().is_empty());
/// ```
#[derive(Default)]
pub struct BOPAlgoSplitter {
    provider: BOPAlgoToolsProvider,
}

impl Deref for BOPAlgoSplitter {
    type Target = BOPAlgoToolsProvider;
    fn deref(&self) -> &BOPAlgoToolsProvider {
        &self.provider
    }
}

impl DerefMut for BOPAlgoSplitter {
    fn deref_mut(&mut self) -> &mut BOPAlgoToolsProvider {
        &mut self.provider
    }
}

impl BOPAlgoSplitter {
    /// An empty splitter (`BOPAlgo_Splitter()`); `BOPAlgoSplitter::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Intersects the arguments and the tools with an own pave filler, then splits the
    /// arguments (`Perform`); `splitter.perform();`.
    pub fn perform(&mut self) {
        let mut shapes: List<Shape> = self.provider.builder.arguments.clone();
        for tool in self.provider.tools().iter() {
            shapes.append(tool.clone());
        }
        perform_on(self, shapes);
    }

    /// Splits the arguments with the results of `filler` (`PerformWithFiller`).
    /// `splitter.perform_with_filler(&mut filler);`.
    pub fn perform_with_filler(&mut self, filler: &mut BOPAlgoPaveFiller) {
        perform_with(self, filler);
    }
}

impl BuilderHooks for BOPAlgoSplitter {
    fn general_fuse(&mut self) -> &mut BOPAlgoBuilder {
        &mut self.provider.builder
    }

    /// At least one argument and two shapes in all, then the filler report merged
    /// (`CheckData`).
    fn check_data(&mut self, filler: &BOPAlgoPaveFiller) {
        let tools = self.provider.tools().len();
        let builder = &mut self.provider.builder;
        if builder.arguments.is_empty() || builder.arguments.len() + tools < 2 {
            builder.options().add_error(&alert(AlertTooFewArguments));
            return;
        }
        builder.check_filler(filler);
    }

    /// The images of the arguments; after the compounds, a single argument kept as one
    /// shape is the result itself, not a compound of it (`BuildResult`).
    fn build_result(&mut self, kind: ShapeEnum, _structure: &DataStructure) -> Result<(), Failure> {
        let builder = &mut self.provider.builder;
        builder.build_result_default(kind)?;
        if kind != ShapeEnum::Compound || builder.arguments.len() != 1 {
            return Ok(());
        }
        let mut parts = ShapeIterator::new(builder.shape(), true, true);
        if let (Some(first), None) = (Iterator::next(&mut parts), Iterator::next(&mut parts)) {
            builder.shape = first;
        }
        Ok(())
    }
}

impl GeneralFuseAlgorithm for BOPAlgoSplitter {
    fn general_fuse(&self) -> &BOPAlgoBuilder {
        &self.provider.builder
    }
    fn general_fuse_mut(&mut self) -> &mut BOPAlgoBuilder {
        &mut self.provider.builder
    }
    fn perform_with_filler(&mut self, filler: &mut BOPAlgoPaveFiller) {
        BOPAlgoSplitter::perform_with_filler(self, filler);
    }
}
