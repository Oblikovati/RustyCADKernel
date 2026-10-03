use super::spot::ShapeAnalysisCheckSmallFace;
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::standard::Failure;

impl ShapeAnalysisCheckSmallFace {
    /// Checks the single-strip topology first, then searches for two strip edges.
    /// `analyzer.check_strip_face(&face, &mut first, &mut second, -1.0)?`.
    pub fn check_strip_face(
        &mut self,
        face: &Face,
        first_edge: &mut Edge,
        second_edge: &mut Edge,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        if self.check_single_strip(face, first_edge, second_edge, tolerance)? {
            return Ok(true);
        }
        let mut maximum_distance = 0.0;
        self.find_strip_edges(
            face,
            first_edge,
            second_edge,
            tolerance,
            &mut maximum_distance,
        )
    }
}
