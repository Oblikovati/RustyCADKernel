//! BRepBuilderAPI probe operations, the Rust twin of
//! tools/oracle/tktopalgo_probe_brepbuilderapi.cxx. The builder is still the STUB
//! `MakeVertex`, so only the vertex it builds is compared.
use super::common::{Reader, attempt, end, flag, pnt, real};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tktopalgo::brep_builder_api::MakeVertex;

/// `point`: BRepBuilderAPI_MakeVertex(P), then IsDone, BRep_Tool::Pnt and Tolerance.
pub fn make_vertex_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let builder = MakeVertex::new(r.pnt());
    flag(builder.is_done());
    attempt(|| {
        pnt(BRepTool::pnt(builder.vertex())?);
        real(BRepTool::vertex_tolerance(builder.vertex())?);
        Ok(())
    });
    end();
}
