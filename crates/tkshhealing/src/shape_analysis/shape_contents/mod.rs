//! ShapeAnalysis_ShapeContents at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp::Explorer,
    topods::{self, Face, Shape},
    toptools::MapOfShape,
};
use rustycad_tkernel::{
    ncollection::HSequence,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::toploc::Location;
mod accessors;
mod curves;
mod state;
mod surfaces;
mod topology;
use state::{ContentCounts, ContentRecordings};
/// Counts topology occurrences and selected geometry properties, optionally retaining matching shapes.
/// Example: `let mut contents = ShapeAnalysisShapeContents::new(); contents.perform(&shape)?;`.
/// Native counters are uninitialized until Clear/Perform; Rust initializes this undefined domain to zero.
#[derive(Clone, Default)]
pub struct ShapeAnalysisShapeContents {
    counts: ContentCounts,
    recordings: ContentRecordings,
}
impl ShapeAnalysisShapeContents {
    /// Allocate six shared recording sequences and disabled recording modes.
    /// Example: `let contents = ShapeAnalysisShapeContents::new();`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Reset counters and empty the existing recording handles while retaining modes.
    /// Example: `contents.clear();`.
    pub fn clear(&mut self) {
        self.counts = ContentCounts::default();
        self.recordings.clear_sequences();
    }
    /// Disable recording without changing counters or saved shapes; `contents.clear_flags()`.
    pub fn clear_flags(&mut self) {
        self.recordings.clear_flags();
    }
    /// Reset statistics, then traverse native topology occurrences and geometry.
    /// Example: `contents.perform(&shape)?; assert_eq!(contents.nb_solids(), 1);`.
    pub fn perform(&mut self, shape: &Shape) -> Result<(), Failure> {
        self.clear();
        self.count_solids(shape)?;
        let shell_faces = self.count_shells(shape)?;
        self.count_faces(shape)?;
        self.counts.shared_wires = visit_kind(shape, ShapeEnum::Wire, ShapeEnum::Shape, |_| {
            self.counts.wires += 1;
            Ok(())
        })?;
        self.counts.free_faces = self.counts.faces - shell_faces;
        self.count_edges(shape)?;
        self.count_remaining_topology(shape)
    }
}
fn visit_kind(
    shape: &Shape,
    kind: ShapeEnum,
    avoid: ShapeEnum,
    mut visit: impl FnMut(&Shape) -> Result<(), Failure>,
) -> Result<i32, Failure> {
    let mut explorer = Explorer::new(shape, kind, avoid)?;
    let mut unique = MapOfShape::default();
    while explorer.more() {
        let current = explorer.current();
        let mut unlocated = current.clone();
        unlocated.set_location(&Location::default(), true)?;
        unique.add(unlocated);
        visit(current)?;
        explorer.next()?;
    }
    Ok(unique.extent() as i32)
}
