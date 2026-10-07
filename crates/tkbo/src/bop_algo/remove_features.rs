//! `BOPAlgo_RemoveFeatures`: removes faces (features) from solids by extending the
//! adjacent faces over the gap and rebuilding the solids.
#[path = "remove_features/feature.rs"]
mod feature;
#[path = "remove_features/fill_gap.rs"]
mod fill_gap;
#[path = "remove_features/helpers.rs"]
mod helpers;
#[path = "remove_features/solids.rs"]
mod solids;
use super::alerts::alert;
use super::{
    AlertEmptyShape, AlertNoFacesToRemove, AlertRemoveFeaturesFailed, AlertTooFewArguments,
    AlertUnsupportedType, BuilderShape,
};
use crate::bop_tools::{make_connexity_blocks, treat_compound};
use helpers::{compound_of, explore, make_removed};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_tools::History;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{CompSolid, Shape};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::message::AlertHandle;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkshhealing::shape_upgrade::ShapeUpgradeUnifySameDomain;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

/// One step of `Perform`.
type RemovalStep = fn(&mut BOPAlgoRemoveFeatures) -> Result<(), Failure>;

/// The feature-removal algorithm: each connected block of the faces to remove is a feature,
/// whose gap is filled by the extended adjacent faces before the solids are rebuilt; the
/// history and the simplification of the result follow.
///
/// ```
/// use rustycad_tkbo::bop_algo::BOPAlgoRemoveFeatures;
/// let mut remover = BOPAlgoRemoveFeatures::new();
/// assert!(remover.input_shape().is_null() && remover.faces_to_remove().is_empty());
/// ```
#[derive(Default)]
pub struct BOPAlgoRemoveFeatures {
    base: BuilderShape,
    input_shape: Shape,
    faces_to_remove: List<Shape>,
    /// The connected blocks of faces to remove (`myFeatures`).
    features: Vec<Shape>,
    /// Every sub-shape of the input shape (`myInputsMap`).
    inputs_map: IndexedMapOfShape,
}

impl Deref for BOPAlgoRemoveFeatures {
    type Target = BuilderShape;
    fn deref(&self) -> &BuilderShape {
        &self.base
    }
}

impl DerefMut for BOPAlgoRemoveFeatures {
    fn deref_mut(&mut self) -> &mut BuilderShape {
        &mut self.base
    }
}

impl BOPAlgoRemoveFeatures {
    /// An empty algorithm (`BOPAlgo_RemoveFeatures()`); `BOPAlgoRemoveFeatures::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the shape to remove the features from (`SetShape`); `remover.set_shape(&solid)`.
    pub fn set_shape(&mut self, shape: &Shape) {
        self.input_shape = shape.clone();
    }

    /// The shape to remove the features from (`InputShape`); `remover.input_shape()`.
    pub fn input_shape(&self) -> &Shape {
        &self.input_shape
    }

    /// Adds a face (or a shape with faces) to remove (`AddFaceToRemove`);
    /// `remover.add_face_to_remove(&face)`.
    pub fn add_face_to_remove(&mut self, face: &Shape) {
        self.faces_to_remove.append(face.clone());
    }

    /// Adds faces to remove (`AddFacesToRemove`); `remover.add_faces_to_remove(&faces)`.
    pub fn add_faces_to_remove(&mut self, faces: &List<Shape>) {
        for face in faces.iter() {
            self.faces_to_remove.append(face.clone());
        }
    }

    /// The faces to remove (`FacesToRemove`); `remover.faces_to_remove().extent()`.
    pub fn faces_to_remove(&self) -> &List<Shape> {
        &self.faces_to_remove
    }

    /// Forgets the inputs, the result and the history (`Clear`); `remover.clear()`.
    pub fn clear(&mut self) {
        self.base.clear();
        self.input_shape.nullify();
        self.base.shape.nullify();
        self.faces_to_remove.clear();
        self.features.clear();
        self.inputs_map.clear(false);
    }

