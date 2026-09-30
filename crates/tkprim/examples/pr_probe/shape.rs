//! The shape digest of the TKPrim probes, the Rust twin of
//! tools/oracle/tkprim_probe_shape.cxx: every TKPrim result is compared through the
//! TKTopAlgo digest (`tp_shape`, brep_lib_common.rs).
use super::brep_lib_common;
use super::common::{Reader, attempt, end};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::Vertex;

/// `pr_digest_vertex x y z tol`: the digest of a vertex built with BRep_Builder.
pub fn digest_vertex_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let point = r.pnt();
    let tolerance = r.real();
    attempt(|| {
        let mut vertex = Vertex::default();
        BRepBuilder::default().make_vertex_at(&mut vertex, point, tolerance)?;
        brep_lib_common::shape(&vertex)
    });
    end();
}
