//! `BRepAlgoAPI_Splitter`: the API of the splitter of arguments by tools.
use super::builder_algo::BRepAlgoAPIBuilderAlgo;
use crate::bop_algo::alerts::alert;
use crate::bop_algo::{AlertTooFewArguments, BOPAlgoPaveFiller, BOPAlgoSplitter};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use std::ops::{Deref, DerefMut};

/// Splits the arguments by the tools.
///
/// ```
/// use rustycad_tkbo::brep_algo_api::BRepAlgoAPISplitter;
/// let splitter = BRepAlgoAPISplitter::new();
/// assert!(splitter.tools().is_empty());
/// ```
#[derive(Default)]
pub struct BRepAlgoAPISplitter {
    builder_algo: BRepAlgoAPIBuilderAlgo,
    tools: List<Shape>,
}

impl Deref for BRepAlgoAPISplitter {
    type Target = BRepAlgoAPIBuilderAlgo;
    fn deref(&self) -> &BRepAlgoAPIBuilderAlgo {
        &self.builder_algo
    }
}

impl DerefMut for BRepAlgoAPISplitter {
    fn deref_mut(&mut self) -> &mut BRepAlgoAPIBuilderAlgo {
        &mut self.builder_algo
    }
}

impl BRepAlgoAPISplitter {
    /// An empty splitter (`BRepAlgoAPI_Splitter()`); `BRepAlgoAPISplitter::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// A splitter on the results of a performed filler (`BRepAlgoAPI_Splitter(PF)`).
    /// `BRepAlgoAPISplitter::with_filler(filler)`.
    pub fn with_filler(filler: BOPAlgoPaveFiller) -> Self {
        Self {
            builder_algo: BRepAlgoAPIBuilderAlgo::with_filler(filler),
            tools: List::default(),
        }
    }

    /// Replaces the tools by a copy of `tools` (`SetTools`); `splitter.set_tools(&tools)`.
    pub fn set_tools(&mut self, tools: &List<Shape>) {
        self.tools = tools.clone();
    }

    /// The tools (`Tools`); `splitter.tools().len()`.
    pub fn tools(&self) -> &List<Shape> {
        &self.tools
    }

    /// Splits the arguments by the tools (`Build`); `splitter.build()?`.
    pub fn build(&mut self) -> Result<(), Failure> {
        self.builder_algo.base.set_not_done();
        self.builder_algo.clear();
        let arguments = self.builder_algo.arguments.clone();
        if arguments.is_empty() || arguments.len() + self.tools.len() < 2 {
            self.builder_algo
                .options
                .add_error(&alert(AlertTooFewArguments));
            return Ok(());
        }
        if self.builder_algo.is_intersection_needed {
            let mut shapes = arguments.clone();
            for tool in self.tools.iter() {
                shapes.append(tool.clone());
            }
            self.builder_algo.intersect_shapes(&shapes)?;
            if self.has_errors() {
                return Ok(());
            }
        }
        let mut splitter = BOPAlgoSplitter::new();
        splitter.set_arguments(&arguments);
        splitter.set_tools(&self.tools);
        self.builder_algo.builder = Some(Box::new(splitter));
        self.builder_algo.build_result()
    }
}