    /// Removes the features: checks the input, finds the features, fills their gaps, then
    /// updates the history, simplifies the result and restores the input type (`Perform`).
    /// A failure of a step is the error `AlertRemoveFeaturesFailed`, as the native catch
    /// does; there is no Message_ProgressRange. `remover.perform()`.
    pub fn perform(&mut self) {
        if self.base.fill_history {
            self.base.history = Handle::new(History::new());
        }
        if self.perform_steps().is_err() {
            self.base
                .options
                .add_error(&alert(AlertRemoveFeaturesFailed));
        }
    }

    fn perform_steps(&mut self) -> Result<(), Failure> {
        let steps: [RemovalStep; 6] = [
            Self::check_data,
            Self::prepare_features,
            Self::remove_features,
            Self::update_history,
            Self::simplify_result,
            Self::post_treat,
        ];
        for step in steps {
            step(self)?;
            if self.base.options.has_errors() {
                break;
            }
        }
        Ok(())
    }

    /// Accepts a solid, a compsolid, or a compound with solids, whose other shapes only
    /// warn and are removed from the history (`CheckData`).
    fn check_data(&mut self) -> Result<(), Failure> {
        self.base.shape = self.input_shape.clone();
        match self.input_shape.shape_type()? {
            ShapeEnum::Solid | ShapeEnum::CompSolid => Ok(()),
            ShapeEnum::Compound => self.check_compound(),
            _ => {
                self.base.options.add_error(&alert(AlertTooFewArguments));
                Ok(())
            }
        }
    }

    fn check_compound(&mut self) -> Result<(), Failure> {
        let (mut shapes, mut fence) = (Vec::new(), MapOfShape::default());
        treat_compound(&self.input_shape, &mut shapes, Some(&mut fence))?;
        if shapes.is_empty() {
            let alert: AlertHandle = Arc::new(AlertEmptyShape::new(&self.input_shape));
            self.base.options.add_error(&alert);
            return Ok(());
        }
        let mut solids = Vec::new();
        let mut others = Vec::new();
        for shape in shapes {
            match shape.shape_type()? {
                ShapeEnum::Solid | ShapeEnum::CompSolid => solids.push(shape),
                _ => others.push(shape),
            }
        }
        if solids.is_empty() {
            self.base.options.add_error(&alert(AlertTooFewArguments));
            return Ok(());
        }
        if others.is_empty() {
            return Ok(());
        }
        for other in &others {
            let alert: AlertHandle = Arc::new(AlertUnsupportedType::new(other));
            self.base.options.add_warning(&alert);
        }
        self.base.shape = compound_of(&solids)?;
        if let Some(history) = self.filled_history().get() {
            make_removed(&others, history, &IndexedMapOfShape::default())?;
        }
        Ok(())
    }

    /// The input faces requested for removal, grouped into connected blocks (the features);
    /// an error when none belongs to the input (`PrepareFeatures`).
    fn prepare_features(&mut self) -> Result<(), Failure> {
        topexp::map_shapes(&self.input_shape, &mut self.inputs_map, true, true);
        let mut faces = Vec::new();
        for shape in self.faces_to_remove.iter() {
            for face in explore(shape, ShapeEnum::Face)? {
                if self.inputs_map.contains(&face) {
                    faces.push(face);
                }
            }
        }
        if faces.is_empty() {
            self.base.options.add_error(&alert(AlertNoFacesToRemove));
            return Ok(());
        }
        let mut ancestors = IndexedDataMapOfShapeListOfShape::default();
        let mut blocks = Vec::new();
        let compound = compound_of(&faces)?;
        make_connexity_blocks(
            &compound,
            ShapeEnum::Edge,
            ShapeEnum::Face,
            &mut blocks,
            &mut ancestors,
        )?;
        self.features = blocks
            .iter()
            .map(|block| compound_of(block))
            .collect::<Result<_, _>>()?;
        Ok(())
    }

