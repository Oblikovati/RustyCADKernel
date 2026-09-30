//! BRepPrim one-axis probe operations, the Rust twin of
//! tools/oracle/tkprim_probe_brepprim_oneaxis.cxx (the records are documented in
//! tools/oracle/tkprim_probe_brepprim.cxx).
use super::brep_lib_common::{geom_curve, geom2d_curve};
use super::brep_prim::{ax2, print_ax2};
use super::common::{Reader, attempt, end, flag, pnt2d, real};
use super::shape::{digest, shape};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::ox;
use rustycad_tkprim::brep_prim::{
    BRepPrimCone, BRepPrimCylinder, BRepPrimOneAxis, BRepPrimRevolution, BRepPrimSphere,
    BRepPrimTorus,
};

type OneAxis = Box<dyn BRepPrimOneAxis>;

/// `pr_oneaxis_member`: the shapes 0..29 of the call record.
fn member(p: &mut dyn BRepPrimOneAxis, op: i32) -> Result<Shape, Failure> {
    Ok(match op {
        0 => p.shell()?.into(),
        1 => p.lateral_face()?.into(),
        2 => p.top_face()?.into(),
        3 => p.bottom_face()?.into(),
        4 => p.start_face()?.into(),
        5 => p.end_face()?.into(),
        6 => p.lateral_wire()?.into(),
        7 => p.lateral_start_wire()?.into(),
        8 => p.lateral_end_wire()?.into(),
        9 => p.top_wire()?.into(),
        10 => p.bottom_wire()?.into(),
        11 => p.start_wire()?.into(),
        12 => p.axis_start_wire()?.into(),
        13 => p.end_wire()?.into(),
        14 => p.axis_end_wire()?.into(),
        _ => return member_edge_or_vertex(p, op),
    })
}

/// The edges and vertices 15..29 of the call record.
fn member_edge_or_vertex(p: &mut dyn BRepPrimOneAxis, op: i32) -> Result<Shape, Failure> {
    Ok(match op {
        15 => p.axis_edge()?.into(),
        16 => p.start_edge()?.into(),
        17 => p.end_edge()?.into(),
        18 => p.start_top_edge()?.into(),
        19 => p.start_bottom_edge()?.into(),
        20 => p.end_top_edge()?.into(),
        21 => p.end_bottom_edge()?.into(),
        22 => p.top_edge()?.into(),
        23 => p.bottom_edge()?.into(),
        24 => p.axis_top_vertex()?.into(),
        25 => p.axis_bottom_vertex()?.into(),
        26 => p.top_start_vertex()?.into(),
        27 => p.top_end_vertex()?.into(),
        28 => p.bottom_start_vertex()?.into(),
        _ => p.bottom_end_vertex()?.into(),
    })
}

/// `pr_oneaxis_setter`: the setters 40..44.
fn setter(p: &mut dyn BRepPrimOneAxis, op: i32, arg: f64) -> Result<(), Failure> {
    match op {
        40 => p.set_angle(arg),
        41 => p.set_vmin(arg),
        42 => p.set_vmax(arg),
        43 => {
            p.set_meridian_offset(arg);
            Ok(())
        }
        _ => {
            let axes = p.axes().rotated(ox(), arg)?;
            p.set_axes(axes)
        }
    }
}

/// `pr_oneaxis_query`: the queries 50..61.
fn query(p: &mut dyn BRepPrimOneAxis, op: i32, arg: f64) -> Result<(), Failure> {
    match op {
        50 => pnt2d(p.meridian_value(arg)?),
        51 => flag(p.meridian_on_axis(arg)?),
        52 => flag(p.meridian_closed()?),
        53 => flag(p.vmax_infinite()),
        54 => flag(p.vmin_infinite()),
        55 => flag(p.has_top()?),
        56 => flag(p.has_bottom()?),
        57 => flag(p.has_sides()),
        58 => real(p.angle()),
        59 => real(p.vmin()),
        60 => real(p.vmax()),
        _ => print_ax2(p.axes()),
    }
    Ok(())
}

