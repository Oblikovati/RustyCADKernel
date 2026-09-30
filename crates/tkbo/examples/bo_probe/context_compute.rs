use super::{curves, fclass_faces, flag, integer, real};
use rustycad_tkbo::int_tools::{Context, Curve, Tools};
use rustycad_tkbrep::{brep::BRepBuilder, topods::Edge};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkmath::gp::{Axis2d, Point3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeVertex};
fn edge(kind: i32) -> Result<Edge, Failure> {
    if kind == 0 {
        return BRepBuilderAPIMakeEdge::from_points(Point3::default(), Point3::new(0., 0., 2.))?
            .edge();
    }
    let mut edge = Edge::default();
    BRepBuilder::default().make_edge(&mut edge)?;
    if kind == 2 {
        BRepBuilder::default().degenerated(&edge, true)?;
    }
    Ok(edge)
}
fn computations(c: &Context, v: &[f64], p: Point3) -> Result<(), Failure> {
    let edge = edge(v[0] as i32)?;
    let vertex = BRepBuilderAPIMakeVertex::new(p)?.vertex()?;
    let (mut t, mut d) = (31., 32.);
    integer(c.compute_pe(p, v[4], &edge, &mut t, &mut d)?);
    real(t);
    real(d);
    (t, d) = (31., 32.);
    integer(c.compute_ve(&vertex, &edge, &mut t, &mut d, v[5])?);
    real(t);
    real(d);
    face_computation(c, &vertex, v[5])?;
    if v[0] == 0. {
        t = 31.;
        flag(c.project_point_on_edge(p, &edge, &mut t)?);
        real(t);
    }
    Ok(())
}
fn face_computation(
    c: &Context,
    vertex: &rustycad_tkbrep::topods::Vertex,
    fuzz: f64,
) -> Result<(), Failure> {
    let f = fclass_faces::face(0, 2.)?;
    let (mut u, mut w) = (33., 34.);
    let mut d = 32.;
    integer(c.compute_vf(vertex, &f, &mut u, &mut w, &mut d, fuzz)?);
    real(u);
    real(w);
    real(d);
    Ok(())
}

fn curves_case(c: &Context, v: &[f64], p: Point3) -> Result<(), Failure> {
    let geom = curves::geometry(v[6] as i32);
    let mut curve = Curve::default();
    curve.set_curve(&geom);
    let vertex = BRepBuilderAPIMakeVertex::new(p)?.vertex()?;
    let mut t = 31.;
    flag(c.is_vertex_on_line(&vertex, &curve, v[4], &mut t)?);
    real(t);
    t = 31.;
    flag(c.is_vertex_on_line_tolerance(&vertex, v[5], &curve, v[4], &mut t)?);
    real(t);
    blocks(c, v, curve)
}
fn blocks(c: &Context, v: &[f64], mut curve: Curve) -> Result<(), Failure> {
    let f = fclass_faces::face(0, 2.)?;
    real(Tools::intermediate_point(v[7], v[8]));
    flag(c.is_valid_block_for_face(v[7], v[8], &curve, &f, v[4])?);
    let pc: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::default())).into_cast();
    let null = Handle::default();
    for mask in 0..4 {
        curve.set_first_curve2d(if mask & 1 != 0 { &pc } else { &null });
        curve.set_second_curve2d(if mask & 2 != 0 { &pc } else { &null });
        flag(c.is_valid_block_for_faces(v[7], v[8], &curve, &f, &f, v[4])?);
    }
    Ok(())
}
pub fn case(v: &[f64]) {
    let c = Context::default();
    let p = Point3::new(v[1], v[2], v[3]);
    if let Err(error) = computations(&c, v, p).and_then(|()| curves_case(&c, v, p)) {
        print!("{} ", error.exception_type());
    }
}
