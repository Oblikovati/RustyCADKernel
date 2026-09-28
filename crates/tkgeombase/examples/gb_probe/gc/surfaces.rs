//! The GC surface and transformation cases of tools/oracle/tkgeombase_probe_gc_surfaces.cxx.
use super::super::common::{Reader, end};
use super::{gc_root, gc_trsf, geom_surface, geom_trsf, geom2d_trsf, ok};
use rustycad_tkgeombase::gc::{
    MakeConicalSurface, MakeCylindricalSurface, MakeMirror, MakeMirror2d, MakePlane, MakeRotation,
    MakeRotation2d, MakeScale, MakeScale2d, MakeTranslation, MakeTranslation2d, MakeTrimmedCone,
    MakeTrimmedCylinder,
};
use rustycad_tkmath::gp::{Cone3, Cylinder3, Line2, Line3, Plane3};

pub fn plane_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => gc_root!(
            geom_surface,
            ok(MakePlane::from_pln(Plane3::from_position(r.ax3())))
        ),
        1 => {
            let (q, d) = (r.pnt(), r.dir());
            gc_root!(geom_surface, MakePlane::new(q, d));
        }
        2 => {
            let (a, b, c, d) = (r.real(), r.real(), r.real(), r.real());
            gc_root!(geom_surface, MakePlane::from_equation(a, b, c, d));
        }
        3 => {
            let (a, b, c) = (r.pnt(), r.pnt(), r.pnt());
            gc_root!(geom_surface, MakePlane::through_points(a, b, c));
        }
        4 => {
            let (a, d) = (r.ax3(), r.real());
            gc_root!(
                geom_surface,
                MakePlane::from_plane_distance(Plane3::from_position(a), d)
            );
        }
        5 => {
            let (a, q) = (r.ax3(), r.pnt());
            gc_root!(
                geom_surface,
                MakePlane::from_plane_point(Plane3::from_position(a), q)
            );
        }
        _ => gc_root!(geom_surface, MakePlane::from_axis(r.ax1())),
    }
    end();
}

pub fn conical_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => {
            let (a, g, d) = (r.ax2(), r.real(), r.real());
            gc_root!(geom_surface, MakeConicalSurface::new(a, g, d));
        }
        1 => {
            let (a, g, d) = (r.ax3(), r.real(), r.real());
            let cone = Cone3::new(a, g, d).unwrap();
            gc_root!(geom_surface, ok(MakeConicalSurface::from_cone(cone)));
        }
        2 => {
            let (a, b, c, d) = (r.pnt(), r.pnt(), r.pnt(), r.pnt());
            gc_root!(geom_surface, MakeConicalSurface::through_points(a, b, c, d));
        }
        _ => {
            let (a, b, r1, r2) = (r.pnt(), r.pnt(), r.real(), r.real());
            gc_root!(
                geom_surface,
                MakeConicalSurface::from_points_radii(a, b, r1, r2)
            );
        }
    }
    end();
}

pub fn cylindrical_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => {
            let (a, d) = (r.ax3(), r.real());
            let cylinder = Cylinder3::new(a, d).unwrap();
            gc_root!(
                geom_surface,
                ok(MakeCylindricalSurface::from_cylinder(cylinder))
            );
        }
        1 => {
            let (a, d) = (r.ax2(), r.real());
            gc_root!(geom_surface, MakeCylindricalSurface::new(a, d));
        }
        2 => {
            let (a, d) = (r.ax1(), r.real());
            gc_root!(geom_surface, MakeCylindricalSurface::from_axis(a, d));
        }
        3 => gc_root!(geom_surface, MakeCylindricalSurface::from_circ(r.circ())),
        4 => {
            let (a, b, c) = (r.pnt(), r.pnt(), r.pnt());
            gc_root!(
                geom_surface,
                MakeCylindricalSurface::through_points(a, b, c)
            );
        }
        5 => {
            let (a, r0, d) = (r.ax3(), r.real(), r.real());
            let cylinder = Cylinder3::new(a, r0).unwrap();
            gc_root!(
                geom_surface,
                MakeCylindricalSurface::from_cylinder_distance(cylinder, d)
            );
        }
        _ => {
            let (a, r0, q) = (r.ax3(), r.real(), r.pnt());
            let cylinder = Cylinder3::new(a, r0).unwrap();
            gc_root!(
                geom_surface,
                MakeCylindricalSurface::from_cylinder_point(cylinder, q)
            );
        }
    }
    end();
}