/// `pr_oneaxis_call`.
fn call(p: &mut dyn BRepPrimOneAxis, op: i32, arg: f64) -> Result<(), Failure> {
    match op {
        0..30 => shape(&member(p, op)?),
        30 => digest(p.make_empty_lateral_face()?),
        31 => digest(p.make_empty_meridian_edge(arg)?),
        32 => {
            let e = p.make_empty_meridian_edge(arg)?;
            let f = p.make_empty_lateral_face()?;
            p.set_meridian_pcurve(&e, &f)?;
            digest(&e)
        }
        33..50 => setter(p, op, arg),
        _ => query(p, op, arg),
    }
}

fn cylinder(r: &mut Reader<'_>, ctor: i32) -> Result<OneAxis, Failure> {
    let cylinder = match ctor {
        0 => BRepPrimCylinder::new(ax2(r)?, r.real(), r.real()),
        1 => BRepPrimCylinder::infinite(r.real()),
        2 => BRepPrimCylinder::infinite_at(r.pnt(), r.real())?,
        3 => BRepPrimCylinder::infinite_on(ax2(r)?, r.real()),
        4 => BRepPrimCylinder::from_radius_height(r.real(), r.real()),
        _ => BRepPrimCylinder::at_center(r.pnt(), r.real(), r.real())?,
    };
    Ok(Box::new(cylinder))
}

fn cone(r: &mut Reader<'_>, ctor: i32) -> Result<OneAxis, Failure> {
    let cone = match ctor {
        0 => {
            let angle = r.real();
            BRepPrimCone::new(angle, ax2(r)?, r.real(), r.real())?
        }
        1 => BRepPrimCone::infinite(r.real())?,
        2 => BRepPrimCone::infinite_at(r.real(), r.pnt())?,
        4 => BRepPrimCone::from_radii(r.real(), r.real(), r.real())?,
        5 => BRepPrimCone::at_center(r.pnt(), r.real(), r.real(), r.real())?,
        _ => BRepPrimCone::with_axes_radii(ax2(r)?, r.real(), r.real(), r.real())?,
    };
    Ok(Box::new(cone))
}

fn sphere(r: &mut Reader<'_>, ctor: i32) -> Result<OneAxis, Failure> {
    let sphere = match ctor {
        0 => BRepPrimSphere::new(r.real())?,
        1 => BRepPrimSphere::at_center(r.pnt(), r.real())?,
        _ => BRepPrimSphere::with_axes(ax2(r)?, r.real())?,
    };
    Ok(Box::new(sphere))
}

fn torus(r: &mut Reader<'_>, ctor: i32) -> Result<OneAxis, Failure> {
    let torus = match ctor {
        0 => BRepPrimTorus::with_axes(ax2(r)?, r.real(), r.real())?,
        1 => BRepPrimTorus::new(r.real(), r.real())?,
        _ => BRepPrimTorus::at_center(r.pnt(), r.real(), r.real())?,
    };
    Ok(Box::new(torus))
}

fn revolution(r: &mut Reader<'_>) -> Result<OneAxis, Failure> {
    let a = ax2(r)?;
    let (vmin, vmax) = (r.real(), r.real());
    let m = geom_curve(r)?;
    let pm = geom2d_curve(r)?;
    Ok(Box::new(BRepPrimRevolution::new(a, vmin, vmax, m, pm)))
}

/// `pr_one_axis`.
fn one_axis(r: &mut Reader<'_>, kind: i32, ctor: i32) -> Result<OneAxis, Failure> {
    match kind {
        0 => cylinder(r, ctor),
        1 => cone(r, ctor),
        2 => sphere(r, ctor),
        3 => torus(r, ctor),
        _ => revolution(r),
    }
}

/// `bp_oneaxis kind ctor fields... ncalls (op arg)...`.
pub fn oneaxis_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, ctor) = (r.integer(), r.integer());
    let mut p = match one_axis(&mut r, kind, ctor) {
        Ok(p) => p,
        Err(error) => {
            print!("{} ", error.exception_type());
            end();
            return;
        }
    };
    let n = r.integer();
    for _ in 0..n {
        let (op, arg) = (r.integer(), r.real());
        if op == 99 {
            end();
            return;
        }
        attempt(|| call(p.as_mut(), op, arg));
    }
    attempt(|| digest(p.shell()?));
    end();
}
