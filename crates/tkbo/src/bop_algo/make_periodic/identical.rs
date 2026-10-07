//! `BOPAlgo_MakePeriodic::Perform`: the check, the trim and the identical opposite sides.
use super::{BOPAlgoMakePeriodic, TwinMap};
use crate::bop_algo::alerts::alert;
use crate::bop_algo::{
    AlertNoPeriodicityRequired, AlertUnableToMakeIdentical, AlertUnableToTrim, BOPAlgoPaveFiller,
    Glue,
};
use crate::brep_algo_api::{BRepAlgoAPIBuilderAlgo, BRepAlgoAPICommon, BRepAlgoAPISplitter};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_tools::History;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Compound, Shape};
use rustycad_tkbrep::toptools::{IndexedMapOfShape, ShapeMapHasher};
use rustycad_tkernel::message::AlertHandle;
use rustycad_tkernel::ncollection::{IndexedDataMap, List};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{self, Direction3, Transform3, Vector3};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPITransform};
use std::sync::Arc;

/// The periodic and repetition directions X, Y, Z (`MY_DIRECTIONS`).
pub(super) fn direction(slot: usize) -> Direction3 {
    [gp::dx(), gp::dy(), gp::dz()][slot]
}

/// The translations of the opposite sides, by sub-shape (`aTranslationHistMap`).
type TranslationHistory = IndexedDataMap<Shape, List<Shape>, ShapeMapHasher>;

impl BOPAlgoMakePeriodic {
    /// Checks a periodic direction is requested, trims the shape to the periods where the
    /// input is not trimmed, then makes the opposite sides identical (`Perform`);
    /// `maker.perform()?`.
    pub fn perform(&mut self) -> Result<(), Failure> {
        self.check_data();
        if self.options.has_errors() {
            return Ok(());
        }
        self.trim()?;
        if self.options.has_errors() {
            return Ok(());
        }
        self.make_identical()
    }

    /// An error when no direction is periodic with a period of at least the confusion
    /// (`CheckData`).
    fn check_data(&self) {
        let unusable = |direction: i32| {
            !self.is_periodic(direction) || self.period(direction) < precision::confusion()
        };
        if unusable(0) && unusable(1) && unusable(2) {
            self.options.add_error(&alert(AlertNoPeriodicityRequired));
        }
    }

    /// The common of the shape with its enlarged bounding box, the box bounded by the period
    /// in the untrimmed directions (`Trim`).
    fn trim(&mut self) -> Result<(), Failure> {
        if self.is_input_x_trimmed() && self.is_input_y_trimmed() && self.is_input_z_trimmed() {
            return Ok(());
        }
        let trim_box = self.trimming_box()?;
        let arguments: List<Shape> = [self.input_shape.clone()].into_iter().collect();
        let mut common = BRepAlgoAPICommon::new();
        common.set_arguments(&arguments);
        common.set_tools(&[trim_box.clone()].into_iter().collect());
        common.set_run_parallel(self.options.run_parallel());
        common.build()?;
        if common.has_errors() {
            self.options.report().merge(common.get_report());
            let shapes = compound_of(&[self.input_shape.clone(), trim_box])?;
            self.options
                .add_error(&(Arc::new(AlertUnableToTrim::new(&shapes)) as AlertHandle));
            return Ok(());
        }
        self.shape = common.shape().clone();
        let history = History::new();
        let algorithm: &mut BRepAlgoAPIBuilderAlgo = &mut common;
        history.merge_algorithm(std::slice::from_ref(&self.input_shape), algorithm)?;
        self.split_history = Handle::new(history);
        Ok(())
    }

    /// The bounding box of the input enlarged by a tenth of its diagonal, set to the period
    /// in each untrimmed direction.
    fn trimming_box(&self) -> Result<Shape, Failure> {
        let mut bounds = BoundingBox3::default();
        BRepBndLib::add(&self.input_shape, &mut bounds, true)?;
        bounds.enlarge(0.1 * bounds.square_extent().sqrt());
        let (mut minimum, mut maximum) = (bounds.corner_min()?, bounds.corner_max()?);
        for direction in 0..3 {
            if self.is_input_trimmed(direction) {
                continue;
            }
            let first = self.period_first(direction);
            minimum.set_coord(direction + 1, first)?;
            maximum.set_coord(direction + 1, first + self.period(direction))?;
        }
        Ok(BRepPrimAPIMakeBox::from_points(minimum, maximum)?
            .solid()?
            .into_shape())
    }

