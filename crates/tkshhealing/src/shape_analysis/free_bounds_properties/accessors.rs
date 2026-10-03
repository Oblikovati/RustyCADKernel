use super::{
    FreeBoundPropertySequence, ShapeAnalysisFreeBoundData,
    ShapeAnalysisFreeBoundsProperties as Properties,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Failure, Handle};
impl Properties {
    /// Query source presence; `analyzer.is_loaded()`.
    pub fn is_loaded(&self) -> bool {
        !self.shape.is_null()
    }
    /// Return the source shape; `analyzer.shape()`.
    pub fn shape(&self) -> Shape {
        self.shape.clone()
    }
    /// Return retained tolerance; `analyzer.tolerance()`.
    pub fn tolerance(&self) -> f64 {
        self.tolerance
    }
    /// Count both accumulated sequences; `analyzer.nb_free_bounds()`.
    pub fn nb_free_bounds(&self) -> i32 {
        self.nb_closed_free_bounds() + self.nb_open_free_bounds()
    }
    /// Count closed boundaries; `analyzer.nb_closed_free_bounds()`.
    pub fn nb_closed_free_bounds(&self) -> i32 {
        self.closed.sequence(|s| s.length())
    }
    /// Count open boundaries; `analyzer.nb_open_free_bounds()`.
    pub fn nb_open_free_bounds(&self) -> i32 {
        self.open.sequence(|s| s.length())
    }
    /// Share the closed boundary sequence; `analyzer.closed_free_bounds()`.
    pub fn closed_free_bounds(&self) -> FreeBoundPropertySequence {
        self.closed.clone()
    }
    /// Share the open boundary sequence; `analyzer.open_free_bounds()`.
    pub fn open_free_bounds(&self) -> FreeBoundPropertySequence {
        self.open.clone()
    }
    /// Get a one-based closed boundary record; `analyzer.closed_free_bound(1)?`.
    pub fn closed_free_bound(
        &self,
        index: i32,
    ) -> Result<Handle<ShapeAnalysisFreeBoundData>, Failure> {
        indexed_property(&self.closed, index)
    }
    /// Get a one-based open boundary record; `analyzer.open_free_bound(1)?`.
    pub fn open_free_bound(
        &self,
        index: i32,
    ) -> Result<Handle<ShapeAnalysisFreeBoundData>, Failure> {
        indexed_property(&self.open, index)
    }
}
fn indexed_property(
    sequence: &FreeBoundPropertySequence,
    index: i32,
) -> Result<Handle<ShapeAnalysisFreeBoundData>, Failure> {
    sequence.sequence(|s| {
        if index < 1 || index > s.length() {
            return Err(Failure::with_type(
                "Standard_OutOfRange",
                &format!("free-bound index {index}; expected 1..={}", s.length()),
                "",
            ));
        }
        s.value(index as usize).cloned()
    })
}
