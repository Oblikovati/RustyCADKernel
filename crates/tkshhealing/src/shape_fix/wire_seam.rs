//! Native seam p-curve exchange and range preservation.
// ShapeFix_Wire.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::ShapeFixWire;
use crate::shape_extend::ShapeExtendStatus;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkernel::standard::{Failure, Handle};

impl ShapeFixWire {
    /// Swap incorrectly ordered seam p-curves; `fixer.fix_seam(0)?` selects the last edge.
    pub fn fix_seam(&self, index: i32) -> Result<bool, Failure> {
        self.set_last_fix_status(ShapeExtendStatus::Ok);
        if !self.is_ready() {
            return Ok(false);
        }
        let analyzer = self.analyzer()?;
        let (mut first, mut second, mut range) = (Handle::default(), Handle::default(), (0., 0.));
        if !analyzer.check_seam_into(index, &mut first, &mut second, &mut range)? {
            return Ok(false);
        }
        let selected = if index > 0 {
            index
        } else {
            analyzer.nb_edges()
        };
        let edge = analyzer.wire_data().edge(selected)?;
        let builder = BRepBuilder::default();
        builder.update_edge_pcurves_on_face(&edge, (&second, &first), &analyzer.face(), 0.)?;
        builder.range_on_face(&edge, &analyzer.face(), range.0, range.1)?;
        self.or_last_fix_status(ShapeExtendStatus::Done1);
        Ok(true)
    }
}
