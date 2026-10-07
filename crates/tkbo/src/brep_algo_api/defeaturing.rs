//! `BRepAlgoAPI_Defeaturing`: the API of the feature removal (`BOPAlgo_RemoveFeatures`).
use super::algo::BRepAlgoAPIAlgo;
use crate::bop_algo::BOPAlgoRemoveFeatures;
use rustycad_tkbrep::brep_tools::History;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use std::ops::{Deref, DerefMut};

/// Removes faces (features) from a solid and fills their gaps with the extended adjacent
/// faces, keeping the history of the input.
///
/// ```
/// use rustycad_tkbo::brep_algo_api::BRepAlgoAPIDefeaturing;
/// let defeaturing = BRepAlgoAPIDefeaturing::new();
/// assert!(defeaturing.has_history() && defeaturing.faces_to_remove().is_empty());
/// ```
pub struct BRepAlgoAPIDefeaturing {
    algo: BRepAlgoAPIAlgo,
    input_shape: Shape,
    faces_to_remove: List<Shape>,
    fill_history: bool,
    feature_removal_tool: BOPAlgoRemoveFeatures,
}

impl Default for BRepAlgoAPIDefeaturing {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for BRepAlgoAPIDefeaturing {
    type Target = BRepAlgoAPIAlgo;
    fn deref(&self) -> &BRepAlgoAPIAlgo {
        &self.algo
    }
}

impl DerefMut for BRepAlgoAPIDefeaturing {
    fn deref_mut(&mut self) -> &mut BRepAlgoAPIAlgo {
        &mut self.algo
    }
}

impl BRepAlgoAPIDefeaturing {
    /// An empty operation filling the history (`BRepAlgoAPI_Defeaturing()`).
    /// `BRepAlgoAPIDefeaturing::new()`.
    pub fn new() -> Self {
        Self {
            algo: BRepAlgoAPIAlgo::new(),
            input_shape: Shape::default(),
            faces_to_remove: List::default(),
            fill_history: true,
            feature_removal_tool: BOPAlgoRemoveFeatures::new(),
        }
    }

    /// Sets the shape to remove the features from (`SetShape`); `api.set_shape(&solid)`.
    pub fn set_shape(&mut self, shape: &Shape) {
        self.input_shape = shape.clone();
    }

    /// The shape to remove the features from (`InputShape`); `api.input_shape()`.
    pub fn input_shape(&self) -> &Shape {
        &self.input_shape
    }

    /// Adds a face to remove (`AddFaceToRemove`); `api.add_face_to_remove(&face)`.
    pub fn add_face_to_remove(&mut self, face: &Shape) {
        self.faces_to_remove.append(face.clone());
    }

    /// Adds faces to remove (`AddFacesToRemove`); `api.add_faces_to_remove(&faces)`.
    pub fn add_faces_to_remove(&mut self, faces: &List<Shape>) {
        for face in faces.iter() {
            self.faces_to_remove.append(face.clone());
        }
    }

    /// The faces to remove (`FacesToRemove`); `api.faces_to_remove()`.
    pub fn faces_to_remove(&self) -> &List<Shape> {
        &self.faces_to_remove
    }

    /// Turns history filling on or off (`SetToFillHistory`); `api.set_to_fill_history(false)`.
    pub fn set_to_fill_history(&mut self, fill: bool) {
        self.fill_history = fill;
    }

    /// Whether the history is filled (`HasHistory`); `api.has_history()`.
    pub fn has_history(&self) -> bool {
        self.fill_history
    }

    /// Removes the features with a fresh BOPAlgo_RemoveFeatures, merges its report and
    /// takes its result when it has no error (`Build`; there is no Message_ProgressRange);
    /// `api.build()`.
    pub fn build(&mut self) {
        self.algo.base.set_not_done();
        self.clear();
        let tool = &mut self.feature_removal_tool;
        tool.set_shape(&self.input_shape);
        tool.add_faces_to_remove(&self.faces_to_remove);
        tool.set_to_fill_history(self.fill_history);
        tool.options_mut()
            .set_run_parallel(self.algo.run_parallel());
        tool.perform();
        self.algo.get_report().merge(tool.options().report());
        if self.algo.has_errors() {
            return;
        }
        self.algo.base.set_done();
        self.algo.base.my_shape = tool.shape().clone();
    }

    /// Clears the report and the removal tool (`Clear`).
    fn clear(&mut self) {
        self.algo.clear();
        self.feature_removal_tool.clear();
    }

    /// The images of `shape` in the result (`Modified`); `api.modified(&face)?`.
    pub fn modified(&self, shape: &Shape) -> Result<Vec<Shape>, Failure> {
        self.feature_removal_tool.modified(shape)
    }

    /// The shapes generated from `shape` (`Generated`); `api.generated(&edge)?`.
    pub fn generated(&self, shape: &Shape) -> Result<Vec<Shape>, Failure> {
        self.feature_removal_tool.generated(shape)
    }

    /// Whether `shape` has no trace in the result (`IsDeleted`); `api.is_deleted(&face)?`.
    pub fn is_deleted(&self, shape: &Shape) -> Result<bool, Failure> {
        self.feature_removal_tool.is_deleted(shape)
    }

    /// Whether a shape was modified (`HasModified`); `api.has_modified()`.
    pub fn has_modified(&self) -> bool {
        self.feature_removal_tool.has_modified()
    }

    /// Whether a shape generated others (`HasGenerated`); `api.has_generated()`.
    pub fn has_generated(&self) -> bool {
        self.feature_removal_tool.has_generated()
    }

    /// Whether a shape was deleted (`HasDeleted`); `api.has_deleted()`.
    pub fn has_deleted(&self) -> bool {
        self.feature_removal_tool.has_deleted()
    }

    /// The history of the removal while it is filled (`History`); `api.history()`.
    pub fn history(&mut self) -> Option<Handle<History>> {
        self.feature_removal_tool.history()
    }
}
