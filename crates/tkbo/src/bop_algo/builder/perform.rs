//! `BOPAlgo_Builder::Perform`, `PerformWithFiller`, `PerformInternal(1)`, `CheckData`,
//! `CheckFiller`, `Prepare`, `BuildResult` and `PostTreat` (BOPAlgo_Builder.cxx).
use super::{BOPAlgoBuilder, SameDomainShapes};
use crate::bop_algo::alerts::alert;
use crate::bop_algo::{AlertBuilderFailed, AlertTooFewArguments, BOPAlgoPaveFiller};
use crate::bop_ds::DataStructure;
use crate::bop_tools::{correct_shape_tolerances, correct_tolerances, make_container};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::{IndexedMapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{List, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// The virtual methods of `BOPAlgo_Builder` that the General Fuse pipeline calls and the
/// derived algorithms override (`CheckData`, `BuildResult`).
pub trait BuilderHooks {
    /// The General Fuse state; `hooks.general_fuse().arguments()`.
    fn general_fuse(&mut self) -> &mut BOPAlgoBuilder;

    /// Checks the arguments (`CheckData`): at least two, then the filler report merged.
    fn check_data(&mut self, filler: &BOPAlgoPaveFiller) {
        self.general_fuse().check_data_default(filler);
    }

    /// Adds the images of the arguments of `kind` to the result (`BuildResult`).
    /// The base adds the images of `arguments()`; derived algorithms may read the DS.
    fn build_result(&mut self, kind: ShapeEnum, _structure: &DataStructure) -> Result<(), Failure> {
        self.general_fuse().build_result_default(kind)
    }

    /// Prepares the result after the check (`Prepare`): an empty compound; derived
    /// algorithms (BRepFeat_Builder) add their own preparation.
    fn prepare(&mut self) -> Result<(), Failure> {
        self.general_fuse().prepare();
        Ok(())
    }

    /// Classifies the face splits against the draft solids (`FillIn3DParts`); derived
    /// algorithms (BRepFeat_Builder) filter the IN parts afterwards.
    fn fill_in_3d_parts(
        &mut self,
        structure: &mut DataStructure,
        draft_solids: &mut SameDomainShapes,
    ) -> Result<(), Failure> {
        self.general_fuse()
            .fill_in_3d_parts(structure, draft_solids)
    }

    /// The pipeline run on the results of `filler` (`PerformInternal1`).
    fn perform_internal1(&mut self, filler: &mut BOPAlgoPaveFiller) -> Result<(), Failure>
    where
        Self: Sized,
    {
        perform_internal1(self, filler)
    }
}

impl BuilderHooks for BOPAlgoBuilder {
    fn general_fuse(&mut self) -> &mut BOPAlgoBuilder {
        self
    }
}

/// One stage of the General Fuse pipeline after the check and the preparation.
type Stage<H> = fn(&mut H, &mut DataStructure) -> Result<(), Failure>;

impl BOPAlgoBuilder {
    /// Intersects the arguments with an own pave filler, then builds the splits (`Perform`).
    /// `builder.perform();`.
    pub fn perform(&mut self) {
        let shapes = self.arguments.clone();
        perform_on(self, shapes);
    }

    /// Builds the splits from the results of `filler`, taking its options
    /// (`PerformWithFiller`); `builder.perform_with_filler(&mut filler);`.
    pub fn perform_with_filler(&mut self, filler: &mut BOPAlgoPaveFiller) {
        perform_with(self, filler);
    }

    /// A pave filler on `arguments` with the options of this builder.
    pub(crate) fn new_filler(&self, arguments: List<Shape>) -> BOPAlgoPaveFiller {
        let mut filler = BOPAlgoPaveFiller::new();
        filler.set_arguments(&arguments);
        let options = self.options();
        filler
            .options_mut()
            .set_run_parallel(options.run_parallel());
        filler.options_mut().set_fuzzy_value(options.fuzzy_value());
        filler.set_non_destructive(self.non_destructive);
        filler.set_glue(self.glue);
        filler.options_mut().set_use_obb(options.use_obb());
        filler
    }

    /// At least two arguments, then the filler report merged (`CheckData`).
    pub(crate) fn check_data_default(&mut self, filler: &BOPAlgoPaveFiller) {
        if self.arguments.len() < 2 {
            self.options().add_error(&alert(AlertTooFewArguments));
            return;
        }
        self.check_filler(filler);
    }

    /// Merges the report of the filler (`CheckFiller`).
    pub(crate) fn check_filler(&mut self, filler: &BOPAlgoPaveFiller) {
        self.options().report().merge(filler.options().report());
    }

    /// The result starts as an empty compound (`Prepare`); `builder.prepare()`.
    pub fn prepare(&mut self) {
        make_container(ShapeEnum::Compound, &mut self.shape);
    }

    /// Adds the images of the arguments of `kind`, or the arguments themselves, to the
    /// result once each (`BuildResult`).
    pub(crate) fn build_result_default(&mut self, kind: ShapeEnum) -> Result<(), Failure> {
        let arguments: Vec<Shape> = self.arguments.iter().cloned().collect();
        self.add_argument_images(&arguments, kind)
    }

    /// Adds the images of the `arguments` of `kind`, or the arguments themselves, to the
    /// result once each.
    pub(crate) fn add_argument_images(
        &mut self,
        arguments: &[Shape],
        kind: ShapeEnum,
    ) -> Result<(), Failure> {
        let mut fence: Map<Shape, ShapeMapHasher> = Map::with_hasher(ShapeMapHasher, 1);
        let builder = BRepBuilder::default();
        for argument in arguments {
            if argument.shape_type()? != kind {
                continue;
            }
            let images: Vec<Shape> = match self.images.seek(argument) {
                Some(images) => images.iter().cloned().collect(),
                None => vec![argument.clone()],
            };
            for image in images {
                if fence.add(image.clone()) {
                    builder.add(&mut self.base.shape, &image)?;
                }
            }
        }
        Ok(())
    }

    /// Corrects the tolerances of the result, the input sub-shapes kept as they are in the
    /// non-destructive mode (`PostTreat`).
    pub(crate) fn post_treat(&mut self, structure: &DataStructure) -> Result<(), Failure> {
        let mut avoided = IndexedMapOfShape::default();
        if self.non_destructive {
            for index in 0..structure.nb_source_shapes() {
                let info = structure.shape_info(index)?;
                if matches!(
                    info.shape_type(),
                    ShapeEnum::Vertex | ShapeEnum::Edge | ShapeEnum::Face
                ) {
                    avoided.add(info.shape().clone());
                }
            }
        }
        let parallel = self.options().run_parallel();
        correct_tolerances(&self.shape, &avoided, 0.05, parallel)?;
        correct_shape_tolerances(&self.shape, &avoided, parallel)
    }
}

/// Intersects `shapes` with a pave filler made with the builder options, runs the pipeline
/// and keeps the filler (`Perform` of the builders, entry point 1); a derived algorithm
/// runs its own hooks through it; `perform_on(&mut algo, shapes)`.
pub fn perform_on<H: BuilderHooks>(hooks: &mut H, shapes: List<Shape>) {
    hooks.general_fuse().options().clear();
    let mut filler = hooks.general_fuse().new_filler(shapes);
    filler.perform();
    perform_internal(hooks, &mut filler);
    hooks.general_fuse().filler = Some(filler);
}

/// Takes the options of `filler` and runs the pipeline on it (`PerformWithFiller`, entry
/// point 0: the filler stays with the caller).
pub(crate) fn perform_with<H: BuilderHooks>(hooks: &mut H, filler: &mut BOPAlgoPaveFiller) {
    let builder = hooks.general_fuse();
    builder.options().clear();
    builder.filler = None;
    builder.non_destructive = filler.non_destructive();
    let (fuzzy, use_obb) = (filler.options().fuzzy_value(), filler.options().use_obb());
    builder.options_mut().set_fuzzy_value(fuzzy);
    builder.glue = filler.glue();
    builder.options_mut().set_use_obb(use_obb);
    perform_internal(hooks, filler);
}

/// Runs the pipeline; a failure becomes an `AlertBuilderFailed` error, as the native catch
/// of Standard_Failure does (`PerformInternal`).
pub(crate) fn perform_internal<H: BuilderHooks>(hooks: &mut H, filler: &mut BOPAlgoPaveFiller) {
    hooks.general_fuse().options().clear();
    if hooks.perform_internal1(filler).is_err() {
        hooks
            .general_fuse()
            .options()
            .add_error(&alert(AlertBuilderFailed));
    }
}

/// The General Fuse pipeline (`PerformInternal1`): images of vertices, edges, wires,
/// faces, shells, solids, compsolids and compounds, each followed by `BuildResult`, then
/// the history and the post treatment.
pub(crate) fn perform_internal1<H: BuilderHooks>(
    hooks: &mut H,
    filler: &mut BOPAlgoPaveFiller,
) -> Result<(), Failure> {
    adopt_filler(hooks.general_fuse(), filler);
    hooks.check_data(filler);
    if hooks.general_fuse().options().has_errors() {
        return Ok(());
    }
    hooks.prepare()?;
    if hooks.general_fuse().options().has_errors() {
        return Ok(());
    }
    let Some(structure) = filler.data_structure_mut() else {
        return Err(no_structure());
    };
    if !fill_images_with_results(hooks, structure)? {
        return Ok(());
    }
    let builder = hooks.general_fuse();
    builder.prepare_history(structure)?;
    if builder.options().has_errors() {
        return Ok(());
    }
    builder.post_treat(structure)
}

/// The image stages from vertices to compounds, each followed by `BuildResult`; false when
/// a stage reported an error.
pub(crate) fn fill_images_with_results<H: BuilderHooks>(
    hooks: &mut H,
    structure: &mut DataStructure,
) -> Result<bool, Failure> {
    let stages: [(Stage<H>, ShapeEnum); 8] = [
        (|h, s| h.general_fuse().stage_vertices(s), ShapeEnum::Vertex),
        (|h, s| h.general_fuse().stage_edges(s), ShapeEnum::Edge),
        (|h, s| h.general_fuse().stage_wires(s), ShapeEnum::Wire),
        (|h, s| h.general_fuse().stage_faces(s), ShapeEnum::Face),
        (|h, s| h.general_fuse().stage_shells(s), ShapeEnum::Shell),
        (
            |h, s| super::solids::fill_images_solids_with(h, s),
            ShapeEnum::Solid,
        ),
        (
            |h, s| h.general_fuse().stage_compsolids(s),
            ShapeEnum::CompSolid,
        ),
        (
            |h, s| h.general_fuse().stage_compounds(s),
            ShapeEnum::Compound,
        ),
    ];
    for (stage, kind) in stages {
        stage(hooks, structure)?;
        if hooks.general_fuse().options().has_errors() {
            return Ok(false);
        }
        hooks.build_result(kind, structure)?;
        if hooks.general_fuse().options().has_errors() {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Takes the context and the options of the filler (the first lines of `PerformInternal1`).
pub(crate) fn adopt_filler(builder: &mut BOPAlgoBuilder, filler: &BOPAlgoPaveFiller) {
    builder.context = filler.context_handle();
    let fuzzy = filler.options().fuzzy_value();
    builder.options_mut().set_fuzzy_value(fuzzy);
    builder.non_destructive = filler.non_destructive();
}

/// The failure of a pipeline run on a filler that built no data structure.
pub(crate) fn no_structure() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BOPAlgo_Builder: the pave filler has no data structure; expected a performed filler",
        "",
    )
}

impl BOPAlgoBuilder {
    fn stage_vertices(&mut self, structure: &mut DataStructure) -> Result<(), Failure> {
        self.fill_images_vertices(structure)
    }

    fn stage_edges(&mut self, structure: &mut DataStructure) -> Result<(), Failure> {
        self.fill_images_edges(structure)
    }

    fn stage_wires(&mut self, structure: &mut DataStructure) -> Result<(), Failure> {
        self.fill_images_containers(ShapeEnum::Wire, structure)
    }

    fn stage_faces(&mut self, structure: &mut DataStructure) -> Result<(), Failure> {
        self.fill_images_faces(structure)
    }

    fn stage_shells(&mut self, structure: &mut DataStructure) -> Result<(), Failure> {
        self.fill_images_containers(ShapeEnum::Shell, structure)
    }

    fn stage_compsolids(&mut self, structure: &mut DataStructure) -> Result<(), Failure> {
        self.fill_images_containers(ShapeEnum::CompSolid, structure)
    }

    fn stage_compounds(&mut self, structure: &mut DataStructure) -> Result<(), Failure> {
        self.fill_images_compounds(structure)
    }
}
