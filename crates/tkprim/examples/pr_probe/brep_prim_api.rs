//! BRepPrimAPI and BRepPreviewAPI primitive probe operations, the Rust twin of
//! tools/oracle/tkprim_probe_brepprimapi.cxx (the records are documented there).
use super::brep_lib_common::geom_curve;
use super::brep_prim::{ax2, print_ax2, wedge_state};
use super::common::{Reader, attempt, end, flag, real};
use super::shape::{digest, shape};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Axis2, Direction3, Point3, Xyz};
use rustycad_tkprim::brep_preview_api::BRepPreviewAPIMakeBox;
use rustycad_tkprim::brep_prim::BRepPrimOneAxis;
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeBox, BRepPrimAPIMakeCone, BRepPrimAPIMakeCylinder, BRepPrimAPIMakeOneAxis,
    BRepPrimAPIMakeRevolution, BRepPrimAPIMakeSphere, BRepPrimAPIMakeTorus, BRepPrimAPIMakeWedge,
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

/// Prints the exception of a failed construction and ends the line.
fn failed(error: Failure) {
    print!("{} ", error.exception_type());
    end();
}

/// A box maker seen as its `BRepBuilderAPI_MakeShape` (whose `Build` may be overridden)
/// and as its `BRepPrimAPI_MakeBox` members.
trait BoxProbe {
    fn maker(&mut self) -> &mut dyn BRepBuilderAPIMakeShape;
    fn box_maker(&mut self) -> &mut BRepPrimAPIMakeBox;
}

impl BoxProbe for BRepPrimAPIMakeBox {
    fn maker(&mut self) -> &mut dyn BRepBuilderAPIMakeShape {
        self
    }
    fn box_maker(&mut self) -> &mut BRepPrimAPIMakeBox {
        self
    }
}

impl BoxProbe for BRepPreviewAPIMakeBox {
    fn maker(&mut self) -> &mut dyn BRepBuilderAPIMakeShape {
        self
    }
    fn box_maker(&mut self) -> &mut BRepPrimAPIMakeBox {
        self
    }
}

/// The arguments of a box call, read before the call.
#[derive(Default)]
struct BoxArgs {
    v: [f64; 3],
    p1: Point3,
    p2: Point3,
    frame: (Point3, Option<Direction3>, Xyz),
}

fn box_args(r: &mut Reader<'_>, op: i32) -> BoxArgs {
    let mut a = BoxArgs::default();
    if op == 12 || op == 13 {
        a.p1 = r.pnt();
    }
    if op == 13 {
        a.p2 = r.pnt();
    }
    if op == 14 {
        a.frame = (r.pnt(), Some(r.dir()), r.xyz());
    }
    if op == 11 || op == 12 || op == 14 {
        a.v = [r.real(), r.real(), r.real()];
    }
    a
}

/// The frame of `P N X` read ahead (pr_ax2).
fn frame((p, n, x): (Point3, Option<Direction3>, Xyz)) -> Result<Axis2, Failure> {
    let n = n.unwrap_or_default();
    if x.modulus() == 0.0 {
        return Axis2::new(p, n);
    }
    Axis2::with_x_direction(p, n, Direction3::from_xyz(x)?)
}

/// `pr_pa_box_call`.
fn box_call<B: BoxProbe>(b: &mut B, op: i32, a: BoxArgs) -> Result<(), Failure> {
    let [dx, dy, dz] = a.v;
    match op {
        0 => shape(b.maker().shape()?)?,
        1 => digest(b.box_maker().shell()?)?,
        2 => digest(b.box_maker().solid()?)?,
        3 => digest(b.box_maker().bottom_face()?)?,
        4 => digest(b.box_maker().back_face()?)?,
        5 => digest(b.box_maker().front_face()?)?,
        6 => digest(b.box_maker().left_face()?)?,
        7 => digest(b.box_maker().right_face()?)?,
        8 => digest(b.box_maker().top_face()?)?,
        9 => b.maker().build(&ProgressRange::default())?,
        10 => flag(b.maker().is_done()),
        11 => b.box_maker().init_sizes(dx, dy, dz)?,
        12 => b.box_maker().init_at_point(a.p1, dx, dy, dz)?,
        13 => b.box_maker().init_points(a.p1, a.p2)?,
        14 => {
            let axes = frame(a.frame)?;
            b.box_maker().init_axes(axes, dx, dy, dz);
        }
        _ => wedge_state(b.box_maker().wedge()),
    }
    Ok(())
}

