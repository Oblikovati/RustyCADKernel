//! BRepPrim probe operations, the Rust twin of tools/oracle/tkprim_probe_brepprim.cxx
//! (the records are documented there).
use super::common::{Reader, attempt, dir, end, flag, pnt, real};
use super::shape::{digest, shape};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Edge, Face, Shell, Vertex, Wire};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{
    Axis2, Axis2d, Circle2, Circle3, Direction2, Direction3, Line2, Line3, Plane3, Point2, Point3,
    Vector3,
};
use rustycad_tkprim::brep_prim::{
    BRepPrimBuilder, BRepPrimDirection, BRepPrimGWedge, BRepPrimWedge,
};
use std::f64::consts::PI;

/// `pr_ax2`: `P N X`, the automatic X direction when X is the null vector.
pub fn ax2(r: &mut Reader<'_>) -> Result<Axis2, Failure> {
    let (p, n, x) = (r.pnt(), r.dir(), r.xyz());
    if x.modulus() == 0.0 {
        return Axis2::new(p, n);
    }
    Axis2::with_x_direction(p, n, Direction3::from_xyz(x)?)
}

/// `pr_print_ax2`.
pub fn print_ax2(a: Axis2) {
    pnt(a.location());
    dir(a.direction());
    dir(a.x_direction());
}

fn direction(value: f64) -> BRepPrimDirection {
    BRepPrimDirection::ALL[value as usize]
}

/// `pr_wedge_call`.
fn wedge_call(
    w: &mut BRepPrimGWedge,
    op: i32,
    [a, b, c]: [BRepPrimDirection; 3],
) -> Result<(), Failure> {
    match op {
        0 => digest(w.shell()?)?,
        1 => digest(w.face(a)?)?,
        2 => digest(w.wire(a)?)?,
        3 => digest(w.edge(a, b)?)?,
        4 => digest(w.vertex(a, b, c)?)?,
        5 => w.open(a)?,
        6 => w.close(a)?,
        7 => {
            let p = w.plane(a)?.position();
            pnt(p.location());
            dir(p.direction());
            dir(p.x_direction());
            dir(p.y_direction());
        }
        8 => {
            let l = w.line(a, b)?;
            pnt(l.location());
            dir(l.direction());
        }
        9 => pnt(w.point(a, b, c)?),
        10 => flag(w.has_face(a)),
        11 => flag(w.has_wire(a)?),
        12 => flag(w.has_edge(a, b)?),
        13 => flag(w.has_vertex(a, b, c)),
        _ => flag(w.is_infinite(a)),
    }
    Ok(())
}

/// `pr_wedge_state`.
pub fn wedge_state(w: &BRepPrimGWedge) {
    print_ax2(w.axes());
    for value in [
        w.get_xmin(),
        w.get_ymin(),
        w.get_zmin(),
        w.get_z2min(),
        w.get_x2min(),
        w.get_xmax(),
        w.get_ymax(),
        w.get_zmax(),
        w.get_z2max(),
        w.get_x2max(),
    ] {
        real(value);
    }
    flag(w.is_degenerated_shape());
}

/// The wedge of `kind [frame] values...`.
fn wedge(r: &mut Reader<'_>, kind: i32) -> Result<BRepPrimGWedge, Failure> {
    if kind == 0 {
        return Ok(std::mem::take(&mut *BRepPrimWedge::new()));
    }
    let a = ax2(r)?;
    let mut wedge = match kind {
        1 => BRepPrimWedge::with_box(a, r.real(), r.real(), r.real()),
        2 => BRepPrimWedge::with_ltx(a, r.real(), r.real(), r.real(), r.real()),
        3 => {
            let v: Vec<f64> = (0..10).map(|_| r.real()).collect();
            BRepPrimWedge::with_limits(
                a,
                [v[0], v[1], v[2], v[3], v[4]],
                [v[5], v[6], v[7], v[8], v[9]],
            )
        }
        _ => {
            let builder = BRepPrimBuilder::with_builder(BRepBuilder::default());
            return Ok(BRepPrimGWedge::with_box(
                builder,
                a,
                r.real(),
                r.real(),
                r.real(),
            ));
        }
    };
    Ok(std::mem::take(&mut *wedge))
}

