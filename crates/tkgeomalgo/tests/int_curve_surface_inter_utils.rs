//! Direct translation of IntCurveSurface_InterUtils_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::int_curve_surface::inter_utils::{
    SectionPolygon, SectionPolyhedron, section_point_to_parameters,
};
use rustycad_tkgeomalgo::intf::{PIType, SectionPoint};
use rustycad_tkmath::gp::Point3;

/// The upstream `TestPolyhedron`: one degenerate facet (two equal vertices).
struct TestPolyhedron {
    my_points: [Point3; 3],
    my_u: [f64; 3],
    my_v: [f64; 3],
}

impl TestPolyhedron {
    fn new() -> Self {
        Self {
            my_points: [
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(2.0, 0.0, 0.0),
                Point3::new(2.0, 0.0, 0.0),
            ],
            my_u: [0.0, 1.0, 2.0],
            my_v: [0.0, 0.0, 0.0],
        }
    }
}

impl SectionPolyhedron for TestPolyhedron {
    fn triangle(&self, _index: i32) -> Result<[i32; 3], Failure> {
        Ok([1, 2, 3])
    }
    fn point(&self, index: i32) -> Result<Point3, Failure> {
        Ok(self.my_points[(index - 1) as usize])
    }
    fn parameters(&self, index: i32) -> Result<(f64, f64), Failure> {
        Ok((
            self.my_u[(index - 1) as usize],
            self.my_v[(index - 1) as usize],
        ))
    }
}

/// The upstream `TestPolygon`: the curve parameter `10 * index + position`.
struct TestPolygon;

impl SectionPolygon for TestPolygon {
    fn approx_param_on_curve(&self, index: i32, param_on_line: f64) -> Result<f64, Failure> {
        Ok(10.0 * f64::from(index) + param_on_line)
    }
}

#[test]
fn IntCurveSurface_InterUtils__SectionPointToParameters_DegenerateFaceFallsBackToLongestEdge() {
    let a_polyhedron = TestPolyhedron::new();
    let a_polygon = TestPolygon;

    let a_section_point = SectionPoint::with_data(
        Point3::new(0.5, 0.0, 0.0),
        PIType::Edge,
        0,
        2,
        0.4,
        PIType::Face,
        1,
        0,
        0.0,
        0.0,
    );

    let (a_u, a_v, a_w) =
        section_point_to_parameters(&a_section_point, &a_polyhedron, &a_polygon).unwrap();

    assert!((a_u - 0.25).abs() <= 1.0e-12);
    assert!((a_v - 0.0).abs() <= 1.0e-12);
    assert!((a_w - 20.4).abs() <= 1.0e-12);
}