/// `pr_pa_box_calls`.
fn box_calls<B: BoxProbe>(b: &mut B, r: &mut Reader<'_>) {
    let n = r.integer();
    for _ in 0..n {
        let op = r.integer();
        let args = box_args(r, op);
        attempt(|| box_call(b, op, args));
    }
}

/// The box of `kind [values]`.
fn make_box(r: &mut Reader<'_>, kind: i32) -> Result<BRepPrimAPIMakeBox, Failure> {
    match kind {
        0 => Ok(BRepPrimAPIMakeBox::new()),
        1 => BRepPrimAPIMakeBox::from_sizes(r.real(), r.real(), r.real()),
        2 => BRepPrimAPIMakeBox::at_point(r.pnt(), r.real(), r.real(), r.real()),
        3 => BRepPrimAPIMakeBox::from_points(r.pnt(), r.pnt()),
        _ => Ok(BRepPrimAPIMakeBox::with_axes(
            ax2(r)?,
            r.real(),
            r.real(),
            r.real(),
        )),
    }
}

/// `pa_box kind [values] ncalls (call)...`.
pub fn box_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let kind = r.integer();
    let mut b = match make_box(&mut r, kind) {
        Ok(b) => b,
        Err(error) => return failed(error),
    };
    box_calls(&mut b, &mut r);
    end();
}

/// `pv_box ncalls (call)...`.
pub fn preview_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut b = BRepPreviewAPIMakeBox::new();
    box_calls(&mut b, &mut r);
    end();
}

/// The wedge maker of `kind [frame] values...`.
fn make_wedge(r: &mut Reader<'_>, kind: i32) -> Result<BRepPrimAPIMakeWedge, Failure> {
    let axes = if kind == 1 || kind == 3 {
        Some(ax2(r)?)
    } else {
        None
    };
    let n = if kind < 2 { 4 } else { 7 };
    let v: Vec<f64> = (0..n).map(|_| r.real()).collect();
    let limits = || [v[3], v[4], v[5], v[6]];
    match (kind, axes) {
        (0, _) => BRepPrimAPIMakeWedge::with_ltx(v[0], v[1], v[2], v[3]),
        (1, Some(a)) => Ok(BRepPrimAPIMakeWedge::with_axes_ltx(
            a, v[0], v[1], v[2], v[3],
        )),
        (2, _) => BRepPrimAPIMakeWedge::with_limits(v[0], v[1], v[2], limits()),
        (_, Some(a)) => Ok(BRepPrimAPIMakeWedge::with_axes_limits(
            a,
            v[0],
            v[1],
            v[2],
            limits(),
        )),
        _ => Err(Failure::out_of_range("pa_wedge: kind")),
    }
}

/// One wedge maker call.
fn wedge_call(w: &mut BRepPrimAPIMakeWedge, op: i32) -> Result<(), Failure> {
    match op {
        0 => shape(w.shape()?)?,
        1 => digest(w.shell()?)?,
        2 => digest(w.solid()?)?,
        3 => w.build(&ProgressRange::default())?,
        4 => flag(w.is_done()),
        _ => wedge_state(w.wedge()),
    }
    Ok(())
}

/// `pa_wedge kind [frame] values... ncalls (op)...`.
pub fn wedge_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let kind = r.integer();
    let mut w = match make_wedge(&mut r, kind) {
        Ok(w) => w,
        Err(error) => return failed(error),
    };
    let n = r.integer();
    for _ in 0..n {
        let op = r.integer();
        attempt(|| wedge_call(&mut w, op));
    }
    end();
}

/// The one-axis makers of the probe, to reach their typed accessors.
enum OneAxisMaker {
    Cylinder(BRepPrimAPIMakeCylinder),
    Cone(BRepPrimAPIMakeCone),
    Sphere(BRepPrimAPIMakeSphere),
    Torus(BRepPrimAPIMakeTorus),
    Revolution(BRepPrimAPIMakeRevolution),
}

impl OneAxisMaker {
    fn maker(&mut self) -> &mut dyn BRepPrimAPIMakeOneAxis {
        match self {
            Self::Cylinder(m) => m,
            Self::Cone(m) => m,
            Self::Sphere(m) => m,
            Self::Torus(m) => m,
            Self::Revolution(m) => m,
        }
    }

