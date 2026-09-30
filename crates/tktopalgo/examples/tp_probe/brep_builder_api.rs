//! BRepBuilderAPI_MakeVertex probe operation, the Rust twin of
//! tools/oracle/tktopalgo_probe_brepbuilderapi.cxx.
use super::common::{Reader, attempt, end, flag, pnt, real};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeVertex};

/// `tp_make_vertex x y z`: BRepBuilderAPI_MakeVertex(P), then IsDone, BRep_Tool::Pnt and
/// Tolerance of the vertex.
pub fn make_vertex_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let point = r.pnt();
    attempt(|| {
        let builder = BRepBuilderAPIMakeVertex::new(point)?;
        flag(builder.is_done());
        attempt(|| {
            let vertex = builder.vertex()?;
            pnt(BRepTool::pnt(&vertex)?);
            real(BRepTool::vertex_tolerance(&vertex)?);
            Ok(())
        });
        Ok(())
    });
    end();
}
