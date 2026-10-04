//! Native `BOPAlgo_PaveFiller::Init` state and broad-phase preparation.
#[path = "pave_filler/edge_edge.rs"]
mod edge_edge;
#[path = "pave_filler/edge_face.rs"]
mod edge_face;
#[path = "pave_filler/face_face_prepare.rs"]
mod face_face_prepare;
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "PerformFF call site is a separate queued port")
)]
#[path = "pave_filler/face_face_task.rs"]
mod face_face_task;
#[cfg(test)]
#[path = "pave_filler/face_face_task_tests.rs"]
mod face_face_task_tests;
#[cfg(test)]
#[path = "pave_filler/section_attribute_tests.rs"]
mod section_attribute_tests;
#[path = "pave_filler/shrunk_data.rs"]
mod shrunk_data;
#[path = "pave_filler/split_blocks.rs"]
mod split_blocks;
#[path = "pave_filler/vertex_edge.rs"]
mod vertex_edge;
#[path = "pave_filler/vertex_face.rs"]
mod vertex_face;
#[path = "pave_filler/vertex_vertex.rs"]
mod vertex_vertex;
use super::{AlertNullInputShapes, AlertTooFewArguments, Glue, Options, SectionAttribute};
use crate::{bop_ds::BOPDSIterator, bop_ds::DataStructure, int_tools::Context};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{message::AlertHandle, ncollection::List, standard::Handle};
use std::sync::Arc;

/// `BOPAlgo_PaveFiller` through data-structure and iterator preparation.
/// Boolean intersection stages are not attached yet; `filler.initialize()?` prepares candidates.
pub struct BOPAlgoPaveFiller {
    options: Options,
    arguments: List<Shape>,
    data_structure: Option<DataStructure>,
    context: Handle<Context>,
    iterator: Option<BOPDSIterator>,
    non_destructive: bool,
    glue: Glue,
    avoid_build_pcurve: bool,
    is_primary: bool,
    vertices_to_avoid_extension: Vec<i32>,
    section_attribute: SectionAttribute,
}

impl Default for BOPAlgoPaveFiller {
    fn default() -> Self {
        Self::new()
    }
}

impl BOPAlgoPaveFiller {
    /// Uses native defaults for primary intersection preparation; `BOPAlgoPaveFiller::new()`.
    pub fn new() -> Self {
        Self {
            options: Options::new(),
            arguments: List::default(),
            data_structure: None,
            context: Handle::default(),
            iterator: None,
            non_destructive: false,
            glue: Glue::Off,
            avoid_build_pcurve: false,
            is_primary: true,
            vertices_to_avoid_extension: Vec::new(),
            section_attribute: SectionAttribute::new(),
        }
    }

    /// Replaces the argument list by copy; `filler.set_arguments(&arguments)`.
    pub fn set_arguments(&mut self, arguments: &List<Shape>) {
        self.arguments = arguments.clone();
    }

    /// Appends one argument without deduplication; `filler.add_argument(&shape)`.
    pub fn add_argument(&mut self, shape: &Shape) {
        self.arguments.append(shape.clone());
    }

    /// Borrows configured arguments; `filler.arguments().len()`.
    pub fn arguments(&self) -> &List<Shape> {
        &self.arguments
    }

    /// Copies intersection flags, retained by Clear; `filler.set_section_attribute(&flags)`.
    pub fn set_section_attribute(&mut self, flags: &SectionAttribute) {
        self.section_attribute = *flags;
    }

    /// Prepares the DS, context, broad-phase iterator, and automatic safety mode.
    /// `filler.initialize()?; assert!(filler.data_structure().is_some());`.
    pub fn initialize(&mut self) -> Result<(), rustycad_tkernel::standard::Failure> {
        if !self.validate_arguments() {
            return Ok(());
        }
        self.clear();
        self.initialize_data_structure()?;
        self.initialize_context_and_iterator()?;
        self.set_non_destructive_from_arguments()?;
        Ok(())
    }

    /// Clears report and prepared DS/iterator while retaining configured arguments and options.
    /// `filler.clear()`.
    pub fn clear(&mut self) {
        self.options.clear();
        self.data_structure = None;
        self.iterator = None;
    }

    /// Borrows the prepared data structure when initialization reached that stage.
    /// `filler.data_structure().is_some()`.
    pub fn data_structure(&self) -> Option<&DataStructure> {
        self.data_structure.as_ref()
    }

    /// Mutably borrows the initialized DS as native `PDS()` does; `filler.data_structure_mut().unwrap()`.
    pub fn data_structure_mut(&mut self) -> Option<&mut DataStructure> {
        self.data_structure.as_mut()
    }

