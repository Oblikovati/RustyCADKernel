//! The shape digest of the TKPrim probes, the Rust twin of
//! tools/oracle/tkprim_probe_shape.cxx: every TKPrim result is compared through the
//! TKTopAlgo digest (`tp_shape`, brep_lib_common.rs).
use super::brep_lib_common;
use super::common::{Reader, attempt, end, int, pnt, show, xyz};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Face, Shape, Vertex};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

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

/// `pr_face_frames`: the first derivatives of the located surface of each face at
/// (0.3, 0.7), after their count; -1 for a face without surface.
fn face_frames(s: &Shape) -> Result<(), Failure> {
    let mut faces = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(s, ShapeEnum::Face, &mut faces)?;
    int(faces.extent() as i32);
    for i in 1..=faces.extent() {
        let face = Face::unchecked(faces.find_key(i)?.clone());
        let surface = BRepTool::surface(&face)?;
        let Some(surface) = surface.get() else {
            int(-1);
            continue;
        };
        show(surface.d1(0.3, 0.7), |d1| {
            pnt(d1.point);
            xyz(*d1.d1u.xyz());
            xyz(*d1.d1v.xyz());
        });
    }
    Ok(())
}

/// `pr_shape`: the TKTopAlgo digest, then the face frames of a non-null shape.
pub fn shape(s: &Shape) -> Result<(), Failure> {
    brep_lib_common::shape(s)?;
    if !s.is_null() {
        face_frames(s)?;
    }
    Ok(())
}

/// [`shape`] of any typed shape; `digest(face)?`.
pub fn digest<S: AsRef<Shape>>(s: S) -> Result<(), Failure> {
    shape(s.as_ref())
}
