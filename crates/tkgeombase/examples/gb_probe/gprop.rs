//! The GProp probe; the counterpart of tools/oracle/tkgeombase_probe_gprop.cxx.
use super::common::{self as p, Reader, attempt, end};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::gprop::{
    self, CelGProps, GProps, PEquation, PGProps, SelGProps, VelGProps,
};
use rustycad_tkmath::gp::{
    Axis1, Axis3, Circle3, Cone3, Cylinder3, Direction3, Line3, Matrix3, Point3, Sphere3, Torus3,
};
use std::f64::consts::PI;

fn mat(m: Matrix3) {
    for i in 1..=3 {
        for j in 1..=3 {
            p::real(m.value(i, j).unwrap());
        }
    }
}

/// gb_gp_props.
fn props(g: &GProps) {
    p::real(g.mass());
    p::pnt(g.centre_of_mass());
    mat(g.matrix_of_inertia());
    let (x, y, z) = g.static_moments();
    [x, y, z].into_iter().for_each(p::real);
    let axes = [
        Axis1::new(
            Point3::new(0.0, 0.0, 0.0),
            Direction3::new(0.0, 0.0, 1.0).unwrap(),
        ),
        Axis1::new(
            Point3::new(1.0, 2.0, 3.0),
            Direction3::new(1.0, 1.0, 1.0).unwrap(),
        ),
    ];
    for axis in axes {
        p::real(g.moment_of_inertia(axis));
        p::real(g.radius_of_gyration(axis));
    }
    p::show(g.principal_properties(), |pp| {
        let (x, y, z) = pp.moments();
        [x, y, z].into_iter().for_each(p::real);
        let (x, y, z) = pp.radius_of_gyration();
        [x, y, z].into_iter().for_each(p::real);
        p::vec(pp.first_axis_of_inertia());
        p::vec(pp.second_axis_of_inertia());
        p::vec(pp.third_axis_of_inertia());
        p::flag(pp.has_symmetry_axis());
        p::flag(pp.has_symmetry_axis_with_tolerance(1e-6));
        p::flag(pp.has_symmetry_point());
        p::flag(pp.has_symmetry_point_with_tolerance(1e-6));
    });
}

fn points_and_densities(
    r: &mut Reader<'_>,
    n: i32,
) -> (Array1<'static, Point3>, Array1<'static, f64>) {
    let mut pts = Array1::with_bounds(1, n).unwrap();
    let mut den = Array1::with_bounds(1, n).unwrap();
    for i in 1..=n {
        pts[i] = r.pnt();
        den[i] = r.real();
    }
    (pts, den)
}

pub fn points_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (mode, n, rows) = (r.integer(), r.integer(), r.integer());
    let (pts, den) = points_and_densities(&mut r, n);
    attempt(|| points_mode(mode, rows, &pts, &den));
    end();
}

fn points_mode(
    mode: i32,
    rows: i32,
    pts: &Array1<'_, Point3>,
    den: &Array1<'_, f64>,
) -> Result<(), Failure> {
    if mode <= 1 {
        let mut g = PGProps::new();
        for i in 1..=pts.len() as i32 {
            if mode == 0 {
                g.add_point(pts[i]);
            } else {
                g.add_weighted_point(pts[i], den[i])?;
            }
        }
        props(&g);
    }
    if mode == 2 {
        props(&PGProps::from_points(pts));
        p::show(PGProps::from_weighted_points(pts, den), |g| props(&g));
        p::show(PGProps::barycentre(pts), p::pnt);
        p::show(PGProps::weighted_barycentre(pts, den), |(m, b)| {
            p::real(m);
            p::pnt(b);
        });
    }
    if mode == 3 {
        let cols: i32 = pts.len() as i32 / rows;
        let mut p2: Array2<'static, Point3> = Array2::with_bounds(1, rows, 1, cols).unwrap();
        let mut d2: Array2<'static, f64> = Array2::with_bounds(1, rows, 1, cols).unwrap();
        for i in 1..=rows {
            for j in 1..=cols {
                p2[(i, j)] = pts[(i - 1) * cols + j];
                d2[(i, j)] = den[(i - 1) * cols + j];
            }
        }
        props(&PGProps::from_points_2d(&p2));
        p::show(PGProps::from_weighted_points_2d(&p2, &d2), |g| props(&g));
        p::pnt(PGProps::barycentre_2d(&p2));
        p::show(PGProps::weighted_barycentre_2d(&p2, &d2), |(m, b)| {
            p::real(m);
            p::pnt(b);
        });
    }
    Ok(())
}

