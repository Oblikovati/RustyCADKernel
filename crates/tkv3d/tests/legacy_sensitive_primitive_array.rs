//! Translation of OCCT's Select3D_SensitivePrimitiveArray_Test.cxx at 3d097a0.
// Copyright (c) 2026 OPEN CASCADE SAS; LGPL-2.1 with OCCT exception.
use rustycad_tkernel::precision::confusion;
use rustycad_tkmath::toploc::Location;
use rustycad_tkservice::graphic3d::Graphic3dArrayOfTriangles;
use rustycad_tkv3d::select_mgr::SelectMgrEntityOwner;
use rustycad_tkv3d::select3d::{Select3DPrimArrayInit, Select3DSensitivePrimitiveArray};

#[test]
fn primitive_array_get_vertex_returns_triangle_vertices() {
    // GTests/Select3D_SensitivePrimitiveArray_Test.cxx: GetVertex_ReturnsTriangleVertices.
    let tris = Graphic3dArrayOfTriangles::new(3, 0, 0).unwrap();
    tris.add_vertex_f64(0.0, 0.0, 0.0).unwrap();
    tris.add_vertex_f64(1.0, 0.0, 0.0).unwrap();
    tris.add_vertex_f64(0.0, 1.0, 0.0).unwrap();
    let sens = Select3DSensitivePrimitiveArray::new(&SelectMgrEntityOwner::new(0).cast()).unwrap();
    let init = Select3DPrimArrayInit::whole(
        &tris.attributes(),
        &tris.indices(),
        &Location::default(),
        true,
        1,
    );
    assert!(sens.init_triangulation(&init).unwrap());
    let vertices = sens.get_vertex(0);
    let expected = [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
    let tolerance = confusion() as f32;
    for (vertex, wanted) in vertices.iter().zip(expected) {
        assert!(
            (vertex.x - wanted[0]).abs() <= tolerance
                && (vertex.y - wanted[1]).abs() <= tolerance
                && (vertex.z - wanted[2]).abs() <= tolerance
        );
    }
}
