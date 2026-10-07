//! `BRepAlgoAPI_BuilderAlgo`: the API of the General Fuse, and the root of the Boolean and
//! split APIs: intersection by a pave filler, then the result by a General Fuse algorithm.
use super::algo::BRepAlgoAPIAlgo;
use crate::bop_algo::{
    BOPAlgoBuilder, BOPAlgoPaveFiller, GeneralFuseAlgorithm, Glue, SectionAttribute,
};
use rustycad_tkbrep::brep_tools::History;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use std::ops::{Deref, DerefMut};

/// The General Fuse API: splits the arguments by each other and keeps the history.
///
/// ```
/// use rustycad_tkbo::brep_algo_api::BRepAlgoAPIBuilderAlgo;
/// let builder = BRepAlgoAPIBuilderAlgo::new();
/// assert!(builder.arguments().is_empty() && builder.has_history());
/// ```
pub struct BRepAlgoAPIBuilderAlgo {
    algo: BRepAlgoAPIAlgo,
    pub(crate) arguments: List<Shape>,
    non_destructive: bool,
    glue: Glue,
    check_inverted: bool,
    fill_history: bool,
    /// False when the filler was given at construction (`myIsIntersectionNeeded`).
    pub(crate) is_intersection_needed: bool,
    pub(crate) ds_filler: Option<BOPAlgoPaveFiller>,
    pub(crate) builder: Option<Box<dyn GeneralFuseAlgorithm>>,
    history: Handle<History>,
    simplifier_history: Handle<History>,
    /// The face/face intersection options set by the section API, none for the others.
    pub(crate) section_attribute: Option<SectionAttribute>,
}

impl BRepAlgoAPIBuilderAlgo {
    /// An empty General Fuse with the native defaults (`BRepAlgoAPI_BuilderAlgo()`).
    /// `BRepAlgoAPIBuilderAlgo::new()`.
    pub fn new() -> Self {
        Self {
            algo: BRepAlgoAPIAlgo::new(),
            arguments: List::default(),
            non_destructive: false,
            glue: Glue::Off,
            check_inverted: true,
            fill_history: true,
            is_intersection_needed: true,
            ds_filler: None,
            builder: None,
            history: Handle::default(),
            simplifier_history: Handle::default(),
            section_attribute: None,
        }
    }

    /// A General Fuse on the results of a performed `filler`, which is not intersected again
    /// (`BRepAlgoAPI_BuilderAlgo(thePF)`); `BRepAlgoAPIBuilderAlgo::with_filler(filler)`.
    pub fn with_filler(filler: BOPAlgoPaveFiller) -> Self {
        let mut builder = Self::new();
        builder.is_intersection_needed = false;
        builder.ds_filler = Some(filler);
        builder
    }

    /// Replaces the arguments by a copy of `arguments` (`SetArguments`).
    /// `builder.set_arguments(&list)`.
    pub fn set_arguments(&mut self, arguments: &List<Shape>) {
        self.arguments = arguments.clone();
    }

    /// The arguments (`Arguments`); `builder.arguments().len()`.
    pub fn arguments(&self) -> &List<Shape> {
        &self.arguments
    }

    /// Sets the non-destructive mode (`SetNonDestructive`); `builder.set_non_destructive(true)`.
    pub fn set_non_destructive(&mut self, enabled: bool) {
        self.non_destructive = enabled;
    }

    /// The non-destructive mode (`NonDestructive`); `builder.non_destructive()`.
    pub fn non_destructive(&self) -> bool {
        self.non_destructive
    }

    /// Sets the glue mode (`SetGlue`); `builder.set_glue(Glue::Full)`.
    pub fn set_glue(&mut self, glue: Glue) {
        self.glue = glue;
    }

    /// The glue mode (`Glue`); `builder.glue()`.
    pub fn glue(&self) -> Glue {
        self.glue
    }

    /// Sets the inverted-solid check (`SetCheckInverted`); `builder.set_check_inverted(false)`.
    pub fn set_check_inverted(&mut self, enabled: bool) {
        self.check_inverted = enabled;
    }

    /// The inverted-solid check (`CheckInverted`); `builder.check_inverted()`.
    pub fn check_inverted(&self) -> bool {
        self.check_inverted
    }

