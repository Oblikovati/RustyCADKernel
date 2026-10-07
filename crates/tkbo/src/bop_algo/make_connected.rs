//! `BOPAlgo_MakeConnected`: glues shapes of one dimension into a connected shape, keeps the
//! material on each side of the shared boundary elements, and makes the result periodic
//! and repeated through `BOPAlgo_MakePeriodic`.
#[path = "make_connected/materials.rs"]
mod materials;
use super::alerts::alert;
use super::make_periodic::compound_of;
use super::{
    AlertMultiDimensionalArguments, AlertShapeIsNotPeriodic, AlertTooFewArguments,
    AlertUnableToGlue, AlertUnableToMakePeriodic, BOPAlgoBuilder, BOPAlgoMakePeriodic, Glue,
    Options, PeriodicityParams,
};
use crate::bop_tools::{dimension, treat_compound};
use rustycad_tkbrep::brep_tools::History;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::{IndexedMapOfShape, MapOfShape, ShapeMapHasher};
use rustycad_tkernel::message::AlertHandle;
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::{Failure, Handle};
use std::sync::Arc;

/// The materials of the boundary elements, by oriented element (`myMaterials`): the default
/// hasher of the native map compares orientations, so the two sides are two keys.
type MaterialMap = DataMap<Shape, List<Shape>>;

/// The input shapes each result shape comes from (`myOrigins`).
type OriginMap = DataMap<Shape, List<Shape>, ShapeMapHasher>;

/// The connected-shape maker: glues the arguments, then answers which input shapes lie on
/// each side of a boundary element; the result can be made periodic and repeated.
///
/// ```
/// use rustycad_tkbo::bop_algo::BOPAlgoMakeConnected;
/// let mut maker = BOPAlgoMakeConnected::new();
/// maker.perform().unwrap();
/// assert!(maker.options().has_errors()); // no arguments
/// ```
#[derive(Default)]
pub struct BOPAlgoMakeConnected {
    options: Options,
    arguments: List<Shape>,
    all_inputs: IndexedMapOfShape,
    periodicity_maker: BOPAlgoMakePeriodic,
    materials: MaterialMap,
    origins: OriginMap,
    glue_history: Handle<History>,
    history: Handle<History>,
    /// The connected shape (`myGlued`).
    glued: Shape,
    /// The working shape: the glued shape, then made periodic and repeated (`myShape`).
    periodic: Shape,
    /// The empty list returned for a shape without materials or origins (`EmptyList`).
    no_shapes: List<Shape>,
}

impl BOPAlgoMakeConnected {
    /// A maker without arguments (`BOPAlgo_MakeConnected()`); `BOPAlgoMakeConnected::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// The error and warning report and the parallel mode (`BOPAlgo_Options`);
    /// `maker.options().has_errors()`.
    pub fn options(&self) -> &Options {
        &self.options
    }

    /// The mutable options; `maker.options_mut().set_run_parallel(false)`.
    pub fn options_mut(&mut self) -> &mut Options {
        &mut self.options
    }

    /// Sets the shapes to connect (`SetArguments`); `maker.set_arguments(&shapes)`.
    pub fn set_arguments(&mut self, arguments: &List<Shape>) {
        self.arguments = arguments.clone();
    }

    /// Adds a shape to connect (`AddArgument`); `maker.add_argument(&solid)`.
    pub fn add_argument(&mut self, shape: &Shape) {
        self.arguments.append(shape.clone());
    }

    /// The shapes to connect (`Arguments`); `maker.arguments().extent()`.
    pub fn arguments(&self) -> &List<Shape> {
        &self.arguments
    }

    /// The periodicity tool (`PeriodicityTool`); `maker.periodicity_tool().x_period()`.
    pub fn periodicity_tool(&self) -> &BOPAlgoMakePeriodic {
        &self.periodicity_maker
    }

    /// The history of the input shapes (`History`); `maker.history().is_null()`.
    pub fn history(&self) -> &Handle<History> {
        &self.history
    }

    /// The connected (glued) shape (`Shape`); `maker.shape()`.
    pub fn shape(&self) -> &Shape {
        &self.glued
    }

    /// The glued shape made periodic and repeated (`PeriodicShape`); `maker.periodic_shape()`.
    pub fn periodic_shape(&self) -> &Shape {
        &self.periodic
    }

    /// Checks the arguments, glues them and associates the materials (`Perform`);
    /// `maker.perform()?`.
    pub fn perform(&mut self) -> Result<(), Failure> {
        self.check_data()?;
        if self.options.has_errors() {
            return Ok(());
        }
        if self.history.is_null() {
            self.history = Handle::new(History::new());
        }
        self.make_connected()?;
        if self.options.has_errors() {
            return Ok(());
        }
        self.associate_materials()
    }

    /// An error without arguments, with only empty compounds, or with leaves of several
    /// dimensions (`CheckData`).
    fn check_data(&self) -> Result<(), Failure> {
        if self.arguments.is_empty() {
            self.options.add_error(&alert(AlertTooFewArguments));
            return Ok(());
        }
        let (mut leaves, mut fence) = (Vec::new(), MapOfShape::default());
        for argument in self.arguments.iter() {
            treat_compound(argument, &mut leaves, Some(&mut fence))?;
        }
        let Some(first) = leaves.first() else {
            self.options.add_error(&alert(AlertTooFewArguments));
            return Ok(());
        };
        let first_dimension = dimension(first)?;
        for leaf in &leaves[1..] {
            if dimension(leaf)? != first_dimension {
                self.options
                    .add_error(&alert(AlertMultiDimensionalArguments));
                return Ok(());
            }
        }
        Ok(())
    }

