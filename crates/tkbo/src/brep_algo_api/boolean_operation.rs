//! `BRepAlgoAPI_BooleanOperation`: the API of the Boolean operations and the section.
use super::builder_algo::BRepAlgoAPIBuilderAlgo;
use crate::bop_algo::alerts::alert;
use crate::bop_algo::{
    AlertBOPNotSet, AlertTooFewArguments, BOPAlgoBOP, BOPAlgoPaveFiller, BOPAlgoSection,
    GeneralFuseAlgorithm, Operation,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use std::ops::{Deref, DerefMut};

/// A Boolean operation (or section) of the arguments (objects) and the tools.
///
/// ```
/// use rustycad_tkbo::bop_algo::Operation;
/// use rustycad_tkbo::brep_algo_api::BRepAlgoAPIBooleanOperation;
/// let mut bop = BRepAlgoAPIBooleanOperation::new();
/// bop.set_operation(Operation::Fuse);
/// assert_eq!(bop.operation(), Operation::Fuse);
/// ```
pub struct BRepAlgoAPIBooleanOperation {
    builder_algo: BRepAlgoAPIBuilderAlgo,
    tools: List<Shape>,
    operation: Operation,
}

impl Default for BRepAlgoAPIBooleanOperation {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for BRepAlgoAPIBooleanOperation {
    type Target = BRepAlgoAPIBuilderAlgo;
    fn deref(&self) -> &BRepAlgoAPIBuilderAlgo {
        &self.builder_algo
    }
}

impl DerefMut for BRepAlgoAPIBooleanOperation {
    fn deref_mut(&mut self) -> &mut BRepAlgoAPIBuilderAlgo {
        &mut self.builder_algo
    }
}

impl BRepAlgoAPIBooleanOperation {
    /// An operation of unknown type (`BRepAlgoAPI_BooleanOperation()`).
    /// `BRepAlgoAPIBooleanOperation::new()`.
    pub fn new() -> Self {
        Self::with_builder(BRepAlgoAPIBuilderAlgo::new(), Operation::Unknown)
    }

    /// An operation on the results of a performed `filler`
    /// (`BRepAlgoAPI_BooleanOperation(thePF)`); `BRepAlgoAPIBooleanOperation::with_filler(pf)`.
    pub fn with_filler(filler: BOPAlgoPaveFiller) -> Self {
        Self::with_builder(
            BRepAlgoAPIBuilderAlgo::with_filler(filler),
            Operation::Unknown,
        )
    }

    /// The operation of `object` and `tool`, intersected by an own filler or by `filler`
    /// (the protected constructors).
    pub(crate) fn with_shapes(
        object: &Shape,
        tool: &Shape,
        filler: Option<BOPAlgoPaveFiller>,
        operation: Operation,
    ) -> Self {
        let builder_algo = match filler {
            Some(filler) => BRepAlgoAPIBuilderAlgo::with_filler(filler),
            None => BRepAlgoAPIBuilderAlgo::new(),
        };
        let mut boolean = Self::with_builder(builder_algo, operation);
        boolean.builder_algo.arguments.append(object.clone());
        boolean.tools.append(tool.clone());
        boolean
    }

    fn with_builder(builder_algo: BRepAlgoAPIBuilderAlgo, operation: Operation) -> Self {
        Self {
            builder_algo,
            tools: List::default(),
            operation,
        }
    }

    /// The first argument (`Shape1`); `bop.shape1()`.
    pub fn shape1(&self) -> Option<&Shape> {
        self.builder_algo.arguments.iter().next()
    }

    /// The first tool (`Shape2`); `bop.shape2()`.
    pub fn shape2(&self) -> Option<&Shape> {
        self.tools.iter().next()
    }

    /// Replaces the tools by a copy of `tools` (`SetTools`); `bop.set_tools(&tools)`.
    pub fn set_tools(&mut self, tools: &List<Shape>) {
        self.tools = tools.clone();
    }

    /// The tools (`Tools`); `bop.tools().len()`.
    pub fn tools(&self) -> &List<Shape> {
        &self.tools
    }

    /// Sets the operation (`SetOperation`); `bop.set_operation(Operation::Cut)`.
    pub fn set_operation(&mut self, operation: Operation) {
        self.operation = operation;
    }

    /// The operation (`Operation`); `bop.operation()`.
    pub fn operation(&self) -> Operation {
        self.operation
    }

    /// Performs the operation (`Build`): the arguments and tools intersected together, then
    /// the Boolean operation or the section on the results; `bop.build()?`.
    pub fn build(&mut self) -> Result<(), Failure> {
        self.builder_algo.base.set_not_done();
        self.builder_algo.clear();
        if self.builder_algo.arguments.is_empty() || self.tools.is_empty() {
            self.builder_algo
                .options
                .add_error(&alert(AlertTooFewArguments));
            return Ok(());
        }
        if self.operation == Operation::Unknown {
            self.builder_algo.options.add_error(&alert(AlertBOPNotSet));
            return Ok(());
        }
        if self.builder_algo.is_intersection_needed {
            let mut shapes = self.builder_algo.arguments.clone();
            for tool in self.tools.iter() {
                shapes.append(tool.clone());
            }
            self.builder_algo.intersect_shapes(&shapes)?;
            if self.has_errors() {
                return Ok(());
            }
        }
        self.builder_algo.builder = Some(self.operation_builder());
        self.builder_algo.build_result()
    }

    /// The section on the filler arguments, or the Boolean operation of the arguments and
    /// the tools.
    fn operation_builder(&self) -> Box<dyn GeneralFuseAlgorithm> {
        if self.operation == Operation::Section {
            let mut section = BOPAlgoSection::new();
            if let Some(filler) = self.builder_algo.ds_filler() {
                section.set_arguments(filler.arguments());
            }
            return Box::new(section);
        }
        let mut bop = BOPAlgoBOP::new();
        bop.set_arguments(&self.builder_algo.arguments);
        bop.set_tools(&self.tools);
        bop.set_operation(self.operation);
        Box::new(bop)
    }
}
