// Rust translation of TopOpeBRepDS_PointExplorer.hxx/.cxx at
// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::TopOpeBRepDSPoint;
use super::explorer_facade::geometry_explorer_facade;

geometry_explorer_facade!(
    TopOpeBRepDSPointExplorer,
    TopOpeBRepDSPoint,
    point,
    point_at,
    is_point,
    is_point_keep,
    nb_point
);

impl TopOpeBRepDSPointExplorer<'_> {
    /// Reads the current point or the empty point after exhaustion; `explorer.point()?`.
    /// Clearing an active source preserves More() but makes this lookup fail natively.
    pub fn point(&self) -> Result<&TopOpeBRepDSPoint, rustycad_tkernel::standard::Failure> {
        match (self.cursor.source(), self.cursor.current_index()) {
            (Some(source), Some(index)) => source.point(index),
            _ => Ok(self.cursor.empty()),
        }
    }
}