    /// `pr_pa_typed`.
    fn typed(&mut self) -> &mut dyn BRepPrimOneAxis {
        match self {
            Self::Cylinder(m) => m.cylinder(),
            Self::Cone(m) => m.cone(),
            Self::Sphere(m) => m.sphere(),
            Self::Torus(m) => m.torus(),
            Self::Revolution(m) => m.revolution(),
        }
    }
}

/// The frame of the kinds from `first_framed` on.
fn optional_frame(r: &mut Reader<'_>, framed: bool) -> Result<Axis2, Failure> {
    if framed { ax2(r) } else { Ok(Axis2::default()) }
}

fn cylinder(r: &mut Reader<'_>, kind: i32) -> Result<OneAxisMaker, Failure> {
    let a = optional_frame(r, kind >= 2)?;
    let (radius, h) = (r.real(), r.real());
    let maker = match kind {
        0 => BRepPrimAPIMakeCylinder::new(radius, h),
        2 => BRepPrimAPIMakeCylinder::with_axes(a, radius, h),
        1 => BRepPrimAPIMakeCylinder::with_angle(radius, h, r.real())?,
        _ => BRepPrimAPIMakeCylinder::with_axes_angle(a, radius, h, r.real())?,
    };
    Ok(OneAxisMaker::Cylinder(maker))
}

fn cone(r: &mut Reader<'_>, kind: i32) -> Result<OneAxisMaker, Failure> {
    let a = optional_frame(r, kind >= 2)?;
    let (r1, r2, h) = (r.real(), r.real(), r.real());
    let maker = match kind {
        0 => BRepPrimAPIMakeCone::new(r1, r2, h)?,
        2 => BRepPrimAPIMakeCone::with_axes(a, r1, r2, h)?,
        1 => BRepPrimAPIMakeCone::with_angle(r1, r2, h, r.real())?,
        _ => BRepPrimAPIMakeCone::with_axes_angle(a, r1, r2, h, r.real())?,
    };
    Ok(OneAxisMaker::Cone(maker))
}

fn sphere_at_origin(radius: f64, v: &[f64]) -> Result<BRepPrimAPIMakeSphere, Failure> {
    match v.len() {
        0 => BRepPrimAPIMakeSphere::new(radius),
        1 => BRepPrimAPIMakeSphere::with_angle(radius, v[0]),
        2 => BRepPrimAPIMakeSphere::with_latitudes(radius, v[0], v[1]),
        _ => BRepPrimAPIMakeSphere::with_latitudes_angle(radius, v[0], v[1], v[2]),
    }
}

fn sphere_at_center(c: Point3, radius: f64, v: &[f64]) -> Result<BRepPrimAPIMakeSphere, Failure> {
    match v.len() {
        0 => BRepPrimAPIMakeSphere::at_center(c, radius),
        1 => BRepPrimAPIMakeSphere::at_center_angle(c, radius, v[0]),
        2 => BRepPrimAPIMakeSphere::at_center_latitudes(c, radius, v[0], v[1]),
        _ => BRepPrimAPIMakeSphere::at_center_latitudes_angle(c, radius, v[0], v[1], v[2]),
    }
}

fn sphere_on_axis(a: Axis2, radius: f64, v: &[f64]) -> Result<BRepPrimAPIMakeSphere, Failure> {
    match v.len() {
        0 => BRepPrimAPIMakeSphere::with_axis(a, radius),
        1 => BRepPrimAPIMakeSphere::with_axis_angle(a, radius, v[0]),
        2 => BRepPrimAPIMakeSphere::with_axis_latitudes(a, radius, v[0], v[1]),
        _ => BRepPrimAPIMakeSphere::with_axis_latitudes_angle(a, radius, v[0], v[1], v[2]),
    }
}

fn sphere(r: &mut Reader<'_>, kind: i32) -> Result<OneAxisMaker, Failure> {
    let a = optional_frame(r, kind >= 8)?;
    let c = if (4..8).contains(&kind) {
        r.pnt()
    } else {
        Point3::default()
    };
    let radius = r.real();
    let v: Vec<f64> = (0..kind % 4).map(|_| r.real()).collect();
    let maker = match kind {
        0..4 => sphere_at_origin(radius, &v)?,
        4..8 => sphere_at_center(c, radius, &v)?,
        _ => sphere_on_axis(a, radius, &v)?,
    };
    Ok(OneAxisMaker::Sphere(maker))
}