    /// Turns history filling on or off (`SetToFillHistory`); `builder.set_to_fill_history(false)`.
    pub fn set_to_fill_history(&mut self, enabled: bool) {
        self.fill_history = enabled;
    }

    /// Whether the history is filled (`HasHistory`); `builder.has_history()`.
    pub fn has_history(&self) -> bool {
        self.fill_history
    }

    /// The intersection tool (`DSFiller`); `builder.ds_filler()`.
    pub fn ds_filler(&self) -> Option<&BOPAlgoPaveFiller> {
        self.ds_filler.as_ref()
    }

    /// Takes the intersection tool out, for reuse by another operation.
    /// `let filler = builder.take_ds_filler();`.
    pub fn take_ds_filler(&mut self) -> Option<BOPAlgoPaveFiller> {
        self.ds_filler.take()
    }

    /// The General Fuse algorithm of the last build (`Builder`); `builder.builder()`.
    pub fn builder(&self) -> Option<&BOPAlgoBuilder> {
        self.builder
            .as_deref()
            .map(|builder| builder.general_fuse())
    }

    /// The history of the operation while history filling is on (`History`).
    /// `builder.history()`.
    pub fn history(&self) -> Option<&Handle<History>> {
        self.fill_history.then_some(&self.history)
    }

    /// Splits the arguments (`Build`); `builder.build()?`.
    pub fn build(&mut self) -> Result<(), Failure> {
        self.base.set_not_done();
        self.clear();
        let arguments = self.arguments.clone();
        self.intersect_shapes(&arguments)?;
        if self.has_errors() {
            return Ok(());
        }
        let mut general_fuse = BOPAlgoBuilder::new();
        general_fuse.set_arguments(&arguments);
        self.builder = Some(Box::new(general_fuse));
        self.build_result()
    }

    /// Forgets the report, the own filler, the builder and the histories (`Clear`).
    pub fn clear(&mut self) {
        self.algo.clear();
        if self.is_intersection_needed {
            self.ds_filler = None;
        }
        self.builder = None;
        self.history = Handle::default();
        self.simplifier_history = Handle::default();
    }

    /// Intersects `arguments` with a new pave filler when no filler was given
    /// (`IntersectShapes`).
    pub(crate) fn intersect_shapes(&mut self, arguments: &List<Shape>) -> Result<(), Failure> {
        if !self.is_intersection_needed {
            return Ok(());
        }
        let mut filler = BOPAlgoPaveFiller::new();
        filler.set_arguments(arguments);
        let options = filler.options_mut();
        options.set_run_parallel(self.run_parallel());
        options.set_fuzzy_value(self.fuzzy_value());
        options.set_use_obb(self.options.use_obb());
        filler.set_non_destructive(self.non_destructive);
        filler.set_glue(self.glue);
        if let Some(attribute) = self.section_attribute {
            // The face/face intersection options of the section API (`SetAttributes`).
            filler.set_section_attribute(&attribute);
        }
        filler.perform();
        self.get_report().merge(filler.options().report());
        self.ds_filler = Some(filler);
        Ok(())
    }

    /// Builds the result with the General Fuse algorithm on the filler results and keeps
    /// its history (`BuildResult`).
    pub(crate) fn build_result(&mut self) -> Result<(), Failure> {
        let (Some(builder), Some(filler)) = (self.builder.as_mut(), self.ds_filler.as_mut()) else {
            return Ok(());
        };
        let general_fuse = builder.general_fuse_mut();
        general_fuse
            .options_mut()
            .set_run_parallel(self.algo.run_parallel());
        general_fuse.set_check_inverted(self.check_inverted);
        general_fuse.set_to_fill_history(self.fill_history);
        builder.perform_with_filler(filler);
        let general_fuse = builder.general_fuse_mut();
        self.algo
            .get_report()
            .merge(general_fuse.options().report());
        if general_fuse.options().has_errors() {
            return Ok(());
        }
        self.algo.base.set_done();
        self.algo.base.my_shape = general_fuse.shape().clone();
        if self.fill_history {
            let history = Handle::new(History::new());
            if let Some(source) = general_fuse.history() {
                history
                    .get()
                    .expect("created history")
                    .merge_handle(&source)?;
            }
            self.history = history;
        }
        Ok(())
    }

