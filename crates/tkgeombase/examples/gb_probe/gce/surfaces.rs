//! The gce cone, cylinder and plane cases of tools/oracle/tkgeombase_probe_gce.cxx.
use super::super::common::{self as p, Reader, end};
use super::{gce_root, ok};
use rustycad_tkgeombase::gce::{MakeCone, MakeCylinder, MakePln, Root};
use rustycad_tkmath::gp::{Circle3, Cone3, Cylinder3, Line3, Plane3};

pub fn cone_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => {
            let (a, g, d) = (r.ax2(), r.real(), r.real());
            gce_root!(p::cone, MakeCone::new(a, g, d));
        }
        k @ (1 | 2) => {
            let (a, g, r0) = (r.ax3(), r.real(), r.real());
            let c = Cone3::new(a, g, r0).unwrap();
            if k == 1 {
                let q = r.pnt();
                gce_root!(p::cone, MakeCone::from_cone_point(c, q));
            } else {
                let d = r.real();
                gce_root!(p::cone, MakeCone::from_cone_distance(c, d));
            }
        }
        3 => {
            let (a, b, c, d) = (r.pnt(), r.pnt(), r.pnt(), r.pnt());
            gce_root!(p::cone, MakeCone::through_points(a, b, c, d));
        }
        4 => {
            let (a, q1, q2) = (r.ax1(), r.pnt(), r.pnt());
            gce_root!(p::cone, MakeCone::from_axis_points(a, q1, q2));
        }
        5 => {
            let (a, q1, q2) = (r.ax1(), r.pnt(), r.pnt());
            gce_root!(
                p::cone,
                MakeCone::from_line_points(Line3::from_axis(a), q1, q2)
            );
        }
        _ => {
            let (a, b, r1, r2) = (r.pnt(), r.pnt(), r.real(), r.real());
            gce_root!(p::cone, MakeCone::from_points_radii(a, b, r1, r2));
        }
    }
    end();
}

pub fn cylinder_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => {
            let (a, d) = (r.ax2(), r.real());
            gce_root!(p::cylinder, MakeCylinder::new(a, d));
        }
        k @ (1 | 2) => {
            let (a, r0) = (r.ax3(), r.real());
            let c = Cylinder3::new(a, r0).unwrap();
            if k == 1 {
                let q = r.pnt();
                gce_root!(p::cylinder, MakeCylinder::from_cylinder_point(c, q));
            } else {
                let d = r.real();
                gce_root!(p::cylinder, MakeCylinder::from_cylinder_distance(c, d));
            }
        }
        3 => {
            let (a, b, c) = (r.pnt(), r.pnt(), r.pnt());
            gce_root!(p::cylinder, MakeCylinder::through_points(a, b, c));
        }
        4 => {
            let (a, d) = (r.ax1(), r.real());
            gce_root!(p::cylinder, MakeCylinder::from_axis(a, d));
        }
        _ => {
            let (a, r0) = (r.ax2(), r.real());
            let circle = Circle3::new(a, r0).unwrap();
            gce_root!(p::cylinder, MakeCylinder::from_circ(circle));
        }
    }
    end();
}

pub fn pln_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => {
            let a = r.ax2();
            gce_root!(p::pln, ok(MakePln::from_frame(a)));
        }
        1 => {
            let (q, d) = (r.pnt(), r.dir());
            gce_root!(p::pln, MakePln::new(q, d));
        }
        2 => {
            let (a, b, c, d) = (r.real(), r.real(), r.real(), r.real());
            gce_root!(p::pln, MakePln::from_equation(a, b, c, d));
        }
        3 => {
            let (a, q) = (r.ax3(), r.pnt());
            gce_root!(
                p::pln,
                MakePln::from_plane_point(Plane3::from_position(a), q)
            );
        }
        4 => {
            let (a, d) = (r.ax3(), r.real());
            gce_root!(
                p::pln,
                MakePln::from_plane_distance(Plane3::from_position(a), d)
            );
        }
        5 => {
            let (a, b, c) = (r.pnt(), r.pnt(), r.pnt());
            gce_root!(p::pln, MakePln::through_points(a, b, c));
        }
        6 => {
            let (a, b) = (r.pnt(), r.pnt());
            gce_root!(p::pln, MakePln::from_points(a, b));
        }
        _ => {
            let a = r.ax1();
            gce_root!(p::pln, MakePln::from_axis(a));
        }
    }
    end();
}
