//! Free-boundary measurements from OCCT ShapeAnalysis_FreeBoundsProperties.
use super::{ShapeAnalysisFreeBoundData, ShapeAnalysisFreeBounds};
use crate::shape_extend::ShapeExtendExplorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::{
    ncollection::HSequence,
    standard::{Failure, Handle},
};
mod accessors;
mod contours;
mod notches;
/// Shared property records; `analyzer.closed_free_bounds().sequence(|s|s.len())`.
pub type FreeBoundPropertySequence = Handle<HSequence<Handle<ShapeAnalysisFreeBoundData>>>;
/// Measure free contours without clearing earlier results; `analyzer.perform()?`.
pub struct ShapeAnalysisFreeBoundsProperties {
    shape: Shape,
    tolerance: f64,
    split_closed: bool,
    split_open: bool,
    closed: FreeBoundPropertySequence,
    open: FreeBoundPropertySequence,
}
impl Default for ShapeAnalysisFreeBoundsProperties {
    fn default() -> Self {
        Self {
            shape: Shape::default(),
            tolerance: 0.,
            split_closed: false,
            split_open: false,
            closed: Handle::new(HSequence::default()),
            open: Handle::new(HSequence::default()),
        }
    }
}
impl ShapeAnalysisFreeBoundsProperties {
    /// Construct empty result sequences; `ShapeAnalysisFreeBoundsProperties::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Configure actual free boundaries; `ShapeAnalysisFreeBoundsProperties::with_shape(&shape,false,false)`.
    pub fn with_shape(shape: &Shape, split_closed: bool, split_open: bool) -> Self {
        let mut result = Self::new();
        result.init(shape, split_closed, split_open);
        result
    }
    /// Configure tolerance-based forecast boundaries; `ShapeAnalysisFreeBoundsProperties::with_tolerance(&shape,1e-6,false,false)`.
    pub fn with_tolerance(
        shape: &Shape,
        tolerance: f64,
        split_closed: bool,
        split_open: bool,
    ) -> Self {
        let mut result = Self::new();
        result.init_with_tolerance(shape, tolerance, split_closed, split_open);
        result
    }
    /// Replace the source and flags, retaining tolerance and results; `analyzer.init(&shape,false,false)`.
    pub fn init(&mut self, shape: &Shape, split_closed: bool, split_open: bool) {
        self.shape = shape.clone();
        self.split_closed = split_closed;
        self.split_open = split_open;
    }
    /// Replace source, flags and tolerance without clearing results; `analyzer.init_with_tolerance(&shape,1e-6,false,false)`.
    pub fn init_with_tolerance(
        &mut self,
        shape: &Shape,
        tolerance: f64,
        split_closed: bool,
        split_open: bool,
    ) {
        self.init(shape, split_closed, split_open);
        self.tolerance = tolerance;
    }
    /// Dispatch, inspect notches, then contours without short-circuiting; `analyzer.perform()?`.
    pub fn perform(&self) -> Result<bool, Failure> {
        let mut result = self.dispatch_bounds()?;
        result |= self.check_notches(0.)?;
        result |= self.check_contours(0.)?;
        Ok(result)
    }
    /// Append current free contours to the shared result sequences; `analyzer.dispatch_bounds()?`.
    pub fn dispatch_bounds(&self) -> Result<bool, Failure> {
        if !self.is_loaded() {
            return Ok(false);
        }
        let bounds = if self.tolerance > 0. {
            ShapeAnalysisFreeBounds::forecast(
                &self.shape,
                self.tolerance,
                self.split_closed,
                self.split_open,
            )?
        } else {
            ShapeAnalysisFreeBounds::from_shape(
                &self.shape,
                self.split_closed,
                self.split_open,
                false,
            )?
        };
        append_bounds(bounds.closed_wires(), &self.closed)?;
        append_bounds(bounds.open_wires(), &self.open)?;
        Ok(true)
    }
}
fn append_bounds(compound: &Shape, output: &FreeBoundPropertySequence) -> Result<(), Failure> {
    let sequence = ShapeExtendExplorer::new().seq_from_compound(compound, false)?;
    let wires = sequence.sequence(|s| s.iter().cloned().collect::<Vec<_>>());
    for shape in wires {
        output.append(Handle::new(ShapeAnalysisFreeBoundData::with_bound(
            &topods::wire(&shape)?,
        )));
    }
    Ok(())
}
fn property_records(
    sequence: &FreeBoundPropertySequence,
) -> Vec<Handle<ShapeAnalysisFreeBoundData>> {
    sequence.sequence(|s| s.iter().cloned().collect())
}
fn property_record(
    properties: &Handle<ShapeAnalysisFreeBoundData>,
) -> Result<&ShapeAnalysisFreeBoundData, Failure> {
    properties.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "free-bound property record is null; expected allocated ShapeAnalysis_FreeBoundData",
            "",
        )
    })
}