    /// The splits of `shape` in the result (`Modified`); `builder.modified(&face)?`.
    pub fn modified(&self, shape: &Shape) -> Result<Vec<Shape>, Failure> {
        match self.filled_history() {
            Some(history) => history.modified(shape),
            None => Ok(Vec::new()),
        }
    }

    /// The shapes generated from `shape` in the result (`Generated`).
    /// `builder.generated(&face)?`.
    pub fn generated(&self, shape: &Shape) -> Result<Vec<Shape>, Failure> {
        match self.filled_history() {
            Some(history) => history.generated(shape),
            None => Ok(Vec::new()),
        }
    }

    /// Whether `shape` has no trace in the result (`IsDeleted`); `builder.is_deleted(&face)?`.
    pub fn is_deleted(&self, shape: &Shape) -> Result<bool, Failure> {
        match self.filled_history() {
            Some(history) => history.is_removed(shape),
            None => Ok(false),
        }
    }

    /// Whether a shape was modified (`HasModified`); `builder.has_modified()`.
    pub fn has_modified(&self) -> bool {
        self.filled_history().is_some_and(History::has_modified)
    }

    /// Whether a shape generated others (`HasGenerated`); `builder.has_generated()`.
    pub fn has_generated(&self) -> bool {
        self.filled_history().is_some_and(History::has_generated)
    }

    /// Whether a shape was deleted (`HasDeleted`); `builder.has_deleted()`.
    pub fn has_deleted(&self) -> bool {
        self.filled_history().is_some_and(History::has_removed)
    }

    fn filled_history(&self) -> Option<&History> {
        if self.fill_history {
            self.history.get()
        } else {
            None
        }
    }

    /// The section edges of all face/face interferences, once each, replaced by their
    /// simplified edges (`SectionEdges`); `builder.section_edges()?`.
    pub fn section_edges(&self) -> Result<List<Shape>, Failure> {
        let mut edges = List::default();
        let (Some(_), Some(filler)) = (self.builder.as_ref(), self.ds_filler.as_ref()) else {
            return Ok(edges);
        };
        let Some(structure) = filler.data_structure() else {
            return Ok(edges);
        };
        let mut fence = MapOfShape::default();
        for record in structure.interf_ff_records().iter() {
            for curve in record.curves().iter() {
                for block in curve.pave_blocks() {
                    let edge = structure.shape(block.lock().expect("pave-block lock").edge())?;
                    if fence.add(edge.clone()) {
                        self.add_simplified_section_edge(edge, &mut fence, &mut edges)?;
                    }
                }
            }
        }
        Ok(edges)
    }

    fn add_simplified_section_edge(
        &self,
        edge: &Shape,
        fence: &mut MapOfShape,
        edges: &mut List<Shape>,
    ) -> Result<(), Failure> {
        let Some(simplifier) = self.simplifier_history.get() else {
            edges.append(edge.clone());
            return Ok(());
        };
        if simplifier.is_removed(edge)? {
            return Ok(());
        }
        let images = simplifier.modified(edge)?;
        if images.is_empty() {
            edges.append(edge.clone());
            return Ok(());
        }
        for image in images {
            if fence.add(image.clone()) {
                edges.append(image);
            }
        }
        Ok(())
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

/// The queries a `BRepTools_History` is built from (the `TheAlgo` of its template
/// constructor and `Merge`), for the BRepAlgoAPI Boolean algorithms.
impl rustycad_tkbrep::brep_tools::HistoryAlgorithm for BRepAlgoAPIBuilderAlgo {
    fn is_deleted(&mut self, shape: &Shape) -> Result<bool, Failure> {
        BRepAlgoAPIBuilderAlgo::is_deleted(self, shape)
    }

    fn modified(&mut self, shape: &Shape) -> Result<Vec<Shape>, Failure> {
        BRepAlgoAPIBuilderAlgo::modified(self, shape)
    }

    fn generated(&mut self, shape: &Shape) -> Result<Vec<Shape>, Failure> {
        BRepAlgoAPIBuilderAlgo::generated(self, shape)
    }
}
