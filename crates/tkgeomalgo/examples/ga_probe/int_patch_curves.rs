//! IntPatch line probe operations, the Rust twin of the conic and analytic lines of
//! tools/oracle/tkgeomalgo_probe_intpatch_lines.cxx.
use super::common::{Reader, attempt, dir, end, flag, int, pnt, real, xyz};
use super::int_patch::{Fixture, read_point};
use super::int_patch_lines::{line_base, read_trans, vertices};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeomalgo::int_patch::{IntPatchALine, IntPatchGLine, IntPatchLineTransitions};
use rustycad_tkgeombase::int_ana::{Curve as IntAnaCurve, IntQuadQuad, Quadric};
use rustycad_tkmath::gp::{
    Axis2, Axis3, Circle3, Cylinder3, Direction3, Ellipse3, Hyperbola3, Line3, Parabola3, Point3,
    Sphere3, Vector3,
};

fn make_gline(
    r: &mut Reader,
    kind: i32,
    (tang, t): (bool, IntPatchLineTransitions),
) -> Result<IntPatchGLine, Failure> {
    let (p, d, xd) = (r.pnt(), r.dir(), r.dir());
    let (r1, r2) = (r.real(), r.real());
    let ax = Axis2::with_x_direction(p, d, xd)?;
    Ok(match kind {
        0 => IntPatchGLine::from_line(&Line3::new(p, d), tang, t)?,
        1 => IntPatchGLine::from_circle(&Circle3::new(ax, r1)?, tang, t),
        2 => IntPatchGLine::from_ellipse(&Ellipse3::new(ax, r1, r2)?, tang, t),
        3 => IntPatchGLine::from_parabola(&Parabola3::new(ax, r1)?, tang, t),
        _ => IntPatchGLine::from_hyperbola(&Hyperbola3::new(ax, r1, r2)?, tang, t),
    })
}

fn gline_command(r: &mut Reader, fx: &Fixture, l: &IntPatchGLine, c: i32) -> Result<(), Failure> {
    match c {
        1 => l.add_vertex(&read_point(r, fx)),
        3 => {
            let i = r.integer();
            l.replace(i, &read_point(r, fx));
            Ok(())
        }
        4 => {
            l.set_first_point(r.integer());
            Ok(())
        }
        5 => {
            l.set_last_point(r.integer());
            Ok(())
        }
        _ => l.compute_vertex_parameters(r.real()),
    }
}

fn gline_geometry(l: &IntPatchGLine) {
    attempt(|| {
        let x = l.line()?;
        pnt(x.location());
        dir(x.direction());
        Ok(())
    });
    attempt(|| {
        let x = l.circle()?;
        pnt(x.location());
        dir(x.axis().direction());
        dir(x.x_axis().direction());
        real(x.radius());
        Ok(())
    });
    attempt(|| {
        let x = l.ellipse()?;
        pnt(x.location());
        dir(x.x_axis().direction());
        real(x.major_radius());
        real(x.minor_radius());
        Ok(())
    });
    attempt(|| {
        let x = l.parabola()?;
        pnt(x.location());
        dir(x.y_axis().direction());
        real(x.focal());
        Ok(())
    });
    attempt(|| {
        let x = l.hyperbola()?;
        pnt(x.location());
        dir(x.axis().direction());
        real(x.major_radius());
        real(x.minor_radius());
        Ok(())
    });
}

/// `ga_ip_gline_case`.
pub fn gline_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let fx = Fixture::new();
    let kind = r.integer();
    let trans = read_trans(&mut r);
    attempt(|| {
        let l = Handle::new(make_gline(&mut r, kind, trans)?);
        for _ in 0..r.integer() {
            l.add_vertex(&read_point(&mut r, &fx))?;
        }
        for _ in 0..r.integer() {
            let cmd = r.integer();
            attempt(|| gline_command(&mut r, &fx, &l, cmd));
        }
        line_base(&l);
        vertices(&fx, &*l);
        gline_geometry(&l);
        Ok(())
    });
    end();
}

fn aline_command(r: &mut Reader, fx: &Fixture, l: &IntPatchALine, c: i32) -> Result<(), Failure> {
    match c {
        1 => {
            l.add_vertex(&read_point(r, fx));
            Ok(())
        }
        3 => {
            let i = r.integer();
            l.replace(i, &read_point(r, fx))
        }
        4 => {
            l.set_first_point(r.integer());
            Ok(())
        }
        5 => {
            l.set_last_point(r.integer());
            Ok(())
        }
        6 => l.compute_vertex_parameters(r.real()),
        _ => {
            let i = r.integer();
            let prm = r.real();
            l.change_vertex(i, &mut |p| p.set_parameter(prm))
        }
    }
}

fn aline_curve(r: &mut Reader) -> Result<IntAnaCurve, Failure> {
    let (p, d, rc) = (r.pnt(), r.dir(), r.real());
    let (sp, sr, tol) = (r.pnt(), r.real(), r.real());
    let cyl = Cylinder3::new(Axis3::from_normal(p, d)?, rc)?;
    let z = Direction3::new(0.0, 0.0, 1.0)?;
    let sphere = Sphere3::new(Axis3::from_normal(sp, z)?, sr)?;
    let q = IntQuadQuad::cylinder(cyl, &Quadric::from_sphere(sphere), tol)?;
    if q.nb_curve()? > 0 {
        return Ok(*q.curve(1)?);
    }
    Ok(IntAnaCurve::new())
}

fn aline_samples(r: &mut Reader, l: &IntPatchALine) {
    for _ in 0..r.integer() {
        let u = r.real();
        attempt(|| l.value(u).map(pnt));
        attempt(|| {
            let (mut x, mut v) = (Point3::default(), Vector3::new(0.0, 0.0, 0.0));
            flag(l.d1(u, &mut x, &mut v)?);
            pnt(x);
            xyz(*v.xyz());
            Ok(())
        });
        attempt(|| {
            let mut ps: List<f64> = List::default();
            l.find_parameter(l.value(u)?, &mut ps)?;
            int(ps.extent());
            ps.iter().copied().for_each(real);
            Ok(())
        });
    }
}

/// `ga_ip_aline_case`.
pub fn aline_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let fx = Fixture::new();
    attempt(|| {
        let curve = aline_curve(&mut r)?;
        let (tang, t) = read_trans(&mut r);
        let l = Handle::new(IntPatchALine::new(&curve, tang, t));
        for _ in 0..r.integer() {
            l.add_vertex(&read_point(&mut r, &fx));
        }
        for _ in 0..r.integer() {
            let cmd = r.integer();
            attempt(|| aline_command(&mut r, &fx, &l, cmd));
        }
        line_base(&l);
        vertices(&fx, &*l);
        attempt(|| {
            l.first_parameter()
                .map(|(p, inc)| (real(p), flag(inc)))
                .map(|_| ())
        });
        attempt(|| {
            l.last_parameter()
                .map(|(p, inc)| (real(p), flag(inc)))
                .map(|_| ())
        });
        aline_samples(&mut r, &l);
        Ok(())
    });
    end();
}