/// `bp_wedge kind [frame] values... ncalls (op a b c)...`.
pub fn wedge_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let kind = r.integer();
    let mut w = match wedge(&mut r, kind) {
        Ok(w) => w,
        Err(error) => {
            print!("{} ", error.exception_type());
            end();
            return;
        }
    };
    let n = r.integer();
    for _ in 0..n {
        let op = r.integer();
        let dirs = [
            direction(r.real()),
            direction(r.real()),
            direction(r.real()),
        ];
        if op == 15 {
            end();
            return;
        }
        attempt(|| wedge_call(&mut w, op, dirs));
    }
    wedge_state(&w);
    attempt(|| digest(w.shell()?));
    end();
}

/// `pr_builder_edges`: the line edge P1-P2 and the seam edge on `face`.
fn builder_edges(
    b: &BRepPrimBuilder,
    face: &Face,
    a: Axis2,
    p1: Point3,
    p2: Point3,
) -> Result<(Edge, Edge), Failure> {
    let mut line = Edge::default();
    b.make_edge_line(
        &mut line,
        Line3::new(p1, Direction3::from_vector(Vector3::between(p1, p2))?),
    )?;
    let (mut v1, mut v2) = (Vertex::default(), Vertex::default());
    b.make_vertex(&mut v1, p1)?;
    b.make_vertex(&mut v2, p2)?;
    b.add_edge_vertex(&mut line, &v1, 0.0, true)?;
    b.add_edge_vertex(&mut line, &v2, p1.distance(&p2), false)?;
    b.set_pcurve_line(
        &line,
        face,
        Line2::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0)?),
    )?;
    b.complete_edge(&line);
    let mut seam = Edge::default();
    b.make_edge_line(&mut seam, Line3::new(a.location(), a.x_direction()))?;
    let first = Line2::new(Point2::new(0.0, 1.0), Direction2::new(0.0, 1.0)?);
    let last = Line2::new(Point2::new(1.0, 0.0), Direction2::new(0.0, 1.0)?);
    b.set_pcurve_lines(&seam, face, first, last)?;
    Ok((line, seam))
}

/// The body of `bp_builder`.
fn builder_body(
    a: Axis2,
    p1: Point3,
    p2: Point3,
    [radius, first, last]: [f64; 3],
) -> Result<(), Failure> {
    let b = BRepPrimBuilder::with_builder(BRepBuilder::default());
    let (mut sh, mut f, mut w) = (Shell::default(), Face::default(), Wire::default());
    let (mut degenerated, mut circle) = (Edge::default(), Edge::default());
    b.make_shell(&mut sh)?;
    flag(sh.closed()?);
    b.make_face(&mut f, Plane3::from_position(a.axis3()))?;
    b.make_wire(&mut w);
    b.make_degenerated_edge(&mut degenerated)?;
    let (line, seam) = builder_edges(&b, &f, a, p1, p2)?;
    b.make_edge_circle(&mut circle, Circle3::new(a, radius)?)?;
    let mut v = Vertex::default();
    b.make_vertex(&mut v, elclib::circle_value(0.0, a, radius))?;
    b.add_edge_vertex_closed(&mut circle, &v, 0.0, 2.0 * PI)?;
    b.set_parameters(&circle, &v, first, last)?;
    let x_axis = Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0)?);
    b.set_pcurve_circle(&circle, &f, Circle2::from_x_axis(x_axis, radius, true)?)?;
    b.complete_edge(&circle);
    b.add_wire_edge(&mut w, &circle, true)?;
    b.complete_wire(&w)?;
    b.reverse_face(&mut f);
    b.add_face_wire(&mut f, &w)?;
    b.complete_face(&f)?;
    b.add_shell_face(&mut sh, &f)?;
    b.complete_shell(&sh)?;
    for s in [
        sh.as_ref(),
        degenerated.as_ref(),
        line.as_ref(),
        seam.as_ref(),
    ] {
        shape(s)?;
    }
    Ok(())
}

/// `bp_builder frame P1 P2 radius first last`.
pub fn builder_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let a = ax2(&mut r);
    let (p1, p2) = (r.pnt(), r.pnt());
    let numbers = [r.real(), r.real(), r.real()];
    attempt(|| builder_body(a?, p1, p2, numbers));
    end();
}