    /// Splits the negative sides by the positive ones, then the positive sides by the
    /// negative ones recording the twins; the history is the split history (`MakeIdentical`).
    fn make_identical(&mut self) -> Result<(), Failure> {
        if self.shape.is_null() {
            self.shape = self.input_shape.clone();
        }
        if self.split_history.is_null() {
            self.split_history = Handle::new(History::new());
        }
        self.split_negative()?;
        if self.options.has_errors() {
            return Ok(());
        }
        self.split_positive()?;
        let history = History::new();
        history.merge_handle(&self.split_history)?;
        self.history = Handle::new(history);
        Ok(())
    }

    /// Splits the shape by its copy translated one period back, direction by direction
    /// (`SplitNegative`).
    fn split_negative(&mut self) -> Result<(), Failure> {
        for slot in 0..3 {
            if !self.is_periodic(slot as i32) {
                continue;
            }
            let offset = Vector3::from(direction(slot).reversed()).scaled(self.period(slot as i32));
            let translated = translate(&self.shape, offset)?.0;
            let history = self.split_history.clone();
            self.split_shape(&[translated].into_iter().collect(), history.get(), None)?;
        }
        Ok(())
    }

    /// Splits the shape by its copies translated one period forward in every periodic
    /// direction, and makes twins of each sub-shape and the splits of its translations
    /// (`SplitPositive`).
    fn split_positive(&mut self) -> Result<(), Failure> {
        let mut sub_shapes = IndexedMapOfShape::default();
        topexp::map_shapes(&self.shape, &mut sub_shapes, true, true);
        let mut tools = List::default();
        let mut translations: TranslationHistory = IndexedDataMap::with_hasher(ShapeMapHasher, 1);
        for slot in 0..3 {
            if !self.is_periodic(slot as i32) {
                continue;
            }
            let offset = Vector3::from(direction(slot)).scaled(self.period(slot as i32));
            let (translated, transform) = translate(&self.shape, offset)?;
            tools.append(translated);
            record_translations(&sub_shapes, &transform, &mut translations)?;
        }
        let (shape_history, tools_history) = (History::new(), History::new());
        self.split_shape(&tools, Some(&shape_history), Some(&tools_history))?;
        if self.options.has_errors() {
            return Ok(());
        }
        if let Some(history) = self.split_history.get() {
            history.merge(&shape_history)?;
        }
        add_opposite_twins(
            &translations,
            &shape_history,
            &tools_history,
            &mut self.twins,
        )
    }

    /// Splits the shape by `tools` given first to the intersection, so the coinciding parts
    /// take the geometry of the tools; the splits of the shape and of the tools go to the
    /// given histories (`SplitShape`).
    fn split_shape(
        &mut self,
        tools: &List<Shape>,
        shape_history: Option<&History>,
        tools_history: Option<&History>,
    ) -> Result<(), Failure> {
        let Some(filler) = self.intersect_with(tools)? else {
            return Ok(());
        };
        let arguments: List<Shape> = [self.shape.clone()].into_iter().collect();
        let mut splitter = BRepAlgoAPISplitter::with_filler(filler);
        splitter.set_arguments(&arguments);
        splitter.set_tools(tools);
        splitter.set_glue(Glue::Shift);
        splitter.set_run_parallel(self.options.run_parallel());
        splitter.build()?;
        if splitter.has_errors() {
            self.options.report().merge(splitter.get_report());
            return self.unable_to_make_identical(tools);
        }
        let previous = self.shape.clone();
        self.shape = splitter.shape().clone();
        let algorithm: &mut BRepAlgoAPIBuilderAlgo = &mut splitter;
        if let Some(history) = shape_history {
            history.merge_algorithm(&[previous], algorithm)?;
        }
        if let Some(history) = tools_history {
            let tools: Vec<Shape> = tools.iter().cloned().collect();
            history.merge_algorithm(&tools, algorithm)?;
        }
        Ok(())
    }

