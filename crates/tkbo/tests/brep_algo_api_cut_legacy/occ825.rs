//! BRepAlgoAPI_Cut_Test.cxx: the OCC825 bug test (a plain TEST outside the BCutSimpleTest fixture).
#![allow(non_snake_case)]

use super::*;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomBezierSurface, GeomSurface};
use rustycad_tkgeombase::geom_convert::surface_to_bspline_surface;
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeHalfSpace, BRepPrimAPIMakeSphere};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;

/// The half-space below the B-spline conversion of the planar Bezier patch y = 0.
fn bspline_half_space(size: f64) -> Shape {
    let mut poles: Array2<'static, Point3> = Array2::with_bounds(1, 2, 1, 2).unwrap();
    poles
        .set_value(1, 1, Point3::new(-size, 0.0, -size))
        .unwrap();
    poles
        .set_value(1, 2, Point3::new(-size, 0.0, size))
        .unwrap();
    poles
        .set_value(2, 1, Point3::new(size, 0.0, -size))
        .unwrap();
    poles.set_value(2, 2, Point3::new(size, 0.0, size)).unwrap();
    let bezier: Handle<dyn GeomSurface> =
        Handle::new(GeomBezierSurface::new(&poles).unwrap()).into_cast();
    let bspline: Handle<dyn GeomSurface> = surface_to_bspline_surface(&bezier).unwrap().into_cast();
    let face_maker =
        BRepBuilderAPIMakeFace::from_surface(&bspline, precision::confusion()).unwrap();
    assert!(face_maker.is_done());
    let face = face_maker.face().unwrap();
    let half_space =
        BRepPrimAPIMakeHalfSpace::with_face(&face, Point3::new(0.0, size, 0.0)).unwrap();
    assert!(half_space.is_done());
    half_space.solid().unwrap().clone().into_shape()
}

fn sphere(center: Point3) -> Shape {
    BRepPrimAPIMakeSphere::at_center(center, 25.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

/// The checks the native test repeats on both cut results.
fn expect_cut_result(result: &Shape) {
    // BRepGProp::SurfaceProperties mass, as GetSurfaceArea computes it.
    expect_near(GetSurfaceArea(result), 5890.42, 5.89042);
    assert_eq!(countSubShapes(result, ShapeEnum::Face), 2);
    assert_eq!(countSubShapes(result, ShapeEnum::Shell), 1);
    assert_eq!(countSubShapes(result, ShapeEnum::Solid), 1);
    assert_eq!(result.shape_type().unwrap(), ShapeEnum::Compound);
}

// OCC825: BRepAlgoAPI_Cut of two spheres with a BSpline-defined half-space produces valid shapes.
// Bug: BRepAlgoAPI_Cut failed on sphere and b-spline face.
// Verifies both cut results are valid and have expected surface area (~5890.42).
#[test]
fn BRepAlgoAPI_CutTest__Bug825_SphereHalfSpaceCut() {
    let half_space = bspline_half_space(50.0);
    let cut1 =
        BRepAlgoAPICut::from_shapes(&sphere(Point3::new(0.0, 0.0, 0.0)), &half_space).unwrap();
    assert!(cut1.is_done(), "BRepAlgoAPI_Cut failed for first sphere");
    let result1 = cut1.shape().clone();
    assert!(!result1.is_null());
    let cut2 =
        BRepAlgoAPICut::from_shapes(&sphere(Point3::new(0.0, 0.00001, 0.0)), &half_space).unwrap();
    assert!(cut2.is_done(), "BRepAlgoAPI_Cut failed for second sphere");
    let result2 = cut2.shape().clone();
    assert!(!result2.is_null());
    expect_cut_result(&result1);
    expect_cut_result(&result2);
}
