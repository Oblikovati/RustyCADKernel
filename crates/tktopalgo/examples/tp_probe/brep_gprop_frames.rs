//! BRepGProp probe operations on the Sinert, Vinert and VinertGK frameworks (constructors
//! and Perform sequences), the rest of the Rust twin of
//! tools/oracle/tktopalgo_probe_brepgprop.cxx.
use super::brep_gprop::{gprops, plane};
use super::common::{Reader, attempt, end, real};
use super::shape::tp_shape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Plane3, Point3};
use rustycad_tktopalgo::brep_gprop::{
    BRepGPropDomain, BRepGPropFace, BRepGPropSinert, BRepGPropVinert, BRepGPropVinertGK,
    VinertGKOptions,
};

/// `TpBound`: the location (0), a point (1) or a plane (2).
#[derive(Clone, Copy)]
enum Bound {
    Location,
    Point(Point3),
    Plane(Plane3),
}

fn bound(r: &mut Reader<'_>) -> Bound {
    match r.integer() {
        1 => Bound::Point(r.pnt()),
        2 => Bound::Plane(plane(r)),
        _ => Bound::Location,
    }
}

/// `shape n (face mode) x y z eps`: one Sinert over constructor and Perform steps.
pub fn sinert_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = tp_shape(&mut r)?;
        let n = r.integer();
        let steps: Vec<(usize, i32)> = (0..n)
            .map(|_| (r.integer() as usize, r.integer()))
            .collect();
        let (loc, eps) = (r.pnt(), r.real());
        let mut g: Option<BRepGPropSinert> = None;
        for (k, mode) in steps {
            let mut f = BRepGPropFace::with_face(&s.faces[k], false)?;
            let mut d = BRepGPropDomain::with_face(&s.faces[k])?;
            attempt(|| {
                sinert_step(&mut g, (&mut f, &mut d), mode, (loc, eps))?;
                let g = g.as_mut().expect("framework");
                real(g.get_epsilon());
                gprops(g);
                Ok(())
            });
        }
        Ok(())
    });
    end();
}

fn sinert_step(
    g: &mut Option<BRepGPropSinert>,
    (f, d): (&mut BRepGPropFace, &mut BRepGPropDomain),
    mode: i32,
    (loc, eps): (Point3, f64),
) -> Result<(), Failure> {
    match mode {
        0 => *g = Some(BRepGPropSinert::with_face(f, loc)?),
        1 => *g = Some(BRepGPropSinert::with_domain(f, d, loc)?),
        2 => *g = Some(BRepGPropSinert::with_face_eps(f, loc, eps)?),
        3 => *g = Some(BRepGPropSinert::with_domain_eps(f, d, loc, eps)?),
        _ => {
            let g = g.get_or_insert_with(|| {
                let mut g = BRepGPropSinert::new();
                g.set_location(loc);
                g
            });
            match mode {
                4 => g.perform(f)?,
                5 => g.perform_domain(f, d)?,
                6 => real(g.perform_eps(f, eps)?),
                _ => real(g.perform_domain_eps(f, d, eps)?),
            }
        }
    }
    Ok(())
}

/// The steps `(face, mode, bound)` of the Vinert and VinertGK requests.
fn steps(r: &mut Reader<'_>) -> Vec<(usize, i32, Bound)> {
    let n = r.integer();
    (0..n)
        .map(|_| {
            let (k, mode) = (r.integer() as usize, r.integer());
            (k, mode, bound(r))
        })
        .collect()
}

/// `shape n (face mode bound) x y z eps`: one Vinert over the steps.
pub fn vinert_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = tp_shape(&mut r)?;
        let steps = steps(&mut r);
        let (loc, eps) = (r.pnt(), r.real());
        let mut g: Option<BRepGPropVinert> = None;
        for (k, mode, b) in steps {
            let mut f = BRepGPropFace::with_face(&s.faces[k], false)?;
            let mut d = BRepGPropDomain::with_face(&s.faces[k])?;
            attempt(|| {
                vinert_step(&mut g, (&mut f, &mut d), (mode, b), (loc, eps))?;
                let g = g.as_mut().expect("framework");
                real(g.get_epsilon());
                gprops(g);
                Ok(())
            });
        }
        Ok(())
    });
    end();
}

/// `tp_vinert_step`: mode bit 0 adaptive, bit 1 with the domain, bit 2 a constructor.
fn vinert_step(
    g: &mut Option<BRepGPropVinert>,
    (f, d): (&mut BRepGPropFace, &mut BRepGPropDomain),
    (mode, b): (i32, Bound),
    (loc, eps): (Point3, f64),
) -> Result<(), Failure> {
    let (adaptive, domain) = (mode & 1 != 0, mode & 2 != 0);
    if mode & 4 != 0 {
        *g = Some(vinert_new(f, d, (adaptive, domain), b, (loc, eps))?);
        return Ok(());
    }
    let g = g.get_or_insert_with(|| {
        let mut g = BRepGPropVinert::new();
        g.set_location(loc);
        g
    });
    match (b, domain, adaptive) {
        (Bound::Location, true, true) => real(g.perform_domain_eps(f, d, eps)?),
        (Bound::Location, true, false) => g.perform_domain(f, d)?,
        (Bound::Location, false, true) => real(g.perform_eps(f, eps)?),
        (Bound::Location, false, false) => g.perform(f)?,
        (Bound::Point(o), true, true) => real(g.perform_domain_origin_eps(f, d, o, eps)?),
        (Bound::Point(o), true, false) => g.perform_domain_origin(f, d, o)?,
        (Bound::Point(o), false, true) => real(g.perform_origin_eps(f, o, eps)?),
        (Bound::Point(o), false, false) => g.perform_origin(f, o)?,
        (Bound::Plane(p), true, true) => real(g.perform_domain_plane_eps(f, d, p, eps)?),
        (Bound::Plane(p), true, false) => g.perform_domain_plane(f, d, p)?,
        (Bound::Plane(p), false, true) => real(g.perform_plane_eps(f, p, eps)?),
        (Bound::Plane(p), false, false) => g.perform_plane(f, p)?,
    }
    Ok(())
}

