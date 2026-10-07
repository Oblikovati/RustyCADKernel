//! `BOPAlgo_Builder::Perform`, `PerformWithFiller`, `PerformInternal(1)`, `CheckData`,
//! `CheckFiller`, `Prepare`, `BuildResult` and `PostTreat` (BOPAlgo_Builder.cxx).
use super::BOPAlgoBuilder;
use crate::bop_algo::alerts::alert;
use crate::bop_algo::{AlertBuilderFailed, AlertTooFewArguments, BOPAlgoPaveFiller};
use crate::bop_ds::DataStructure;
use crate::bop_tools::{correct_shape_tolerances, correct_tolerances, make_container};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::{IndexedMapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::Map;
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
    fn build_result(&mut self, kind: ShapeEnum) -> Result<(), Failure> {
        self.general_fuse().build_result_default(kind)
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
        self.options().clear();
        let mut filler = self.new_filler(self.arguments.clone());
        filler.perform();
        perform_internal(self, &mut filler);
        self.filler = Some(filler);
    }

    /// Builds the splits from the results of `filler`, taking its options
    /// (`PerformWithFiller`); `builder.perform_with_filler(&mut filler);`.
    pub fn perform_with_filler(&mut self, filler: &mut BOPAlgoPaveFiller) {
        self.options().clear();
        self.filler = None;
        self.non_destructive = filler.non_destructive();
        let (fuzzy, use_obb) = (filler.options().fuzzy_value(), filler.options().use_obb());
        self.options_mut().set_fuzzy_value(fuzzy);
        self.glue = filler.glue();
        self.options_mut().set_use_obb(use_obb);
        perform_internal(self, filler);
    }

    /// A pave filler on `arguments` with the options of this builder.
    pub(crate) fn new_filler(
        &self,
        arguments: rustycad_tkernel::ncollection::List<Shape>,
    ) -> BOPAlgoPaveFiller {
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

    /// The result starts as an empty compound (`Prepare`).
    pub(crate) fn prepare(&mut self) {
        make_container(ShapeEnum::Compound, &mut self.shape);
    }

    /// Adds the images of the arguments of `kind`, or the arguments themselves, to the
    /// result once each (`BuildResult`).
    pub(crate) fn build_result_default(&mut self, kind: ShapeEnum) -> Result<(), Failure> {
        let mut fence: Map<Shape, ShapeMapHasher> = Map::with_hasher(ShapeMapHasher, 1);
        let builder = BRepBuilder::default();
        let arguments: Vec<Shape> = self.arguments.iter().cloned().collect();
        for argument in arguments {
            if argument.shape_type()? != kind {
                continue;
            }
            let images: Vec<Shape> = match self.images.seek(&argument) {
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

/// Runs the pipeline; a failure becomes an `AlertBuilderFailed` error, as the native catch
/// of Standard_Failure does (`PerformInternal`).
pub(crate) fn perform_internal<H: BuilderHooks>(hooks: &mut H, filler: &mut BOPAlgoPaveFiller) {
    hooks.general_fuse().options().clear();
    if perform_internal1(hooks, filler).is_err() {
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
    hooks.general_fuse().prepare();
    let Some(structure) = filler.data_structure_mut() else {
        return Err(no_structure());
    };
    let stages: [(Stage<BOPAlgoBuilder>, ShapeEnum); 8] = [
        (BOPAlgoBuilder::stage_vertices, ShapeEnum::Vertex),
        (BOPAlgoBuilder::stage_edges, ShapeEnum::Edge),
        (BOPAlgoBuilder::stage_wires, ShapeEnum::Wire),
        (BOPAlgoBuilder::stage_faces, ShapeEnum::Face),
        (BOPAlgoBuilder::stage_shells, ShapeEnum::Shell),
        (BOPAlgoBuilder::stage_solids, ShapeEnum::Solid),
        (BOPAlgoBuilder::stage_compsolids, ShapeEnum::CompSolid),
        (BOPAlgoBuilder::stage_compounds, ShapeEnum::Compound),
    ];
    for (stage, kind) in stages {
        stage(hooks.general_fuse(), structure)?;
        if hooks.general_fuse().options().has_errors() {
            return Ok(());
        }
        hooks.build_result(kind)?;
        if hooks.general_fuse().options().has_errors() {
            return Ok(());
        }
    }
    let builder = hooks.general_fuse();
    builder.prepare_history(structure)?;
    if builder.options().has_errors() {
        return Ok(());
    }
    builder.post_treat(structure)
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

    fn stage_solids(&mut self, structure: &mut DataStructure) -> Result<(), Failure> {
        self.fill_images_solids(structure)
    }

    fn stage_compsolids(&mut self, structure: &mut DataStructure) -> Result<(), Failure> {
        self.fill_images_containers(ShapeEnum::CompSolid, structure)
    }

    fn stage_compounds(&mut self, structure: &mut DataStructure) -> Result<(), Failure> {
        self.fill_images_compounds(structure)
    }
}
