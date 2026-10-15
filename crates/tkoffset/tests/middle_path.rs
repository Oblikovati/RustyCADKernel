//! Independent checks of BRepOffsetAPI_MiddlePath: the middle path of a box between
//! opposite faces is the segment between the face centroids, and that of a partial torus
//! between its caps is the arc of the main radius.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use rustycad_tkoffset::brep_offset_api::BRepOffsetAPIMiddlePath;
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakeTorus};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use rustycad_tktopalgo::brep_gprop::BRepGProp;

fn faces(s: &Shape) -> Result<Vec<Shape>, Failure> {
    Ok(Explorer::new(s, ShapeEnum::Face, ShapeEnum::Shape)?.collect())
}

fn edges(s: &Shape) -> Result<Vec<Shape>, Failure> {
    Ok(Explorer::new(s, ShapeEnum::Edge, ShapeEnum::Shape)?.collect())
}

fn length(s: &Shape) -> Result<f64, Failure> {
    let mut props = GProps::new();
    BRepGProp::linear_properties(s, &mut props, false, false)?;
    Ok(props.mass())
}

#[test]
fn box_middle_path_joins_the_centroids_of_the_opposite_faces() -> Result<(), Failure> {
    let a_box = BRepPrimAPIMakeBox::from_sizes(2.0, 3.0, 4.0)?
        .shape()?
        .clone();
    let f = faces(&a_box)?;
    // Faces 0 and 1 are x = 0 and x = 2.
    let mut path = BRepOffsetAPIMiddlePath::new(&a_box, &f[0], &f[1])?;
    let wire = path.shape()?.clone();
    let e = edges(&wire)?;
    assert_eq!(e.len(), 1);
    let (curve, first, last) = BRepTool::curve(&topods::edge(&e[0])?)?;
    let curve = curve.get().expect("3d curve");
    let (p1, p2) = (curve.value(first)?, curve.value(last)?);
    let expected = [Point3::new(0.0, 1.5, 2.0), Point3::new(2.0, 1.5, 2.0)];
    let close = |a: Point3, b: Point3| a.distance(&b) < 1.0e-12;
    assert!(
        (close(p1, expected[0]) && close(p2, expected[1]))
            || (close(p1, expected[1]) && close(p2, expected[0])),
        "{p1:?} {p2:?}"
    );
    assert!((length(&wire)? - 2.0).abs() < 1.0e-12);
    Ok(())
}

#[test]
fn partial_torus_middle_path_is_the_arc_of_the_main_radius() -> Result<(), Failure> {
    let (main, minor, angle) = (3.0, 1.0, 1.5);
    let frame = Axis2::new(Point3::new(0.0, 0.0, 0.0), Direction3::new(0.0, 0.0, 1.0)?)?;
    let torus = BRepPrimAPIMakeTorus::with_axes_angle(frame, main, minor, angle)?
        .shape()?
        .clone();
    let f = faces(&torus)?;
    let mut path = BRepOffsetAPIMiddlePath::new(&torus, &f[1], &f[2])?;
    let wire = path.shape()?.clone();
    let e = edges(&wire)?;
    assert_eq!(e.len(), 1);
    let (curve, _, _) = BRepTool::curve(&topods::edge(&e[0])?)?;
    assert!(curve.get().expect("3d curve").is_kind_name("Geom_Circle"));
    // The centroid of each disk cap is on the circle of the main radius.
    assert!(
        (length(&wire)? - main * angle).abs() < 1.0e-9,
        "length {}",
        length(&wire)?
    );
    Ok(())
}
