//! The GC curve cases of tools/oracle/tkgeombase_probe_gc_curves.cxx.
use super::super::common::{Reader, end};
use super::{gc_root, geom_curve, geom2d_curve, ok};
use rustycad_tkgeombase::gc::{
    MakeArcOfCircle, MakeArcOfCircle2d, MakeArcOfEllipse, MakeArcOfEllipse2d, MakeArcOfHyperbola,
    MakeArcOfHyperbola2d, MakeArcOfParabola, MakeArcOfParabola2d, MakeCircle, MakeCircle2d,
    MakeEllipse, MakeEllipse2d, MakeHyperbola, MakeHyperbola2d, MakeLine, MakeLine2d,
    MakeParabola2d, MakeSegment, MakeSegment2d,
};
use rustycad_tkmath::gp::{Line2, Line3};

/// gb_gc_arc and gb_gc_arc2d: the three overloads that every arc class shares.
macro_rules! arc_case {
    ($maker:ident, $print:path, $k:expr, $r:expr, $conic:expr, $point:ident) => {{
        let conic = $conic;
        match $k {
            0 => {
                let (a, b, s) = ($r.real(), $r.real(), $r.flag());
                gc_root!($print, $maker::from_parameters(conic, a, b, s));
            }
            1 => {
                let (q, a, s) = ($r.$point(), $r.real(), $r.flag());
                gc_root!($print, $maker::from_point_parameter(conic, q, a, s));
            }
            _ => {
                let (q1, q2, s) = ($r.$point(), $r.$point(), $r.flag());
                gc_root!($print, $maker::from_points(conic, q1, q2, s));
            }
        }
    }};
}

pub fn arc_circle_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        k @ 0..=2 => arc_case!(MakeArcOfCircle, geom_curve, k, r, r.circ(), pnt),
        3 => {
            let (a, b, c) = (r.pnt(), r.pnt(), r.pnt());
            gc_root!(geom_curve, MakeArcOfCircle::through_points(a, b, c));
        }
        _ => {
            let (a, v, b) = (r.pnt(), r.vec(), r.pnt());
            gc_root!(geom_curve, MakeArcOfCircle::from_tangent(a, v, b));
        }
    }
    end();
}

pub fn arc_ellipse_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let k = r.integer();
    arc_case!(MakeArcOfEllipse, geom_curve, k, r, r.elips(), pnt);
    end();
}

pub fn arc_hyperbola_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let k = r.integer();
    arc_case!(MakeArcOfHyperbola, geom_curve, k, r, r.hypr(), pnt);
    end();
}

pub fn arc_parabola_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let k = r.integer();
    arc_case!(MakeArcOfParabola, geom_curve, k, r, r.parab(), pnt);
    end();
}

pub fn arc_circle2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        k @ 0..=2 => arc_case!(MakeArcOfCircle2d, geom2d_curve, k, r, r.circ2d(), pnt2d),
        3 => {
            let (a, b, c) = (r.pnt2d(), r.pnt2d(), r.pnt2d());
            gc_root!(geom2d_curve, MakeArcOfCircle2d::through_points(a, b, c));
        }
        _ => {
            let (a, v, b) = (r.pnt2d(), r.vec2d(), r.pnt2d());
            gc_root!(geom2d_curve, MakeArcOfCircle2d::from_tangent(a, v, b));
        }
    }
    end();
}

pub fn arc_ellipse2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let k = r.integer();
    arc_case!(MakeArcOfEllipse2d, geom2d_curve, k, r, r.elips2d(), pnt2d);
    end();
}

pub fn arc_hyperbola2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let k = r.integer();
    arc_case!(MakeArcOfHyperbola2d, geom2d_curve, k, r, r.hypr2d(), pnt2d);
    end();
}

pub fn arc_parabola2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let k = r.integer();
    arc_case!(MakeArcOfParabola2d, geom2d_curve, k, r, r.parab2d(), pnt2d);
    end();
}

pub fn circle_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => gc_root!(geom_curve, ok(MakeCircle::from_circ(r.circ()))),
        1 => {
            let (a, d) = (r.ax2(), r.real());
            gc_root!(geom_curve, MakeCircle::new(a, d));
        }
        2 => {
            let (c, d) = (r.circ(), r.real());
            gc_root!(geom_curve, MakeCircle::from_circ_distance(c, d));
        }
        3 => {
            let (c, q) = (r.circ(), r.pnt());
            gc_root!(geom_curve, MakeCircle::from_circ_point(c, q));
        }
        4 => {
            let (a, b, c) = (r.pnt(), r.pnt(), r.pnt());
            gc_root!(geom_curve, MakeCircle::through_points(a, b, c));
        }
        5 => {
            let (c, n, d) = (r.pnt(), r.dir(), r.real());
            gc_root!(geom_curve, MakeCircle::from_center_normal(c, n, d));
        }
        6 => {
            let (c, q, d) = (r.pnt(), r.pnt(), r.real());
            gc_root!(geom_curve, MakeCircle::from_center_axis_point(c, q, d));
        }
        _ => {
            let (a, d) = (r.ax1(), r.real());
            gc_root!(geom_curve, MakeCircle::from_axis(a, d));
        }
    }
    end();
}

