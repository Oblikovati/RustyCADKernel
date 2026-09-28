//! The gce transformation cases of tools/oracle/tkgeombase_probe_gce.cxx.
use super::super::common::{self as p, Reader, end};
use super::{gce_trsf, ok};
use rustycad_tkgeombase::gce::{
    MakeMirror, MakeMirror2d, MakeRotation, MakeRotation2d, MakeScale, MakeScale2d,
    MakeTranslation, MakeTranslation2d,
};
use rustycad_tkmath::gp::{Line2, Line3, Plane3};

pub fn mirror_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => {
            let q = r.pnt();
            gce_trsf!(p::trsf, ok(MakeMirror::from_point(q)));
        }
        1 => {
            let a = r.ax1();
            gce_trsf!(p::trsf, ok(MakeMirror::from_axis(a)));
        }
        2 => {
            let a = r.ax1();
            gce_trsf!(p::trsf, ok(MakeMirror::from_line(Line3::from_axis(a))));
        }
        3 => {
            let (q, d) = (r.pnt(), r.dir());
            gce_trsf!(p::trsf, ok(MakeMirror::from_point_direction(q, d)));
        }
        4 => {
            let a = r.ax3();
            gce_trsf!(p::trsf, MakeMirror::from_plane(Plane3::from_position(a)));
        }
        _ => {
            let a = r.ax2();
            gce_trsf!(p::trsf, ok(MakeMirror::from_frame(a)));
        }
    }
    end();
}

pub fn mirror2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => {
            let q = r.pnt2d();
            gce_trsf!(p::trsf2d, ok(MakeMirror2d::from_point(q)));
        }
        1 => {
            let a = r.ax2d();
            gce_trsf!(p::trsf2d, ok(MakeMirror2d::from_axis(a)));
        }
        2 => {
            let a = r.ax2d();
            let line = Line2::new(a.location(), a.direction());
            gce_trsf!(p::trsf2d, ok(MakeMirror2d::from_line(line)));
        }
        _ => {
            let (q, d) = (r.pnt2d(), r.dir2d());
            gce_trsf!(p::trsf2d, ok(MakeMirror2d::from_point_direction(q, d)));
        }
    }
    end();
}

pub fn rotation_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => {
            let (a, g) = (r.ax1(), r.real());
            gce_trsf!(p::trsf, MakeRotation::from_line(Line3::from_axis(a), g));
        }
        1 => {
            let (a, g) = (r.ax1(), r.real());
            gce_trsf!(p::trsf, MakeRotation::from_axis(a, g));
        }
        _ => {
            let (q, d, g) = (r.pnt(), r.dir(), r.real());
            gce_trsf!(p::trsf, MakeRotation::from_point_direction(q, d, g));
        }
    }
    end();
}

pub fn rotation2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (q, g) = (r.pnt2d(), r.real());
    gce_trsf!(p::trsf2d, ok(MakeRotation2d::new(q, g)));
    end();
}

pub fn scale_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (q, s) = (r.pnt(), r.real());
    gce_trsf!(p::trsf, MakeScale::new(q, s));
    end();
}

pub fn scale2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (q, s) = (r.pnt2d(), r.real());
    gce_trsf!(p::trsf2d, ok(MakeScale2d::new(q, s)));
    end();
}

pub fn translation_case(values: &[f64]) {
    let mut r = Reader::new(values);
    if r.integer() == 0 {
        let v = r.vec();
        gce_trsf!(p::trsf, ok(MakeTranslation::new(v)));
    } else {
        let (a, b) = (r.pnt(), r.pnt());
        gce_trsf!(p::trsf, ok(MakeTranslation::from_points(a, b)));
    }
    end();
}

pub fn translation2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    if r.integer() == 0 {
        let v = r.vec2d();
        gce_trsf!(p::trsf2d, ok(MakeTranslation2d::new(v)));
    } else {
        let (a, b) = (r.pnt2d(), r.pnt2d());
        gce_trsf!(p::trsf2d, ok(MakeTranslation2d::from_points(a, b)));
    }
    end();
}
