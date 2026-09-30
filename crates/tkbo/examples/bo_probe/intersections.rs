use super::{flag, integer, range, real};
use rustycad_tkbo::int_tools::*;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Edge, Face},
};
use rustycad_tkmath::gp::Point3;
fn point(value: Point3) {
    real(value.x());
    real(value.y());
    real(value.z());
}
fn face_point_state(value: &PntOnFace) {
    flag(value.valid());
    flag(value.face().is_null());
    point(*value.pnt());
    let (u, v) = value.parameters();
    real(u);
    real(v);
}
fn pair_state(value: &PntOn2Faces) {
    flag(value.is_valid());
    face_point_state(value.p1());
    face_point_state(value.p2());
}
pub fn points_case(v: &[f64]) {
    let mut first = PntOnFace::default();
    face_point_state(&first);
    let mut face = Face::default();
    BRepBuilder::default().make_face(&mut face);
    first.init(&face, Point3::new(v[0], v[1], v[2]), v[3], v[4]);
    face_point_state(&first);
    flag(first.face().is_same(&face));
    first.set_valid(v[5] != 0.0);
    first.set_face(&Face::default());
    first.set_pnt(Point3::new(v[2], v[1], v[0]));
    first.set_parameters(v[4], v[3]);
    face_point_state(&first);
    pair_case(&first);
}
fn pair_case(first: &PntOnFace) {
    let second = PntOnFace::default();
    pair_state(&PntOn2Faces::default());
    let mut pair = PntOn2Faces::new(first, &second);
    pair_state(&pair);
    pair.set_p1(&second);
    pair.set_p2(first);
    pair.set_valid(true);
    pair_state(&pair);
}
fn common_state(part: &CommonPrt) {
    flag(part.edge1().is_null());
    flag(part.edge2().is_null());
    integer(part.shape_type() as i32);
    range(*part.range1());
    let (first, last) = part.range1_bounds();
    real(first);
    real(last);
    real(part.vertex_parameter1());
    real(part.vertex_parameter2());
    flag(part.all_null_flag());
    let (first, last) = part.bounding_points();
    point(first);
    point(last);
    integer(part.ranges2().len() as i32);
    for r in part.ranges2() {
        range(*r);
    }
}
fn configured_common(v: &[f64]) -> CommonPrt {
    let mut part = CommonPrt::default();
    common_state(&part);
    let mut edge = Edge::default();
    BRepBuilder::default().make_edge(&mut edge).unwrap();
    part.set_edge1(&edge);
    part.set_edge2(&edge);
    flag(part.edge1().is_same(&edge));
    flag(part.edge2().is_same(&edge));
    part.set_type(rustycad_tkg3d::top_abs::ShapeEnum::Edge);
    part.set_range1(Range::new(v[0], v[1]));
    common_state(&part);
    configure_intervals(&mut part, v);
    part
}
fn configure_intervals(part: &mut CommonPrt, v: &[f64]) {
    part.set_range1_bounds(v[2], v[3]);
    part.append_range2(Range::new(v[0], v[2]));
    part.append_range2_bounds(v[1], v[3]);
    part.set_vertex_parameter1(v[4]);
    part.set_vertex_parameter2(v[5]);
    part.set_all_null_flag(true);
    part.set_bounding_points(Point3::new(v[0], v[1], v[2]), Point3::new(v[3], v[4], v[5]));
}
pub fn common_case(v: &[f64]) {
    let part = configured_common(v);
    common_state(&part);
    let mut clone = part.clone();
    common_state(&clone);
    clone.assign(&part);
    common_state(&clone);
    clone.assign(&part);
    common_state(&clone);
    let mut target = CommonPrt::default();
    target.append_range2_bounds(-9.0, -8.0);
    part.copy_to(&mut target);
    common_state(&target);
    target.change_ranges2().clear();
    common_state(&target);
}
