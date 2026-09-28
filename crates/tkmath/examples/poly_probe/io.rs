//! Poly::Write, Poly::Dump and Poly::Read* counterparts.
use super::{Fields, failure, flag, flat, mesh};
use rustycad_tkernel::{
    ncollection::Array1,
    standard::{Failure, Handle, RuntimeTyped, dump::DumpStream},
};
use rustycad_tkmath::{
    gp::{Point2, Point3},
    poly::{self, Polygon2D, Polygon3D, Triangulation},
};
use std::io::Cursor;
/// One Poly transient with its native writer, reader and JSON dump.
trait Written: RuntimeTyped + Sized {
    fn write(&self, out: &mut Vec<u8>, compact: bool) -> Result<(), Failure>;
    fn read(input: &mut Cursor<Vec<u8>>) -> Result<Handle<Self>, Failure>;
    fn json(&self, out: &mut DumpStream);
}
impl Written for Triangulation {
    fn write(&self, out: &mut Vec<u8>, compact: bool) -> Result<(), Failure> {
        poly::write_triangulation(self, out, compact)
    }
    fn read(input: &mut Cursor<Vec<u8>>) -> Result<Handle<Self>, Failure> {
        poly::read_triangulation(input)
    }
    fn json(&self, out: &mut DumpStream) {
        self.dump_json(out).unwrap();
    }
}
impl Written for Polygon3D {
    fn write(&self, out: &mut Vec<u8>, compact: bool) -> Result<(), Failure> {
        poly::write_polygon3d(self, out, compact)
    }
    fn read(input: &mut Cursor<Vec<u8>>) -> Result<Handle<Self>, Failure> {
        poly::read_polygon3d(input)
    }
    fn json(&self, out: &mut DumpStream) {
        self.dump_json(out).unwrap();
    }
}
impl Written for Polygon2D {
    fn write(&self, out: &mut Vec<u8>, compact: bool) -> Result<(), Failure> {
        poly::write_polygon2d(self, out, compact)
    }
    fn read(input: &mut Cursor<Vec<u8>>) -> Result<Handle<Self>, Failure> {
        poly::read_polygon2d(input)
    }
    fn json(&self, out: &mut DumpStream) {
        self.dump_json(out).unwrap();
    }
}
fn reread<T: Written>(compact: &[u8]) -> Result<String, Failure> {
    let mut input = Cursor::new([compact, compact].concat());
    let (first, second) = (T::read(&mut input)?, T::read(&mut input)?);
    let mut text = Vec::new();
    first.write(&mut text, true)?;
    second.write(&mut text, true)?;
    Ok(flat(&text))
}
fn round_trip<T: Written>(shape: &T) {
    let (mut compact, mut dump) = (Vec::new(), Vec::new());
    shape.write(&mut compact, true).unwrap();
    shape.write(&mut dump, false).unwrap();
    print!("{} {} ", flat(&compact), flat(&dump));
    match reread::<T>(&compact) {
        Ok(text) => print!("{text} "),
        Err(error) => failure(&error),
    }
    let mut json = DumpStream::default();
    shape.json(&mut json);
    print!("{} ", json.text());
}
pub(crate) fn io_case(values: &[f64]) {
    let mut fields = Fields::new(values);
    let mesh = mesh(&mut fields);
    let options = fields.int();
    let (deflection, scale) = (fields.real(), fields.real());
    mesh.set_deflection(deflection);
    if options & 1 != 0 {
        mesh.add_uv_nodes().unwrap();
        for index in 1..=mesh.nb_nodes() {
            let node = mesh.node(index).unwrap();
            mesh.set_uv_node(index, Point2::new(node.x() * scale, node.y() / scale))
                .unwrap();
        }
    }
    round_trip(&*mesh);
    let count = mesh.nb_nodes();
    let nodes: Vec<Point3> = (1..=count).map(|i| mesh.node(i).unwrap()).collect();
    let mut node_table = Array1::with_bounds(1, count).unwrap();
    node_table.as_mut_slice().copy_from_slice(&nodes);
    let mut parameters = Array1::with_bounds(1, count).unwrap();
    parameters
        .iter_mut()
        .enumerate()
        .for_each(|(i, p)| *p = (i + 1) as f64 * scale);
    let polygon3d = if options & 2 != 0 {
        Polygon3D::from_nodes_and_parameters(&node_table, &parameters).unwrap()
    } else {
        Polygon3D::from_nodes(&node_table)
    };
    polygon3d.set_deflection(deflection * 2.0);
    round_trip(&polygon3d);
    let uv: Vec<Point2> = nodes.iter().map(|n| Point2::new(n.y(), n.z())).collect();
    let mut uv_table = Array1::with_bounds(1, count).unwrap();
    uv_table.as_mut_slice().copy_from_slice(&uv);
    let polygon2d = Polygon2D::from_nodes(&uv_table);
    polygon2d.set_deflection(deflection / 3.0);
    round_trip(&polygon2d);
    let mut wrong = Cursor::new(b"Poly_Polygon2D 1 0 0 0".to_vec());
    println!(
        "{}",
        flag(poly::read_triangulation(&mut wrong).unwrap().is_null())
    );
}
