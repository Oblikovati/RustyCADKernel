//! The gce circle, conic, direction and line cases of tools/oracle/tkgeombase_probe_gce.cxx.
use super::super::common::{self as p, Reader, end, int};
use super::{gce_root, ok};
use rustycad_tkgeombase::gce::{
    ErrorType, MakeCirc, MakeCirc2d, MakeDir, MakeDir2d, MakeElips, MakeElips2d, MakeHypr,
    MakeHypr2d, MakeLin, MakeLin2d, MakeParab, MakeParab2d, Root,
};
use rustycad_tkmath::gp::{Circle2, Circle3, Line2, Line3, Plane3};

pub fn circ_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => {
            let (a, d) = (r.ax2(), r.real());
            gce_root!(p::circ, MakeCirc::new(a, d));
        }
        k @ (1 | 2) => {
            let (a, r0) = (r.ax2(), r.real());
            let c = Circle3::new(a, r0).unwrap();
            if k == 1 {
                let d = r.real();
                gce_root!(p::circ, MakeCirc::from_circ_distance(c, d));
            } else {
                let q = r.pnt();
                gce_root!(p::circ, MakeCirc::from_circ_point(c, q));
            }
        }
        3 => {
            let (a, b, c) = (r.pnt(), r.pnt(), r.pnt());
            gce_root!(p::circ, MakeCirc::through_points(a, b, c));
        }
        4 => {
            let (c, n, d) = (r.pnt(), r.dir(), r.real());
            gce_root!(p::circ, MakeCirc::from_center_normal(c, n, d));
        }
        5 => {
            let (c, o, n, d) = (r.pnt(), r.pnt(), r.dir(), r.real());
            let plane = Plane3::new(o, n).unwrap();
            gce_root!(p::circ, MakeCirc::from_center_plane(c, plane, d));
        }
        6 => {
            let (c, q, d) = (r.pnt(), r.pnt(), r.real());
            gce_root!(p::circ, MakeCirc::from_center_axis_point(c, q, d));
        }
        _ => {
            let (a, d) = (r.ax1(), r.real());
            gce_root!(p::circ, MakeCirc::from_axis(a, d));
        }
    }
    end();
}

pub fn circ2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => {
            let (a, d, s) = (r.ax2d(), r.real(), r.flag());
            gce_root!(p::circ2d, MakeCirc2d::new(a, d, s));
        }
        1 => {
            let (a, d) = (r.ax22d(), r.real());
            gce_root!(p::circ2d, MakeCirc2d::from_frame(a, d));
        }
        k @ (2 | 3) => {
            let (a, r0) = (r.ax22d(), r.real());
            let c = Circle2::new(a, r0).unwrap();
            if k == 2 {
                let d = r.real();
                gce_root!(p::circ2d, MakeCirc2d::from_circ_distance(c, d));
            } else {
                let q = r.pnt2d();
                gce_root!(p::circ2d, MakeCirc2d::from_circ_point(c, q));
            }
        }
        4 => {
            let (a, b, c) = (r.pnt2d(), r.pnt2d(), r.pnt2d());
            gce_root!(p::circ2d, MakeCirc2d::through_points(a, b, c));
        }
        5 => {
            let (c, d, s) = (r.pnt2d(), r.real(), r.flag());
            gce_root!(p::circ2d, MakeCirc2d::from_center(c, d, s));
        }
        _ => {
            let (c, q, s) = (r.pnt2d(), r.pnt2d(), r.flag());
            gce_root!(p::circ2d, MakeCirc2d::from_center_point(c, q, s));
        }
    }
    end();
}

pub fn dir_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => {
            let v = r.vec();
            gce_root!(p::dir, MakeDir::from_vector(v));
        }
        1 => {
            let v = r.xyz();
            gce_root!(p::dir, MakeDir::from_xyz(v));
        }
        2 => {
            let (x, y, z) = (r.real(), r.real(), r.real());
            gce_root!(p::dir, MakeDir::from_coordinates(x, y, z));
        }
        _ => {
            let (a, b) = (r.pnt(), r.pnt());
            gce_root!(p::dir, MakeDir::from_points(a, b));
        }
    }
    end();
}

pub fn dir2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => {
            let v = r.vec2d();
            gce_root!(p::dir2d, MakeDir2d::from_vector(v));
        }
        1 => {
            let v = r.xy();
            gce_root!(p::dir2d, MakeDir2d::from_xy(v));
        }
        2 => {
            let (x, y) = (r.real(), r.real());
            gce_root!(p::dir2d, MakeDir2d::from_coordinates(x, y));
        }
        _ => {
            let (a, b) = (r.pnt2d(), r.pnt2d());
            gce_root!(p::dir2d, MakeDir2d::from_points(a, b));
        }
    }
    end();
}

pub fn elips_case(values: &[f64]) {
    let mut r = Reader::new(values);
    if r.integer() == 0 {
        let (a, m, n) = (r.ax2(), r.real(), r.real());
        gce_root!(p::elips, MakeElips::new(a, m, n));
    } else {
        let (a, b, c) = (r.pnt(), r.pnt(), r.pnt());
        gce_root!(p::elips, MakeElips::from_points(a, b, c));
    }
    end();
}

