//! Poly triangulation, package functions, text I/O and connectivity oracle counterpart.
mod coherent;
mod io;
mod loops;
mod merge;
mod mesh;
pub(super) use coherent::coherent_case;
pub(super) use io::io_case;
pub(super) use loops::loops_case;
pub(super) use merge::merge_case;
pub(super) use mesh::{
    connect_case, mesh_case, point_on_triangle_case, properties_case, tri_line_case,
};
use rustycad_tkernel::{
    ncollection::Vec3,
    standard::{Failure, Handle},
};
use rustycad_tkmath::{
    bnd::BoundingBox3,
    gp::{Axis1, Direction3, Point3, Transform3},
    poly::{Triangle, Triangulation},
};
/// Sequential reader over one request's numeric fields.
pub(super) struct Fields<'v> {
    values: &'v [f64],
    next: usize,
}
impl<'v> Fields<'v> {
    pub(super) fn new(values: &'v [f64]) -> Self {
        Self { values, next: 0 }
    }
    pub(super) fn real(&mut self) -> f64 {
        self.next += 1;
        self.values[self.next - 1]
    }
    pub(super) fn int(&mut self) -> i32 {
        self.real() as i32
    }
}
pub(super) fn failure(error: &Failure) {
    print!("{} ", error.exception_type());
}
pub(super) fn flag(value: bool) -> i32 {
    i32::from(value)
}
pub(super) fn fvec(value: Vec3<f32>) {
    print!(
        "{} {} {} ",
        f64::from(value.x()).to_bits(),
        f64::from(value.y()).to_bits(),
        f64::from(value.z()).to_bits()
    );
}
/// The native `mesh()` reader: precision flag, counts, nodes and triangles.
pub(super) fn mesh(fields: &mut Fields<'_>) -> Handle<Triangulation> {
    let single = fields.int() != 0;
    let (nodes, triangles) = (fields.int(), fields.int());
    let mesh = Triangulation::new();
    mesh.set_double_precision(!single).unwrap();
    mesh.resize_nodes(nodes, false).unwrap();
    mesh.resize_triangles(triangles, false).unwrap();
    for index in 1..=nodes {
        let point = Point3::new(fields.real(), fields.real(), fields.real());
        mesh.set_node(index, point).unwrap();
    }
    for index in 1..=triangles {
        let triangle = Triangle::new(fields.int(), fields.int(), fields.int());
        mesh.set_triangle(index, triangle).unwrap();
    }
    Handle::new(mesh)
}
pub(super) fn bounds(range: &BoundingBox3) {
    match range.get() {
        Ok(values) => values.iter().for_each(|v| print!("{} ", v.to_bits())),
        Err(_) => print!("void "),
    }
}
/// The native `transform()` reader: offset, angle or scale, and a form selector.
pub(super) fn transform(fields: &mut Fields<'_>) -> Transform3 {
    let point = Point3::new(fields.real(), fields.real(), fields.real());
    let angle = fields.real();
    match fields.int() {
        1 => Transform3::translation(*point.coordinates()),
        2 => Transform3::rotation(Axis1::new(point, Direction3::Z), angle).unwrap(),
        3 => Transform3::scaling(point, angle).unwrap(),
        4 => Transform3::mirror_point(point),
        _ => Transform3::default(),
    }
}
/// Newlines of a text block become `|`, as the native probe prints them.
pub(super) fn flat(text: &[u8]) -> String {
    String::from_utf8_lossy(text).replace('\n', "|")
}