pub fn pequation_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (n, tol) = (r.integer(), r.real());
    let mut pts = Array1::with_bounds(1, n).unwrap();
    for i in 1..=n {
        pts[i] = r.pnt();
    }
    let e = PEquation::new(&pts, tol);
    p::int(e.get_type() as i32);
    [e.is_point(), e.is_linear(), e.is_planar(), e.is_space()]
        .into_iter()
        .for_each(p::flag);
    p::pnt(e.barycentre());
    for i in 1..=3 {
        p::vec(e.principal_axis(i).unwrap());
        p::real(e.extent(i).unwrap());
    }
    p::show(e.point(), p::pnt);
    p::show(e.line(), p::lin);
    p::show(e.plane(), p::pln);
    p::show(e.box_(), |(b, v1, v2, v3)| {
        p::pnt(b);
        [v1, v2, v3].into_iter().for_each(p::vec);
    });
    end();
}

// The shapes of gb_gp_patches: 0 cylinder, 1 cone, 2 sphere, 3 torus.
enum Patch {
    Cylinder(Cylinder3),
    Cone(Cone3),
    Sphere(Sphere3),
    Torus(Torus3),
}

fn patch(r: &mut Reader<'_>, shape: i32) -> Result<(Patch, [f64; 4]), Failure> {
    let ax: Axis3 = r.ax3();
    let r1 = r.real();
    let r2 = if shape == 1 || shape == 3 {
        r.real()
    } else {
        0.0
    };
    let a = [r.real(), r.real(), r.real(), r.real()];
    let built = match shape {
        0 => Patch::Cylinder(Cylinder3::new(ax, r1)?),
        1 => Patch::Cone(Cone3::new(ax, r1, r2)?),
        2 => Patch::Sphere(Sphere3::new(ax, r1)?),
        _ => Patch::Torus(Torus3::new(ax, r1, r2)?),
    };
    Ok((built, a))
}

macro_rules! patch_props {
    ($props:ty, $shape:expr, $a:expr, $loc:expr, $perform:expr) => {{
        let [a1, a2, b1, b2] = $a;
        let g = if $perform {
            let mut g = <$props>::new();
            g.set_location($loc);
            match $shape {
                Patch::Cylinder(s) => g.perform_cylinder(s, a1, a2, b1, b2)?,
                Patch::Cone(s) => g.perform_cone(s, a1, a2, b1, b2)?,
                Patch::Sphere(s) => g.perform_sphere(s, a1, a2, b1, b2)?,
                Patch::Torus(s) => g.perform_torus(s, a1, a2, b1, b2)?,
            }
            g
        } else {
            match $shape {
                Patch::Cylinder(s) => <$props>::from_cylinder(s, a1, a2, b1, b2, $loc)?,
                Patch::Cone(s) => <$props>::from_cone(s, a1, a2, b1, b2, $loc)?,
                Patch::Sphere(s) => <$props>::from_sphere(s, a1, a2, b1, b2, $loc)?,
                Patch::Torus(s) => <$props>::from_torus(s, a1, a2, b1, b2, $loc)?,
            }
        };
        props(&g);
    }};
}

pub fn elementary_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, perform, loc) = (r.integer(), r.flag(), r.pnt());
    attempt(|| {
        if kind <= 1 {
            let (ax, rad) = (r.ax2(), r.real());
            let c = Circle3::new(ax, rad)?;
            let (u1, u2) = if kind == 1 {
                (r.real(), r.real())
            } else {
                (0.0, 2.0 * PI)
            };
            let g = if perform {
                let mut g = CelGProps::new();
                g.set_location(loc);
                g.perform_circle(c, u1, u2)?;
                g
            } else if kind == 0 {
                CelGProps::from_circle(c, loc)?
            } else {
                CelGProps::from_circle_arc(c, u1, u2, loc)?
            };
            props(&g);
        }
        if kind == 2 {
            let pnt = r.pnt();
            let l = Line3::new(pnt, r.dir());
            let (u1, u2) = (r.real(), r.real());
            let g = if perform {
                let mut g = CelGProps::new();
                g.set_location(loc);
                g.perform_line(l, u1, u2);
                g
            } else {
                CelGProps::from_line(l, u1, u2, loc)
            };
            props(&g);
        }
        if (3..=6).contains(&kind) {
            let (shape, a) = patch(&mut r, kind - 3)?;
            patch_props!(SelGProps, shape, a, loc, perform);
        }
        if kind >= 7 {
            let (shape, a) = patch(&mut r, kind - 7)?;
            patch_props!(VelGProps, shape, a, loc, perform);
        }
        Ok(())
    });
    end();
}

pub fn compose_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut g = GProps::with_location(r.pnt());
    for _ in 0..r.integer() {
        let mut points = PGProps::new();
        for _ in 0..r.integer() {
            points.add_point(r.pnt());
        }
        let density = r.real();
        attempt(|| g.add(&points, density));
    }
    props(&g);
    end();
}

pub fn hoperator_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (g, q) = (r.pnt(), r.pnt());
    mat(gprop::h_operator(g, q, r.real()));
    end();
}
