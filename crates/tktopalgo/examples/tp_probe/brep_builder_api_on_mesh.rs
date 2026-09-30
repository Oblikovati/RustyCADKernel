//! The BRepBuilderAPI_MakeShapeOnMesh probe operation, the Rust twin of
//! tools/oracle/tktopalgo_probe_brepbuilderapi_onmesh.cxx (the request layout is
//! documented there).
use super::brep_builder_api_makers::made;
use super::common::{Reader, attempt, end};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::poly::{Triangle, Triangulation};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShapeOnMesh,
};

/// `tp_onmesh_mesh`: the triangulation, sized first (null for a negative node count).
fn on_mesh_mesh(r: &mut Reader<'_>) -> Result<Handle<Triangulation>, Failure> {
    let nt = r.integer();
    if nt < 0 {
        return Ok(Handle::default());
    }
    let nodes: Vec<_> = (0..nt).map(|_| r.pnt()).collect();
    let ntri = r.integer();
    let mesh = Triangulation::with_sizes(nt, ntri, false, false)?;
    for (i, node) in (1..).zip(nodes) {
        mesh.set_node(i, node)?;
    }
    for i in 1..=ntri {
        let (a, b) = (r.integer(), r.integer());
        mesh.set_triangle(i, Triangle::new(a, b, r.integer()))?;
    }
    mesh.set_deflection(r.real());
    Ok(Handle::new(mesh))
}

/// `mesh builds`: Build() `builds` times, then `tp_bba_made`.
pub fn on_mesh_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let mesh = on_mesh_mesh(&mut r)?;
        let builds = r.integer();
        let mut m = BRepBuilderAPIMakeShapeOnMesh::new(&mesh);
        for _ in 0..builds {
            attempt(|| m.build(&ProgressRange::default()));
        }
        made(&mut m, true);
        Ok(())
    });
    end();
}