    /// The performed pave filler of the tools then the shape, gluing by shift and keeping
    /// the inputs; None after `AlertUnableToMakeIdentical`.
    fn intersect_with(
        &mut self,
        tools: &List<Shape>,
    ) -> Result<Option<BOPAlgoPaveFiller>, Failure> {
        let mut filler = BOPAlgoPaveFiller::new();
        filler.set_arguments(tools);
        filler.add_argument(&self.shape);
        filler.set_glue(Glue::Shift);
        filler.set_non_destructive(true);
        filler
            .options_mut()
            .set_run_parallel(self.options.run_parallel());
        filler.perform();
        if filler.options().has_errors() {
            self.options.report().merge(filler.options().report());
            self.unable_to_make_identical(tools)?;
            return Ok(None);
        }
        Ok(Some(filler))
    }

    /// The error naming the tools and the shape that could not be split.
    fn unable_to_make_identical(&mut self, tools: &List<Shape>) -> Result<(), Failure> {
        let mut shapes: Vec<Shape> = tools.iter().cloned().collect();
        shapes.push(self.shape.clone());
        let alert: AlertHandle = Arc::new(AlertUnableToMakeIdentical::new(&compound_of(&shapes)?));
        self.options.add_error(&alert);
        Ok(())
    }
}

/// The shape moved by `offset` (geometry not copied), with the transformation tool.
pub(super) fn translate(
    shape: &Shape,
    offset: Vector3,
) -> Result<(Shape, BRepBuilderAPITransform), Failure> {
    let mut transform = Transform3::default();
    transform.set_translation_part(offset);
    let mut tool = BRepBuilderAPITransform::with_shape(shape, transform, false, false)?;
    let moved = tool.shape()?.clone();
    Ok((moved, tool))
}

/// Appends the translated image of each supported sub-shape to its translations.
fn record_translations(
    sub_shapes: &IndexedMapOfShape,
    transform: &BRepBuilderAPITransform,
    translations: &mut TranslationHistory,
) -> Result<(), Failure> {
    for position in 1..=sub_shapes.extent() {
        let shape = sub_shapes.find_key(position)?;
        if !History::is_supported_type(shape)? {
            continue;
        }
        let image = transform.modified_shape(shape)?;
        match translations.change_seek(shape) {
            Some(images) => {
                images.append(image);
            }
            None => {
                translations.add(shape.clone(), [image].into_iter().collect());
            }
        }
    }
    Ok(())
}

/// Each sub-shape (or its single split) and the splits of its translations are twins; a
/// sub-shape split in several parts has none.
fn add_opposite_twins(
    translations: &TranslationHistory,
    shapes: &History,
    tools: &History,
    twins: &mut TwinMap,
) -> Result<(), Failure> {
    for position in 1..=translations.extent() {
        let mut shape = translations.find_key(position)?.clone();
        let splits = shapes.modified(&shape)?;
        match splits.len() {
            0 => {}
            1 => shape = splits[0].clone(),
            _ => continue,
        }
        for translated in translations.find_from_index(position)?.iter() {
            for split in tools.modified(translated)? {
                add_twin(&shape, &split, twins);
                add_twin(&split, &shape, twins);
            }
        }
    }
    Ok(())
}

/// Adds `twin` to the twins of `shape` unless already there (`AddTwin`).
pub(super) fn add_twin(shape: &Shape, twin: &Shape, twins: &mut TwinMap) {
    let Some(list) = twins.change_seek(shape) else {
        twins
            .bound(shape.clone(), List::default())
            .append(twin.clone());
        return;
    };
    if !list.iter().any(|known| twin.is_same(known)) {
        list.append(twin.clone());
    }
}

/// A compound of the shapes (`AddToShape`).
pub(super) fn compound_of(shapes: &[Shape]) -> Result<Shape, Failure> {
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    for shape in shapes {
        builder.add(&mut compound, shape)?;
    }
    Ok(compound.into_shape())
}