    /// Borrows the geometric context created for broad-phase OBB checks.
    /// `filler.context().is_some()`.
    pub fn context(&self) -> Option<&Context> {
        self.context.get()
    }

    /// Borrows prepared AABB/OBB candidates; `filler.iterator().is_some()`.
    pub fn iterator(&self) -> Option<&BOPDSIterator> {
        self.iterator.as_ref()
    }

    /// Mutably borrows prepared pair traversal; `filler.iterator_mut().unwrap().initialize(...)`.
    pub fn iterator_mut(&mut self) -> Option<&mut BOPDSIterator> {
        self.iterator.as_mut()
    }

    /// Configures non-destructive operation; `filler.set_non_destructive(true)`.
    pub fn set_non_destructive(&mut self, enabled: bool) {
        self.non_destructive = enabled;
    }

    /// Reads configured non-destructive operation; `filler.non_destructive()`.
    pub fn non_destructive(&self) -> bool {
        self.non_destructive
    }

    /// Configures coincident-shape gluing; `filler.set_glue(Glue::Full)`.
    pub fn set_glue(&mut self, glue: Glue) {
        self.glue = glue;
    }

    /// Reads coincident-shape gluing mode; `filler.glue() == Glue::Off`.
    pub fn glue(&self) -> Glue {
        self.glue
    }

    /// Configures p-curve construction avoidance; `filler.set_avoid_build_pcurve(true)`.
    pub fn set_avoid_build_pcurve(&mut self, avoid: bool) {
        self.avoid_build_pcurve = avoid;
    }

    /// Reads p-curve construction avoidance; `filler.is_avoid_build_pcurve()`.
    pub fn is_avoid_build_pcurve(&self) -> bool {
        self.avoid_build_pcurve
    }

    /// Sets whether this filler is the primary operation's intersection engine.
    /// `filler.set_is_primary(false)` prevents automatic safety-mode rewriting.
    pub fn set_is_primary(&mut self, primary: bool) {
        self.is_primary = primary;
    }

    /// Reads primary-operation status; `filler.is_primary()`.
    pub fn is_primary(&self) -> bool {
        self.is_primary
    }

    /// Reads vertices protected from later curve-end extension; `filler.vertices_to_avoid_extension()`.
    pub fn vertices_to_avoid_extension(&self) -> &[i32] {
        &self.vertices_to_avoid_extension
    }

    /// Borrows inherited algorithm options; `filler.options().fuzzy_value()`.
    pub fn options(&self) -> &Options {
        &self.options
    }

    /// Mutably borrows inherited algorithm options; `filler.options_mut().set_use_obb(true)`.
    pub fn options_mut(&mut self) -> &mut Options {
        &mut self.options
    }

    fn validate_arguments(&self) -> bool {
        if self.arguments.is_empty() {
            self.options.add_error(&alert(AlertTooFewArguments));
            return false;
        }
        if self.arguments.iter().any(Shape::is_null) {
            self.options.add_error(&alert(AlertNullInputShapes));
            return false;
        }
        true
    }

    fn initialize_data_structure(&mut self) -> Result<(), rustycad_tkernel::standard::Failure> {
        let arguments: Vec<Shape> = self.arguments.iter().cloned().collect();
        self.data_structure = Some(DataStructure::default());
        let structure = self
            .data_structure
            .as_mut()
            .expect("created data structure");
        structure.set_arguments(&arguments);
        structure.init(self.options.fuzzy_value())?;
        Ok(())
    }

    fn initialize_context_and_iterator(
        &mut self,
    ) -> Result<(), rustycad_tkernel::standard::Failure> {
        self.context = Handle::new(Context::default());
        self.iterator = Some(BOPDSIterator::default());
        let context = self.context.get().expect("created context");
        let structure = self
            .data_structure
            .as_ref()
            .expect("initialized data structure");
        let iterator = self.iterator.as_mut().expect("created iterator");
        iterator.set_run_parallel(self.options.run_parallel());
        iterator.prepare_with_context(
            structure,
            Some(context),
            self.options.use_obb(),
            self.options.fuzzy_value(),
        )?;
        Ok(())
    }

    fn set_non_destructive_from_arguments(
        &mut self,
    ) -> Result<(), rustycad_tkernel::standard::Failure> {
        if !self.is_primary || self.non_destructive {
            return Ok(());
        }
        for shape in self.arguments.iter() {
            if shape.locked()? {
                self.non_destructive = true;
                break;
            }
        }
        Ok(())
    }
}

fn alert<T: rustycad_tkernel::message::Alert + 'static>(value: T) -> AlertHandle {
    Arc::new(value)
}