/// The Vinert constructors of `tp_vinert_step`.
fn vinert_new(
    f: &mut BRepGPropFace,
    d: &mut BRepGPropDomain,
    (adaptive, domain): (bool, bool),
    b: Bound,
    (loc, eps): (Point3, f64),
) -> Result<BRepGPropVinert, Failure> {
    match (b, domain, adaptive) {
        (Bound::Location, true, true) => BRepGPropVinert::with_domain_eps(f, d, loc, eps),
        (Bound::Location, true, false) => BRepGPropVinert::with_domain(f, d, loc),
        (Bound::Location, false, true) => BRepGPropVinert::with_face_eps(f, loc, eps),
        (Bound::Location, false, false) => BRepGPropVinert::with_face(f, loc),
        (Bound::Point(o), true, true) => BRepGPropVinert::with_domain_origin_eps(f, d, o, loc, eps),
        (Bound::Point(o), true, false) => BRepGPropVinert::with_domain_origin(f, d, o, loc),
        (Bound::Point(o), false, true) => BRepGPropVinert::with_face_origin_eps(f, o, loc, eps),
        (Bound::Point(o), false, false) => BRepGPropVinert::with_face_origin(f, o, loc),
        (Bound::Plane(p), true, true) => BRepGPropVinert::with_domain_plane_eps(f, d, p, loc, eps),
        (Bound::Plane(p), true, false) => BRepGPropVinert::with_domain_plane(f, d, p, loc),
        (Bound::Plane(p), false, true) => BRepGPropVinert::with_face_plane_eps(f, p, loc, eps),
        (Bound::Plane(p), false, false) => BRepGPropVinert::with_face_plane(f, p, loc),
    }
}

/// `shape n (face mode bound) x y z tol cg i span`: one VinertGK over the steps.
pub fn vinert_gk_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = tp_shape(&mut r)?;
        let steps = steps(&mut r);
        let loc = r.pnt();
        let options = VinertGKOptions {
            tolerance: r.real(),
            cg_flag: r.flag(),
            i_flag: r.flag(),
        };
        let span = r.flag();
        let mut g: Option<BRepGPropVinertGK> = None;
        for (k, mode, b) in steps {
            let mut f = BRepGPropFace::with_face(&s.faces[k], span)?;
            let mut d = BRepGPropDomain::with_face(&s.faces[k])?;
            attempt(|| {
                vinert_gk_step(&mut g, (&mut f, &mut d), (mode, b), (loc, options))?;
                let g = g.as_ref().expect("framework");
                real(g.get_error_reached());
                gprops(g);
                Ok(())
            });
        }
        Ok(())
    });
    end();
}

/// `tp_vinert_gk_step`: mode bit 0 with the domain, bit 1 a constructor.
fn vinert_gk_step(
    g: &mut Option<BRepGPropVinertGK>,
    (f, d): (&mut BRepGPropFace, &mut BRepGPropDomain),
    (mode, b): (i32, Bound),
    (loc, o): (Point3, VinertGKOptions),
) -> Result<(), Failure> {
    let domain = mode & 1 != 0;
    if mode & 2 != 0 {
        *g = Some(match (b, domain) {
            (Bound::Location, true) => BRepGPropVinertGK::with_domain(f, d, loc, o)?,
            (Bound::Location, false) => BRepGPropVinertGK::with_face(f, loc, o)?,
            (Bound::Point(p), true) => BRepGPropVinertGK::with_domain_point(f, d, p, loc, o)?,
            (Bound::Point(p), false) => BRepGPropVinertGK::with_face_point(f, p, loc, o)?,
            (Bound::Plane(p), true) => BRepGPropVinertGK::with_domain_plane(f, d, p, loc, o)?,
            (Bound::Plane(p), false) => BRepGPropVinertGK::with_face_plane(f, p, loc, o)?,
        });
        return Ok(());
    }
    let g = g.get_or_insert_with(|| {
        let mut g = BRepGPropVinertGK::new();
        g.set_location(loc);
        g
    });
    real(match (b, domain) {
        (Bound::Location, true) => g.perform_domain(f, d, o)?,
        (Bound::Location, false) => g.perform(f, o)?,
        (Bound::Point(p), true) => g.perform_domain_point(f, d, p, o)?,
        (Bound::Point(p), false) => g.perform_point(f, p, o)?,
        (Bound::Plane(p), true) => g.perform_domain_plane(f, d, p, o)?,
        (Bound::Plane(p), false) => g.perform_plane(f, p, o)?,
    });
    Ok(())
}