pub fn elips2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => {
            let (a, m, n, s) = (r.ax2d(), r.real(), r.real(), r.flag());
            gce_root!(p::elips2d, MakeElips2d::new(a, m, n, s));
        }
        1 => {
            let (a, m, n) = (r.ax22d(), r.real(), r.real());
            gce_root!(p::elips2d, MakeElips2d::from_frame(a, m, n));
        }
        _ => {
            let (a, b, c) = (r.pnt2d(), r.pnt2d(), r.pnt2d());
            gce_root!(p::elips2d, MakeElips2d::from_points(a, b, c));
        }
    }
    end();
}

pub fn hypr_case(values: &[f64]) {
    let mut r = Reader::new(values);
    if r.integer() == 0 {
        let (a, m, n) = (r.ax2(), r.real(), r.real());
        gce_root!(p::hypr, MakeHypr::new(a, m, n));
    } else {
        let (a, b, c) = (r.pnt(), r.pnt(), r.pnt());
        gce_root!(p::hypr, MakeHypr::from_points(a, b, c));
    }
    end();
}

pub fn hypr2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => {
            let (a, b, c) = (r.pnt2d(), r.pnt2d(), r.pnt2d());
            gce_root!(p::hypr2d, MakeHypr2d::from_points(a, b, c));
        }
        1 => {
            let (a, m, n, s) = (r.ax2d(), r.real(), r.real(), r.flag());
            gce_root!(p::hypr2d, MakeHypr2d::new(a, m, n, s));
        }
        _ => {
            let (a, m, n) = (r.ax22d(), r.real(), r.real());
            gce_root!(p::hypr2d, MakeHypr2d::from_frame(a, m, n));
        }
    }
    end();
}

pub fn lin_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => {
            let a = r.ax1();
            gce_root!(p::lin, ok(MakeLin::from_axis(a)));
        }
        1 => {
            let (q, d) = (r.pnt(), r.dir());
            gce_root!(p::lin, ok(MakeLin::new(q, d)));
        }
        2 => {
            let (a, b) = (r.pnt(), r.pnt());
            gce_root!(p::lin, MakeLin::from_points(a, b));
        }
        _ => {
            let (a, q) = (r.ax1(), r.pnt());
            gce_root!(p::lin, ok(MakeLin::from_line_point(Line3::from_axis(a), q)));
        }
    }
    end();
}

pub fn lin2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => {
            let a = r.ax2d();
            gce_root!(p::lin2d, ok(MakeLin2d::from_axis(a)));
        }
        1 => {
            let (q, d) = (r.pnt2d(), r.dir2d());
            gce_root!(p::lin2d, ok(MakeLin2d::new(q, d)));
        }
        2 => {
            let (a, b, c) = (r.real(), r.real(), r.real());
            gce_root!(p::lin2d, MakeLin2d::from_equation(a, b, c));
        }
        3 => {
            let (a, d) = (r.ax2d(), r.real());
            let line = Line2::new(a.location(), a.direction());
            gce_root!(p::lin2d, ok(MakeLin2d::from_line_distance(line, d)));
        }
        4 => {
            let (a, q) = (r.ax2d(), r.pnt2d());
            let line = Line2::new(a.location(), a.direction());
            gce_root!(p::lin2d, ok(MakeLin2d::from_line_point(line, q)));
        }
        _ => {
            let (a, b) = (r.pnt2d(), r.pnt2d());
            gce_root!(p::lin2d, MakeLin2d::from_points(a, b));
        }
    }
    end();
}

pub fn parab_case(values: &[f64]) {
    let mut r = Reader::new(values);
    if r.integer() == 0 {
        let (a, f) = (r.ax2(), r.real());
        gce_root!(p::parab, MakeParab::new(a, f));
    } else {
        let (a, f) = (r.ax1(), r.pnt());
        gce_root!(p::parab, MakeParab::from_directrix(a, f));
    }
    end();
}

pub fn parab2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => {
            let (a, f, s) = (r.ax2d(), r.real(), r.flag());
            gce_root!(p::parab2d, MakeParab2d::new(a, f, s));
        }
        1 => {
            let (a, f) = (r.ax22d(), r.real());
            gce_root!(p::parab2d, MakeParab2d::from_frame(a, f));
        }
        2 => {
            let (a, f, s) = (r.ax2d(), r.pnt2d(), r.flag());
            gce_root!(p::parab2d, MakeParab2d::from_directrix(a, f, s));
        }
        _ => {
            let (a, c, s) = (r.pnt2d(), r.pnt2d(), r.flag());
            gce_root!(p::parab2d, MakeParab2d::from_points(a, c, s));
        }
    }
    end();
}

pub fn status_case(_values: &[f64]) {
    for error in ErrorType::ALL {
        int(error as i32);
    }
    end();
}
