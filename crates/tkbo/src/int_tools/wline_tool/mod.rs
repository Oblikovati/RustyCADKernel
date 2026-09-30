// IntTools_WLineTool.hxx/.cxx at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Walking-line approximation decisions and decomposition.
mod degenerate;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeomalgo::int_patch::{IntPatchPointLine, IntPatchWLine};
/// Native walking-line utilities; `WLineTool::not_use_surfaces_for_approx(&f1, &f2, &line, 1, 2)?`.
pub struct WLineTool;
impl WLineTool {
    /// Tests endpoint UVs using native degenerate-zone behavior; `WLineTool::not_use_surfaces_for_approx(&f1, &f2, &line, 1, 2)?`.
    pub fn not_use_surfaces_for_approx(
        first: &Face,
        last: &Face,
        line: &Handle<IntPatchWLine>,
        start: i32,
        end: i32,
    ) -> Result<bool, Failure> {
        let points = line.curve();
        if degenerate::point_in_zone(points.value(start)?, first, last)? {
            return Ok(true);
        }
        degenerate::point_in_zone(points.value(end)?, first, last)
    }
}
