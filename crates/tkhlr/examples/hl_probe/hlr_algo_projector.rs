//! HLRAlgo_Projector, the Rust twin of tools/oracle/tkhlr_probe_hlralgo_projector.cxx:
//! the five constructors, Set and Scaled, and every projection.
use super::common::{Reader, attempt, end, flag, pnt, pnt2d, real, xy, xyz};
use rustycad_tkernel::standard::Failure;
use rustycad_tkhlr::hlr_algo::Projector;
use rustycad_tkmath::gp::{Axis2, Axis3, Point2, Transform3, Vector2, Vector3};

/// `P N X k V`: see `hl_read_trsf`.
fn read_trsf(r: &mut Reader) -> Result<Transform3, Failure> {
    let (p, n, x, k, v) = (r.pnt(), r.dir(), r.dir(), r.real(), r.xyz());
    let mut t = Transform3::default();
    t.set_transformation_to(Axis3::from_axis2(Axis2::with_x_direction(p, n, x)?));
    t.set_scale_factor(k)?;
    t.set_translation_part(Vector3::new(v.x(), v.y(), v.z()));
    Ok(t)
}

fn print_trsf(t: &Transform3) -> Result<(), Failure> {
    for i in 1..=3 {
        for j in 1..=4 {
            real(t.value(i, j)?);
        }
    }
    Ok(())
}

fn print_projector(pr: &Projector) -> Result<(), Failure> {
    flag(pr.perspective());
    print_trsf(&pr.transformation())?;
    print_trsf(&pr.inverted_transformation())?;
    print_trsf(&pr.full_transformation())?;
    let (d1, d2, d3) = pr.directions();
    [d1, d2, d3].into_iter().for_each(|d| xy(d.xy()));
    attempt(|| {
        real(pr.focus()?);
        Ok(())
    });
    Ok(())
}

fn build_projector(r: &mut Reader, kind: i32) -> Result<Projector, Failure> {
    match kind {
        1 => {
            let (p, n, x) = (r.pnt(), r.dir(), r.dir());
            Projector::with_frame(Axis2::with_x_direction(p, n, x)?)
        }
        2 => {
            let (p, n, x) = (r.pnt(), r.dir(), r.dir());
            let frame = Axis2::with_x_direction(p, n, x)?;
            Projector::with_frame_focus(frame, r.real())
        }
        3 => {
            let t = read_trsf(r)?;
            let persp = r.flag();
            Projector::with_transformation(t, persp, r.real())
        }
        4 => {
            let t = read_trsf(r)?;
            let (persp, focus) = (r.flag(), r.real());
            let (a, b, c) = (r.pnt2d(), r.pnt2d(), r.pnt2d());
            let dirs = [a, b, c].map(|p| Vector2::new(p.x(), p.y()));
            Projector::with_directions(t, persp, focus, dirs)
        }
        _ => Projector::new(),
    }
}

fn project_point(r: &mut Reader, pr: &Projector) {
    let (p, d, x, y) = (r.pnt(), r.xyz(), r.real(), r.real());
    let mut out = Point2::new(7.0, 8.0);
    pr.project(p, &mut out);
    pnt2d(out);
    let (px, py, pz) = pr.project_xyz(p);
    [px, py, pz].into_iter().for_each(real);
    let d = Vector3::new(d.x(), d.y(), d.z());
    let (o2, v2) = pr.project_d1(p, d);
    pnt2d(o2);
    xy(v2.xy());
    attempt(|| {
        let line = pr.shoot(x, y)?;
        pnt(line.location());
        super::common::dir(line.direction());
        Ok(())
    });
    let mut tp = p;
    pr.transform_point(&mut tp);
    pnt(tp);
    let mut tv = d;
    pr.transform_vector(&mut tv);
    xyz(*tv.xyz());
}

fn projector_body(r: &mut Reader) -> Result<(), Failure> {
    let kind = r.integer();
    let mut pr = build_projector(r, kind)?;
    if r.flag() {
        let t = read_trsf(r)?;
        let persp = r.flag();
        pr.set(t, persp, r.real())?;
    }
    let scaled = r.integer();
    if scaled >= 0 {
        pr.scaled(scaled != 0)?;
    }
    print_projector(&pr)?;
    for _ in 0..r.integer() {
        project_point(r, &pr);
    }
    Ok(())
}

/// `kind ... n (P D x y)*n`: see `hl_ha_projector_case`.
pub fn projector_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| projector_body(&mut r));
    end();
}
