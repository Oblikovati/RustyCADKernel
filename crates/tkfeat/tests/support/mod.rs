//! Shared fixtures of the TKFeat tests: boxes and cylinders, faces picked by their
//! centre, and the measures (volume, area, validity) the closed-form checks compare.
#![allow(dead_code)]
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::Point3;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use rustycad_tktopalgo::brep_gprop::BRepGProp;

/// The box `[x0, x0 + dx] x [y0, y0 + dy] x [z0, z0 + dz]`.
pub fn make_box(origin: (f64, f64, f64), size: (f64, f64, f64)) -> Shape {
    BRepPrimAPIMakeBox::at_point(
        Point3::new(origin.0, origin.1, origin.2),
        size.0,
        size.1,
        size.2,
    )
    .unwrap()
    .shape()
    .unwrap()
    .clone()
}

/// The sub-shapes of `s` of type `kind`, in exploration order.
pub fn sub_shapes(s: &Shape, kind: ShapeEnum) -> Vec<Shape> {
    Explorer::new(s, kind, ShapeEnum::Shape).unwrap().collect()
}

/// The number of distinct sub-shapes of type `kind`.
pub fn count(s: &Shape, kind: ShapeEnum) -> usize {
    let mut map = rustycad_tkbrep::toptools::IndexedMapOfShape::default();
    rustycad_tkbrep::topexp::map_shapes_of_type(s, kind, &mut map).unwrap();
    map.extent()
}

/// The centre of mass of the face surface.
pub fn face_centre(f: &Face) -> Point3 {
    let mut p = GProps::new();
    BRepGProp::surface_properties(f, &mut p, false, false).unwrap();
    p.centre_of_mass()
}

/// The face of `s` whose centre is nearest to `target`.
pub fn face_near(s: &Shape, target: Point3) -> Face {
    sub_shapes(s, ShapeEnum::Face)
        .into_iter()
        .map(|f| topods::face(&f).unwrap())
        .min_by(|a, b| {
            let da = face_centre(a).distance(&target);
            let db = face_centre(b).distance(&target);
            da.partial_cmp(&db).unwrap()
        })
        .unwrap()
}

/// The edge of `s` whose middle point is nearest to `target`.
pub fn edge_near(s: &Shape, target: Point3) -> Edge {
    sub_shapes(s, ShapeEnum::Edge)
        .into_iter()
        .map(|e| topods::edge(&e).unwrap())
        .min_by(|a, b| {
            let mid = |e: &Edge| {
                let (c, f, l) = BRepTool::curve(e).unwrap();
                c.get().unwrap().value((f + l) / 2.).unwrap()
            };
            mid(a)
                .distance(&target)
                .partial_cmp(&mid(b).distance(&target))
                .unwrap()
        })
        .unwrap()
}

/// The volume of a solid (or of the solids of a compound).
pub fn volume(s: &Shape) -> f64 {
    let mut p = GProps::new();
    BRepGProp::volume_properties(s, &mut p, false, false, false).unwrap();
    p.mass()
}

/// The area of the faces of a shape.
pub fn area(s: &Shape) -> f64 {
    let mut p = GProps::new();
    BRepGProp::surface_properties(s, &mut p, false, false).unwrap();
    p.mass()
}

/// BRepCheck validity with the geometric controls.
pub fn is_valid(s: &Shape) -> bool {
    BRepCheckAnalyzer::new(s, true, false, false)
        .unwrap()
        .is_valid()
        .unwrap()
}

/// Asserts `a` and `b` agree to the relative tolerance `rel`.
pub fn assert_close(a: f64, b: f64, rel: f64) {
    assert!(
        (a - b).abs() <= rel * b.abs().max(1.),
        "{a} differs from {b} by more than {rel} relative"
    );
}

/// The planar face of the rectangle `[x0, x1] x [y0, y1]` at height `z`, normal +Z.
pub fn rect_face(x0: f64, y0: f64, x1: f64, y1: f64, z: f64) -> Face {
    use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon};
    let mut poly = BRepBuilderAPIMakePolygon::new();
    for (x, y) in [(x0, y0), (x1, y0), (x1, y1), (x0, y1)] {
        poly.add_point(Point3::new(x, y, z)).unwrap();
    }
    poly.close().unwrap();
    BRepBuilderAPIMakeFace::from_wire(&poly.wire().unwrap(), true)
        .unwrap()
        .face()
        .unwrap()
}

/// The planar face bounded by the closed polygon through `points`.
pub fn polygon_face(points: &[(f64, f64, f64)]) -> Face {
    use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon};
    let mut poly = BRepBuilderAPIMakePolygon::new();
    for p in points {
        poly.add_point(Point3::new(p.0, p.1, p.2)).unwrap();
    }
    poly.close().unwrap();
    BRepBuilderAPIMakeFace::from_wire(&poly.wire().unwrap(), true)
        .unwrap()
        .face()
        .unwrap()
}
