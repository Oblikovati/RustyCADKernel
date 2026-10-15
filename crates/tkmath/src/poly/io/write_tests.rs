//! The Poly writers take their number text from the caller's stream format.
use super::{
    write_polygon2d_with, write_polygon3d, write_polygon3d_with, write_triangulation_with,
};
use crate::gp::{Point2, Point3};
use crate::poly::{Polygon2D, Polygon3D, Triangle, Triangulation};

fn polygon3d() -> Polygon3D {
    let polygon = Polygon3D::new(2, false).unwrap();
    polygon.change_nodes(|nodes| {
        nodes.set_value(1, Point3::new(0.5, 1., 2.)).unwrap();
        nodes.set_value(2, Point3::new(3., 4., 5.)).unwrap();
    });
    polygon.set_deflection(0.25);
    polygon
}

fn written(write: impl FnOnce(&mut Vec<u8>)) -> String {
    let mut bytes = Vec::new();
    write(&mut bytes);
    String::from_utf8(bytes).unwrap()
}

#[test]
fn dump_pads_the_caller_number_text_in_seventeen_columns() {
    let text = written(|out| {
        write_polygon3d_with(&polygon3d(), out, false, &|value| format!("{value:.2}")).unwrap()
    });
    assert_eq!(
        text,
        "Poly_Polygon3D\n       2 Nodes\nwithout parameters\nDeflection : 0.25\n\nNodes :\n\
         \x20        1 :              0.50              1.00              2.00\n\
         \x20        2 :              3.00              4.00              5.00\n"
    );
}

#[test]
fn save_writes_the_caller_number_text_without_columns() {
    let scientific = |value: f64| format!("{value:.1e}");
    let text = written(|out| write_polygon3d_with(&polygon3d(), out, true, &scientific).unwrap());
    assert_eq!(
        text,
        "Poly_Polygon3D\n2 0\n2.5e-1\n5.0e-1 1.0e0 2.0e0\n3.0e0 4.0e0 5.0e0\n"
    );
}

#[test]
fn default_writer_keeps_the_general_number_text() {
    let explicit = written(|out| {
        write_polygon3d_with(&polygon3d(), out, true, &crate::stream_format::general).unwrap()
    });
    let default = written(|out| write_polygon3d(&polygon3d(), out, true).unwrap());
    assert_eq!(explicit, default);
}

#[test]
fn polygon2d_and_triangulation_use_the_same_formatter() {
    let polygon = Polygon2D::new(1).unwrap();
    polygon.change_nodes(|nodes| nodes.set_value(1, Point2::new(1.5, -2.)).unwrap());
    let text = written(|out| {
        write_polygon2d_with(&polygon, out, true, &|value| format!("<{value}>")).unwrap()
    });
    assert_eq!(text, "Poly_Polygon2D\n1 <0>\n<1.5> <-2>\n");
    let mesh = Triangulation::with_sizes(3, 1, false, false).unwrap();
    mesh.set_node(1, Point3::new(0., 0., 0.)).unwrap();
    mesh.set_node(2, Point3::new(1., 0., 0.)).unwrap();
    mesh.set_node(3, Point3::new(0., 1., 0.)).unwrap();
    mesh.set_triangle(1, Triangle::new(1, 2, 3)).unwrap();
    let text = written(|out| {
        write_triangulation_with(&mesh, out, true, &|value| format!("[{value}]")).unwrap()
    });
    assert_eq!(
        text,
        "Poly_Triangulation\n3 1 0\n[0]\n[0] [0] [0]\n[1] [0] [0]\n[0] [1] [0]\n1 2 3\n"
    );
}