pub fn trimmed_cone_case(values: &[f64]) {
    let mut r = Reader::new(values);
    if r.integer() == 0 {
        let (a, b, c, d) = (r.pnt(), r.pnt(), r.pnt(), r.pnt());
        gc_root!(geom_surface, MakeTrimmedCone::through_points(a, b, c, d));
    } else {
        let (a, b, r1, r2) = (r.pnt(), r.pnt(), r.real(), r.real());
        gc_root!(
            geom_surface,
            MakeTrimmedCone::from_points_radii(a, b, r1, r2)
        );
    }
    end();
}

pub fn trimmed_cylinder_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => {
            let (a, b, c) = (r.pnt(), r.pnt(), r.pnt());
            gc_root!(geom_surface, MakeTrimmedCylinder::through_points(a, b, c));
        }
        1 => {
            let (c, h) = (r.circ(), r.real());
            gc_root!(geom_surface, MakeTrimmedCylinder::from_circ(c, h));
        }
        _ => {
            let (a, d, h) = (r.ax1(), r.real(), r.real());
            gc_root!(geom_surface, MakeTrimmedCylinder::from_axis(a, d, h));
        }
    }
    end();
}

pub fn mirror_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => gc_trsf!(geom_trsf, ok(MakeMirror::from_point(r.pnt()))),
        1 => gc_trsf!(geom_trsf, ok(MakeMirror::from_axis(r.ax1()))),
        2 => gc_trsf!(
            geom_trsf,
            ok(MakeMirror::from_line(Line3::from_axis(r.ax1())))
        ),
        3 => {
            let (q, d) = (r.pnt(), r.dir());
            gc_trsf!(geom_trsf, ok(MakeMirror::from_point_direction(q, d)));
        }
        4 => gc_trsf!(
            geom_trsf,
            MakeMirror::from_plane(Plane3::from_position(r.ax3()))
        ),
        _ => gc_trsf!(geom_trsf, ok(MakeMirror::from_frame(r.ax2()))),
    }
    end();
}

pub fn mirror2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => gc_trsf!(geom2d_trsf, ok(MakeMirror2d::from_point(r.pnt2d()))),
        1 => gc_trsf!(geom2d_trsf, ok(MakeMirror2d::from_axis(r.ax2d()))),
        2 => {
            let a = r.ax2d();
            let line = Line2::new(a.location(), a.direction());
            gc_trsf!(geom2d_trsf, ok(MakeMirror2d::from_line(line)));
        }
        _ => {
            let (q, d) = (r.pnt2d(), r.dir2d());
            gc_trsf!(geom2d_trsf, ok(MakeMirror2d::from_point_direction(q, d)));
        }
    }
    end();
}

pub fn rotation_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => {
            let (a, g) = (r.ax1(), r.real());
            gc_trsf!(geom_trsf, MakeRotation::from_line(Line3::from_axis(a), g));
        }
        1 => {
            let (a, g) = (r.ax1(), r.real());
            gc_trsf!(geom_trsf, MakeRotation::from_axis(a, g));
        }
        _ => {
            let (q, d, g) = (r.pnt(), r.dir(), r.real());
            gc_trsf!(geom_trsf, MakeRotation::from_point_direction(q, d, g));
        }
    }
    end();
}

pub fn rotation2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (q, g) = (r.pnt2d(), r.real());
    gc_trsf!(geom2d_trsf, ok(MakeRotation2d::new(q, g)));
    end();
}

pub fn scale_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (q, s) = (r.pnt(), r.real());
    gc_trsf!(geom_trsf, MakeScale::new(q, s));
    end();
}

pub fn scale2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (q, s) = (r.pnt2d(), r.real());
    gc_trsf!(geom2d_trsf, ok(MakeScale2d::new(q, s)));
    end();
}

pub fn translation_case(values: &[f64]) {
    let mut r = Reader::new(values);
    if r.integer() == 0 {
        gc_trsf!(geom_trsf, ok(MakeTranslation::new(r.vec())));
    } else {
        let (a, b) = (r.pnt(), r.pnt());
        gc_trsf!(geom_trsf, ok(MakeTranslation::from_points(a, b)));
    }
    end();
}

pub fn translation2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    if r.integer() == 0 {
        gc_trsf!(geom2d_trsf, ok(MakeTranslation2d::new(r.vec2d())));
    } else {
        let (a, b) = (r.pnt2d(), r.pnt2d());
        gc_trsf!(geom2d_trsf, ok(MakeTranslation2d::from_points(a, b)));
    }
    end();
}