fn torus(r: &mut Reader<'_>, kind: i32) -> Result<OneAxisMaker, Failure> {
    let framed = kind >= 4;
    let a = optional_frame(r, framed)?;
    let (r1, r2) = (r.real(), r.real());
    let v: Vec<f64> = (0..kind % 4).map(|_| r.real()).collect();
    let maker = match (framed, v.len()) {
        (false, 0) => BRepPrimAPIMakeTorus::new(r1, r2)?,
        (false, 1) => BRepPrimAPIMakeTorus::with_angle(r1, r2, v[0])?,
        (false, 2) => BRepPrimAPIMakeTorus::with_pipe(r1, r2, v[0], v[1])?,
        (false, _) => BRepPrimAPIMakeTorus::with_pipe_angle(r1, r2, v[0], v[1], v[2])?,
        (true, 0) => BRepPrimAPIMakeTorus::with_axes(a, r1, r2)?,
        (true, 1) => BRepPrimAPIMakeTorus::with_axes_angle(a, r1, r2, v[0])?,
        (true, 2) => BRepPrimAPIMakeTorus::with_axes_pipe(a, r1, r2, v[0], v[1])?,
        (true, _) => BRepPrimAPIMakeTorus::with_axes_pipe_angle(a, r1, r2, v[0], v[1], v[2])?,
    };
    Ok(OneAxisMaker::Torus(maker))
}

fn revolution(r: &mut Reader<'_>, kind: i32) -> Result<OneAxisMaker, Failure> {
    let framed = kind >= 4;
    let a = optional_frame(r, framed)?;
    let m = geom_curve(r)?;
    let v: Vec<f64> = (0..kind % 4).map(|_| r.real()).collect();
    let maker = match (framed, v.len()) {
        (false, 0) => BRepPrimAPIMakeRevolution::new(&m)?,
        (false, 1) => BRepPrimAPIMakeRevolution::with_angle(&m, v[0])?,
        (false, 2) => BRepPrimAPIMakeRevolution::with_range(&m, v[0], v[1])?,
        (false, _) => BRepPrimAPIMakeRevolution::with_range_angle(&m, v[0], v[1], v[2])?,
        (true, 0) => BRepPrimAPIMakeRevolution::with_axes(a, &m)?,
        (true, 1) => BRepPrimAPIMakeRevolution::with_axes_angle(a, &m, v[0])?,
        (true, 2) => BRepPrimAPIMakeRevolution::with_axes_range(a, &m, v[0], v[1])?,
        (true, _) => BRepPrimAPIMakeRevolution::with_axes_range_angle(a, &m, v[0], v[1], v[2])?,
    };
    Ok(OneAxisMaker::Revolution(maker))
}

/// `pr_pa_oneaxis_state`.
fn one_axis_state(p: &dyn BRepPrimOneAxis) {
    real(p.angle());
    real(p.vmin());
    real(p.vmax());
    print_ax2(p.axes());
}

/// One one-axis maker call.
fn one_axis_call(m: &mut OneAxisMaker, op: i32) -> Result<(), Failure> {
    match op {
        0 => shape(m.maker().shape()?)?,
        1 => digest(m.maker().face()?)?,
        2 => digest(m.maker().shell()?)?,
        3 => digest(m.maker().solid()?)?,
        4 => m.maker().build(&ProgressRange::default())?,
        5 => flag(m.maker().is_done()),
        6 => one_axis_state(m.maker().one_axis()),
        _ => one_axis_state(m.typed()),
    }
    Ok(())
}

/// `pa_oneaxis maker kind values... ncalls (op)...`.
pub fn oneaxis_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (maker, kind) = (r.integer(), r.integer());
    let made = match maker {
        0 => cylinder(&mut r, kind),
        1 => cone(&mut r, kind),
        2 => sphere(&mut r, kind),
        3 => torus(&mut r, kind),
        _ => revolution(&mut r, kind),
    };
    let mut m = match made {
        Ok(m) => m,
        Err(error) => return failed(error),
    };
    let n = r.integer();
    for _ in 0..n {
        let op = r.integer();
        attempt(|| one_axis_call(&mut m, op));
    }
    end();
}