    /// The history of the input sub-shapes without trace in the result made removed
    /// (`UpdateHistory`).
    fn update_history(&mut self) -> Result<(), Failure> {
        let handle = self.filled_history();
        let Some(history) = handle.get() else {
            return Ok(());
        };
        self.base.map_shape.clear(false);
        let mut result_shapes = IndexedMapOfShape::default();
        topexp::map_shapes(&self.base.shape, &mut result_shapes, true, true);
        for shape in result_shapes.iter() {
            self.base.map_shape.add(shape.clone());
        }
        let removals = History::new();
        for shape in self.inputs_map.iter() {
            if !History::is_supported_type(shape)? || history.is_removed(shape)? {
                continue;
            }
            let images = history.modified(shape)?;
            if images.is_empty() && !self.base.map_shape.contains(shape) {
                removals.remove(shape)?;
            }
            for image in images
                .iter()
                .filter(|image| !self.base.map_shape.contains(image))
            {
                removals.remove(image)?;
            }
        }
        history.merge(&removals)
    }

    /// Unifies the faces and edges of the rebuilt result keeping the input sub-shapes it
    /// still holds (`SimplifyResult`).
    fn simplify_result(&mut self) -> Result<(), Failure> {
        if self.base.shape.is_same(&self.input_shape) {
            return Ok(());
        }
        let unifier = ShapeUpgradeUnifySameDomain::empty();
        unifier.initialize(&self.base.shape, true, true, false)?;
        unifier.allow_internal_edges(false);
        if self.base.map_shape.is_empty() {
            let mut result_shapes = IndexedMapOfShape::default();
            topexp::map_shapes(&self.base.shape, &mut result_shapes, true, true);
            for shape in result_shapes.iter() {
                self.base.map_shape.add(shape.clone());
            }
        }
        for input in self.inputs_map.iter() {
            if self.base.map_shape.contains(input) {
                unifier.keep_shape(input)?;
            }
        }
        unifier.build()?;
        self.base.shape = unifier.shape();
        if let Some(history) = self.filled_history().get() {
            history.merge_handle(&unifier.history())?;
        }
        Ok(())
    }

    /// Restores the type of the input: its solid, or a compound or compsolid of the result
    /// solids (`PostTreat`).
    fn post_treat(&mut self) -> Result<(), Failure> {
        let input_type = self.input_shape.shape_type()?;
        if self.base.shape.shape_type()? == input_type {
            return Ok(());
        }
        let solids = explore(&self.base.shape, ShapeEnum::Solid)?;
        if input_type == ShapeEnum::Solid {
            // TopExp_Explorer::Current raises Standard_NoSuchObject on an exhausted explorer.
            self.base.shape = solids.first().cloned().ok_or_else(no_solid)?;
            return Ok(());
        }
        if input_type == ShapeEnum::Compound {
            self.base.shape = compound_of(&solids)?;
            return Ok(());
        }
        let builder = BRepBuilder::default();
        let mut compsolid = CompSolid::default();
        builder.make_comp_solid(&mut compsolid);
        let mut result = compsolid.into_shape();
        for solid in &solids {
            builder.add(&mut result, solid)?;
        }
        self.base.shape = result;
        Ok(())
    }

    /// The history handle while history filling is on (`HasHistory()` and `myHistory`); a
    /// null handle otherwise.
    fn filled_history(&self) -> Handle<History> {
        if self.base.fill_history {
            self.base.history.clone()
        } else {
            Handle::default()
        }
    }
}

/// The failure of the post-treatment of a result without solid.
fn no_solid() -> Failure {
    Failure::with_type(
        "Standard_NoSuchObject",
        "BOPAlgo_RemoveFeatures::PostTreat: the result has no solid; expected one for a solid input",
        "",
    )
}