    /// Glues several arguments by shift keeping the inputs, records the gluing history and
    /// the origins (`MakeConnected`).
    fn make_connected(&mut self) -> Result<(), Failure> {
        if self.glue_history.is_null() {
            self.glue_history = Handle::new(History::new());
        }
        match self.arguments.first() {
            // No need to glue the single shape.
            Some(single) if self.arguments.extent() == 1 => self.periodic = single.clone(),
            _ => {
                if !self.glue_arguments()? {
                    return Ok(());
                }
            }
        }
        self.glued = self.periodic.clone();
        self.fill_origins()
    }

    /// The General Fuse of the arguments glued by shift; false after `AlertUnableToGlue`.
    fn glue_arguments(&mut self) -> Result<bool, Failure> {
        let mut gluer = BOPAlgoBuilder::new();
        gluer.set_arguments(&self.arguments);
        gluer.set_glue(Glue::Shift);
        gluer
            .options_mut()
            .set_run_parallel(self.options.run_parallel());
        gluer.set_non_destructive(true);
        gluer.perform();
        if gluer.options().has_errors() {
            let arguments: Vec<Shape> = self.arguments.iter().cloned().collect();
            let alert: AlertHandle = Arc::new(AlertUnableToGlue::new(&compound_of(&arguments)?));
            self.options.add_error(&alert);
            return Ok(false);
        }
        self.periodic = gluer.shape().clone();
        let arguments: Vec<Shape> = gluer.arguments().iter().cloned().collect();
        let glue_history = self.glue_history.get().ok_or_else(null_history)?;
        glue_history.merge_algorithm(&arguments, &mut *gluer)?;
        self.history()
            .get()
            .ok_or_else(null_history)?
            .merge_handle(&self.glue_history)?;
        Ok(true)
    }

    /// Makes the glued shape periodic with `params`; a periodicity failure only warns
    /// (`MakePeriodic`); `maker.make_periodic(&params)?`.
    pub fn make_periodic(&mut self, params: &PeriodicityParams) -> Result<(), Failure> {
        if self.options.has_errors() {
            return Ok(());
        }
        let maker = &mut self.periodicity_maker;
        maker.clear()?;
        maker.set_shape(&self.glued);
        maker.set_periodicity_parameters(params);
        maker
            .options_mut()
            .set_run_parallel(self.options.run_parallel());
        maker.perform()?;
        if maker.options().has_errors() {
            let alert: AlertHandle = Arc::new(AlertUnableToMakePeriodic::new(&self.periodic));
            self.options.add_warning(&alert);
            return Ok(());
        }
        self.periodic = maker.shape().clone();
        self.update()
    }

    /// Repeats the periodic shape `times` along the direction; a warning when the shape is
    /// not periodic (`RepeatShape`); `maker.repeat_shape(0, 2)?`.
    pub fn repeat_shape(&mut self, direction: i32, times: i32) -> Result<(), Failure> {
        if !self.is_periodic_ready() {
            return Ok(());
        }
        self.periodic = self
            .periodicity_maker
            .repeat_shape(direction, times)?
            .clone();
        self.update()
    }

    /// Forgets the repetitions, keeping the shape periodic; a warning when the shape is not
    /// periodic (`ClearRepetitions`); `maker.clear_repetitions()?`.
    pub fn clear_repetitions(&mut self) -> Result<(), Failure> {
        if !self.is_periodic_ready() {
            return Ok(());
        }
        self.periodicity_maker.clear_repetitions()?;
        self.periodic = self.periodicity_maker.shape().clone();
        self.update()
    }

    /// Whether the maker has no error and a periodic shape; warns
    /// `AlertShapeIsNotPeriodic` when it has none.
    fn is_periodic_ready(&self) -> bool {
        if self.options.has_errors() {
            return false;
        }
        let maker = &self.periodicity_maker;
        if maker.shape().is_null() || maker.options().has_errors() {
            // The shape has not been made periodic yet.
            let alert: AlertHandle = Arc::new(AlertShapeIsNotPeriodic::new(&self.periodic));
            self.options.add_warning(&alert);
            return false;
        }
        true
    }

    /// Forgets the arguments, the results, the histories and the periodicity (`Clear`);
    /// `maker.clear()?`.
    pub fn clear(&mut self) -> Result<(), Failure> {
        self.options.clear();
        self.arguments.clear();
        self.all_inputs.clear(true);
        self.periodicity_maker.clear()?;
        self.origins.clear(true);
        self.materials.clear(true);
        for history in [&self.glue_history, &self.history] {
            if let Some(history) = history.get() {
                history.clear();
            }
        }
        self.glued.nullify();
        self.periodic.nullify();
        Ok(())
    }
}

/// The failure of a step that needs the history `Perform` creates.
pub(super) fn null_history() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BOPAlgo_MakeConnected: the history is null; expected Perform to have run",
        "",
    )
}
