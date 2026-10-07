//! `BOPAlgo_BOP`: the Boolean operations (common, fuse, cut, cut21) of arguments and tools.
use super::builder::perform::{
    adopt_filler, fill_images_with_results, no_structure, perform_on, perform_with,
};
use super::{BOPAlgoBuilder, BOPAlgoPaveFiller, BOPAlgoToolsProvider, BuilderHooks, Operation};
use crate::bop_ds::DataStructure;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::ops::{Deref, DerefMut};

#[path = "bop/build_rc.rs"]
mod build_rc;
#[path = "bop/build_shape.rs"]
mod build_shape;
#[path = "bop/build_solid.rs"]
mod build_solid;
#[path = "bop/check.rs"]
mod check;

/// The Boolean operation of the arguments (objects) and the tools, built on the General
/// Fuse of both groups.
///
/// ```
/// use rustycad_tkbo::bop_algo::{BOPAlgoBOP, Operation};
/// let mut bop = BOPAlgoBOP::new();
/// bop.set_operation(Operation::Cut);
/// assert_eq!(bop.operation(), Operation::Cut);
/// ```
pub struct BOPAlgoBOP {
    provider: BOPAlgoToolsProvider,
    operation: Operation,
    dims: [i32; 2],
    /// `myRC`: the splits making the result before the containers are rebuilt.
    rc: Shape,
}

impl Default for BOPAlgoBOP {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for BOPAlgoBOP {
    type Target = BOPAlgoToolsProvider;
    fn deref(&self) -> &BOPAlgoToolsProvider {
        &self.provider
    }
}

impl DerefMut for BOPAlgoBOP {
    fn deref_mut(&mut self) -> &mut BOPAlgoToolsProvider {
        &mut self.provider
    }
}

impl BOPAlgoBOP {
    /// An operation of unknown type with no shapes (`BOPAlgo_BOP()`); `BOPAlgoBOP::new()`.
    pub fn new() -> Self {
        Self {
            provider: BOPAlgoToolsProvider::new(),
            operation: Operation::Unknown,
            dims: [-1, -1],
            rc: Shape::default(),
        }
    }

    /// Forgets the operation, the dimensions and the shapes (`Clear`); `bop.clear()`.
    pub fn clear(&mut self) {
        self.operation = Operation::Unknown;
        self.dims = [-1, -1];
        self.provider.clear();
    }

    /// Sets the operation (`SetOperation`); `bop.set_operation(Operation::Fuse)`.
    pub fn set_operation(&mut self, operation: Operation) {
        self.operation = operation;
    }

    /// The operation (`Operation`); `bop.operation()`.
    pub fn operation(&self) -> Operation {
        self.operation
    }

    /// Intersects the arguments and the tools with an own pave filler, then builds the
    /// result (`Perform`); `bop.perform();`.
    pub fn perform(&mut self) {
        let mut shapes: List<Shape> = self.provider.builder.arguments.clone();
        for tool in self.provider.tools().iter() {
            shapes.append(tool.clone());
        }
        perform_on(self, shapes);
    }

    /// Builds the result from the results of `filler` (`PerformWithFiller`).
    /// `bop.perform_with_filler(&mut filler);`.
    pub fn perform_with_filler(&mut self, filler: &mut BOPAlgoPaveFiller) {
        perform_with(self, filler);
    }

    /// The pipeline of the Boolean operation (`PerformInternal1`): the General Fuse stages,
    /// then the result shape, the history and the post treatment.
    fn run(&mut self, filler: &mut BOPAlgoPaveFiller) -> Result<(), Failure> {
        adopt_filler(&mut self.provider.builder, filler);
        self.check_operation_data(filler)?;
        if self.builder().options().has_errors() {
            return Ok(());
        }
        self.provider.builder.prepare();
        let Some(structure) = filler.data_structure_mut() else {
            return Err(no_structure());
        };
        if self.has_empty_shape_warning() && self.treat_empty_shape()? {
            return self.provider.builder.prepare_history(structure);
        }
        if !fill_images_with_results(self, structure)? {
            return Ok(());
        }
        self.build_shape(structure)?;
        if self.builder().options().has_errors() {
            return Ok(());
        }
        let builder = &mut self.provider.builder;
        builder.prepare_history(structure)?;
        if builder.options().has_errors() {
            return Ok(());
        }
        builder.post_treat(structure)
    }

    fn builder(&self) -> &BOPAlgoBuilder {
        &self.provider.builder
    }
}

impl BuilderHooks for BOPAlgoBOP {
    fn general_fuse(&mut self) -> &mut BOPAlgoBuilder {
        &mut self.provider.builder
    }

    fn build_result(&mut self, kind: ShapeEnum, structure: &DataStructure) -> Result<(), Failure> {
        self.build_result_of_arguments(kind, structure)
    }

    fn perform_internal1(&mut self, filler: &mut BOPAlgoPaveFiller) -> Result<(), Failure> {
        self.run(filler)
    }
}
