//! 1:1 translation of src/ModelingAlgorithms/TKPrim/GTests/BRepPrimAPI_MakePrism_Test.cxx.
#![allow(non_snake_case)]

use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakePrism;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeVertex};

#[test]
fn BRepPrimAPI_MakePrism_Test__OCC31294_GeneratedListForNonBaseShape() {
    // Bug OCC31294: Modeling Algorithms - Regression relatively 7.3.0.
    // Crash in method BRepPrimAPI_MakePrism::Generated(...) if input sub-shape
    // does not belong to the base shape
    //
    // This test verifies that calling Generated() with a shape that doesn't belong
    // to the base shape doesn't crash and returns an empty list
    let mut a_mk_vert = BRepBuilderAPIMakeVertex::new(Point3::new(0., 0., 0.)).unwrap();
    let mut a_mk_dummy = BRepBuilderAPIMakeVertex::new(Point3::new(0., 0., 0.)).unwrap();
    let a_vert = a_mk_vert.shape().unwrap().clone();
    let a_dummy = a_mk_dummy.shape().unwrap().clone();
    let mut a_mk_prism =
        BRepPrimAPIMakePrism::new(&a_vert, Vector3::new(0., 0., 1.), false, true).unwrap();

    // Check that Generated() returns 1 shape for the vertex used to create the prism
    let a_nb_gen = a_mk_prism.generated(&a_vert).unwrap().len();
    assert_eq!(a_nb_gen, 1);

    // Check that Generated() returns 0 shapes for a vertex not used to create the prism
    // (this should not crash)
    let a_nb_dummy = a_mk_prism.generated(&a_dummy).unwrap().len();
    assert_eq!(a_nb_dummy, 0);
}
