use super::{fclass_faces, flag, integer};
use rustycad_tkbo::int_tools::{CommonPrt, Tools};
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topexp::Explorer,
    topods::{self, Edge, Face, Wire},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeVertex};
fn edge(offset: f64) -> Result<Edge, Failure> {
    BRepBuilderAPIMakeEdge::from_points(Point3::new(offset, 0., 0.), Point3::new(offset, 0., 2.))?
        .edge()
}
fn wire_face(e: &Edge, v: &[f64]) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    builder.add(&mut wire, &e.oriented(Orientation::ALL[v[3] as usize]))?;
    flag(Tools::has_internal_edge(&wire)?);
    let face = fclass_faces::face(0, 2.)?;
    let mut output = Face::default();
    let source = Face::unchecked(face.oriented(Orientation::Reversed));
    Tools::make_face_from_wire_and_face(&wire, &source, &mut output)?;
    integer(output.orientation() as i32);
    flag(output.is_same(&source));
    integer(Explorer::new(&output, ShapeEnum::Edge, ShapeEnum::Shape)?.count() as i32);
    integer(Tools::classify_point_by_face(&face, Point2::new(v[0], v[1]))? as i32);
    Ok(())
}
fn run(v: &[f64]) -> Result<(), Failure> {
    let e = edge(0.)?;
    let other = edge(v[1])?;
    let vertex = topods::vertex(Explorer::new(&e, ShapeEnum::Vertex, ShapeEnum::Shape)?.current())?;
    let p = Point3::new(v[1], 0., v[0]);
    let external = BRepBuilderAPIMakeVertex::new(p)?.vertex()?;
    BRepBuilder::default().update_vertex_tolerance(&vertex, v[2])?;
    integer(Tools::compute_vv(&vertex, &external)?);
    flag(Tools::is_point_vertex(p, v[2], &vertex)?);
    flag(Tools::is_vertex_on_edge(&e, v[0])?);
    flag(Tools::is_edge_vertex(&e, &vertex, v[0])?);
    flag(Tools::is_middle_points_equal(&e, &other)?);
    let mut part = CommonPrt::default();
    part.set_edge1(&e);
    part.set_edge2(&other);
    part.set_range1_bounds(v[0], v[0]);
    part.append_range2_bounds(v[1], v[1]);
    flag(Tools::is_common_vertex(&part)?);
    wire_face(&e, v)
}
pub fn case(v: &[f64]) {
    if let Err(error) = run(v) {
        print!("{} ", error.exception_type());
    }
}