pub fn circle2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => gc_root!(geom2d_curve, ok(MakeCircle2d::from_circ(r.circ2d()))),
        1 => {
            let (a, d, s) = (r.ax2d(), r.real(), r.flag());
            gc_root!(geom2d_curve, MakeCircle2d::new(a, d, s));
        }
        2 => {
            let (a, d) = (r.ax22d(), r.real());
            gc_root!(geom2d_curve, MakeCircle2d::from_frame(a, d));
        }
        3 => {
            let (c, d) = (r.circ2d(), r.real());
            gc_root!(geom2d_curve, MakeCircle2d::from_circ_distance(c, d));
        }
        4 => {
            let (c, q) = (r.circ2d(), r.pnt2d());
            gc_root!(geom2d_curve, MakeCircle2d::from_circ_point(c, q));
        }
        5 => {
            let (a, b, c) = (r.pnt2d(), r.pnt2d(), r.pnt2d());
            gc_root!(geom2d_curve, MakeCircle2d::through_points(a, b, c));
        }
        6 => {
            let (c, d, s) = (r.pnt2d(), r.real(), r.flag());
            gc_root!(geom2d_curve, MakeCircle2d::from_center(c, d, s));
        }
        _ => {
            let (c, q, s) = (r.pnt2d(), r.pnt2d(), r.flag());
            gc_root!(geom2d_curve, MakeCircle2d::from_center_point(c, q, s));
        }
    }
    end();
}

pub fn ellipse_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => gc_root!(geom_curve, ok(MakeEllipse::from_elips(r.elips()))),
        1 => {
            let (a, m, n) = (r.ax2(), r.real(), r.real());
            gc_root!(geom_curve, MakeEllipse::new(a, m, n));
        }
        _ => {
            let (a, b, c) = (r.pnt(), r.pnt(), r.pnt());
            gc_root!(geom_curve, MakeEllipse::from_points(a, b, c));
        }
    }
    end();
}

pub fn hyperbola_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => gc_root!(geom_curve, ok(MakeHyperbola::from_hypr(r.hypr()))),
        1 => {
            let (a, m, n) = (r.ax2(), r.real(), r.real());
            gc_root!(geom_curve, MakeHyperbola::new(a, m, n));
        }
        _ => {
            let (a, b, c) = (r.pnt(), r.pnt(), r.pnt());
            gc_root!(geom_curve, MakeHyperbola::from_points(a, b, c));
        }
    }
    end();
}

pub fn ellipse2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => gc_root!(geom2d_curve, ok(MakeEllipse2d::from_elips(r.elips2d()))),
        1 => {
            let (a, m, n) = (r.ax22d(), r.real(), r.real());
            gc_root!(geom2d_curve, MakeEllipse2d::from_frame(a, m, n));
        }
        2 => {
            let (a, m, n, s) = (r.ax2d(), r.real(), r.real(), r.flag());
            gc_root!(geom2d_curve, MakeEllipse2d::new(a, m, n, s));
        }
        _ => {
            let (a, b, c) = (r.pnt2d(), r.pnt2d(), r.pnt2d());
            gc_root!(geom2d_curve, MakeEllipse2d::from_points(a, b, c));
        }
    }
    end();
}

pub fn hyperbola2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => gc_root!(geom2d_curve, ok(MakeHyperbola2d::from_hypr(r.hypr2d()))),
        1 => {
            let (a, m, n, s) = (r.ax2d(), r.real(), r.real(), r.flag());
            gc_root!(geom2d_curve, MakeHyperbola2d::new(a, m, n, s));
        }
        2 => {
            let (a, m, n) = (r.ax22d(), r.real(), r.real());
            gc_root!(geom2d_curve, MakeHyperbola2d::from_frame(a, m, n));
        }
        _ => {
            let (a, b, c) = (r.pnt2d(), r.pnt2d(), r.pnt2d());
            gc_root!(geom2d_curve, MakeHyperbola2d::from_points(a, b, c));
        }
    }
    end();
}

