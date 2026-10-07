//! `ShapeUpgrade_UnifySameDomain::FillHistory`: the history of the context filtered to
//! the vertices, edges, faces and solids of the input and the result.
use super::UnifyState;
use rustycad_tkbrep::brep_tools::{History, ReShape};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;

const HISTORY_KINDS: [ShapeEnum; 4] = [
    ShapeEnum::Vertex,
    ShapeEnum::Edge,
    ShapeEnum::Face,
    ShapeEnum::Solid,
];

/// The vertices, edges, faces and solids of `shape`, in that order.
fn history_shapes(shape: &Shape) -> Result<IndexedMapOfShape, Failure> {
    let mut map = IndexedMapOfShape::default();
    for kind in HISTORY_KINDS {
        topexp::map_shapes_of_type(shape, kind, &mut map)?;
    }
    Ok(map)
}

impl UnifyState {
    /// `FillHistory()`: nothing when the history is null; else every input shape absent
    /// from the result is recorded as modified into its images present in the result, or
    /// removed when it has none.
    pub(super) fn fill_history(&self) -> Result<(), Failure> {
        let Some(history) = self.history.get() else {
            return Ok(());
        };
        let context_history = self.context()?.history()?;
        let context_history = context_history.get().ok_or_else(null_context_history)?;
        let usd_history = Handle::new(History::new());
        let inputs = history_shapes(&self.init_shape)?;
        let results = history_shapes(&self.shape)?;
        for index in 1..=inputs.extent() {
            let shape = inputs.find_key(index)?;
            if !results.contains(shape) {
                record_input(shape, context_history, &results, &usd_history)?;
            }
        }
        history.merge_handle(&usd_history)
    }
}

/// The modified images of `shape` present in the result, or its removal.
fn record_input(
    shape: &Shape,
    context_history: &History,
    results: &IndexedMapOfShape,
    usd_history: &Handle<History>,
) -> Result<(), Failure> {
    let usd = usd_history.get().ok_or_else(null_context_history)?;
    let images = context_history.modified(shape)?;
    let mut removed = true;
    for image in images.iter().filter(|image| results.contains(image)) {
        if !image.is_same(shape) {
            usd.add_modified(shape, image)?;
        }
        removed = false;
    }
    if removed {
        usd.remove(shape)?;
    }
    Ok(())
}

fn null_context_history() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "ShapeUpgrade_UnifySameDomain::FillHistory: null history; expected a BRepTools_History",
        "",
    )
}