pub fn parabola2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => gc_root!(geom2d_curve, ok(MakeParabola2d::from_parab(r.parab2d()))),
        1 => {
            let (a, f, s) = (r.ax2d(), r.real(), r.flag());
            gc_root!(geom2d_curve, MakeParabola2d::new(a, f, s));
        }
        2 => {
            let (a, f) = (r.ax22d(), r.real());
            gc_root!(geom2d_curve, MakeParabola2d::from_frame(a, f));
        }
        3 => {
            let (a, f, s) = (r.ax2d(), r.pnt2d(), r.flag());
            gc_root!(geom2d_curve, MakeParabola2d::from_directrix(a, f, s));
        }
        _ => {
            let (a, o) = (r.pnt2d(), r.pnt2d());
            gc_root!(geom2d_curve, MakeParabola2d::from_points(a, o));
        }
    }
    end();
}

pub fn line_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => {
            let (q, d) = (r.pnt(), r.dir());
            gc_root!(geom_curve, ok(MakeLine::new(q, d)));
        }
        1 => gc_root!(geom_curve, ok(MakeLine::from_axis(r.ax1()))),
        2 => gc_root!(
            geom_curve,
            ok(MakeLine::from_lin(Line3::from_axis(r.ax1())))
        ),
        3 => {
            let (a, b) = (r.pnt(), r.pnt());
            gc_root!(geom_curve, MakeLine::from_points(a, b));
        }
        _ => {
            let (a, q) = (r.ax1(), r.pnt());
            gc_root!(
                geom_curve,
                MakeLine::from_line_point(Line3::from_axis(a), q)
            );
        }
    }
    end();
}

pub fn line2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let line = |a: rustycad_tkmath::gp::Axis2d| Line2::new(a.location(), a.direction());
    match r.integer() {
        0 => gc_root!(geom2d_curve, ok(MakeLine2d::from_axis(r.ax2d()))),
        1 => gc_root!(geom2d_curve, ok(MakeLine2d::from_lin(line(r.ax2d())))),
        2 => {
            let (q, d) = (r.pnt2d(), r.dir2d());
            gc_root!(geom2d_curve, ok(MakeLine2d::new(q, d)));
        }
        3 => {
            let (a, b) = (r.pnt2d(), r.pnt2d());
            gc_root!(geom2d_curve, MakeLine2d::from_points(a, b));
        }
        4 => {
            let (a, q) = (r.ax2d(), r.pnt2d());
            gc_root!(geom2d_curve, MakeLine2d::from_line_point(line(a), q));
        }
        _ => {
            let (a, d) = (r.ax2d(), r.real());
            gc_root!(geom2d_curve, MakeLine2d::from_line_distance(line(a), d));
        }
    }
    end();
}

pub fn segment_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => {
            let (a, b) = (r.pnt(), r.pnt());
            gc_root!(geom_curve, MakeSegment::from_points(a, b));
        }
        1 => {
            let (a, q, u) = (r.ax1(), r.pnt(), r.real());
            gc_root!(
                geom_curve,
                MakeSegment::from_point_parameter(Line3::from_axis(a), q, u)
            );
        }
        2 => {
            let (a, q1, q2) = (r.ax1(), r.pnt(), r.pnt());
            gc_root!(
                geom_curve,
                MakeSegment::from_line_points(Line3::from_axis(a), q1, q2)
            );
        }
        _ => {
            let (a, u, v) = (r.ax1(), r.real(), r.real());
            gc_root!(
                geom_curve,
                MakeSegment::from_parameters(Line3::from_axis(a), u, v)
            );
        }
    }
    end();
}

pub fn segment2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let line = |a: rustycad_tkmath::gp::Axis2d| Line2::new(a.location(), a.direction());
    match r.integer() {
        0 => {
            let (a, d, b) = (r.pnt2d(), r.dir2d(), r.pnt2d());
            gc_root!(geom2d_curve, MakeSegment2d::from_direction(a, d, b));
        }
        1 => {
            let (a, b) = (r.pnt2d(), r.pnt2d());
            gc_root!(geom2d_curve, MakeSegment2d::from_points(a, b));
        }
        2 => {
            let (a, q, u) = (r.ax2d(), r.pnt2d(), r.real());
            gc_root!(
                geom2d_curve,
                MakeSegment2d::from_point_parameter(line(a), q, u)
            );
        }
        3 => {
            let (a, q1, q2) = (r.ax2d(), r.pnt2d(), r.pnt2d());
            gc_root!(
                geom2d_curve,
                MakeSegment2d::from_line_points(line(a), q1, q2)
            );
        }
        _ => {
            let (a, u, v) = (r.ax2d(), r.real(), r.real());
            gc_root!(geom2d_curve, MakeSegment2d::from_parameters(line(a), u, v));
        }
    }
    end();
}
