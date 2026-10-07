//! The two-surface fillet functions of BlendFunc (`fl_const_rad`, `fl_evol_rad`); the
//! twin of the fillet cases of tools/oracle/tkfillet_probe_blendfunc.cxx.
use super::blend_func::{
    app_queries, equations, guide, read_x, sections, shape_of, surface, tangents, to_vector, vec3,
};
use super::common::{Reader, attempt, dir, end, flag, pnt, real};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkfillet::blend::{BlendAppFunction, BlendFunction};
use rustycad_tkfillet::blend_func::{ConstRad, EvolRad};
use rustycad_tkgeomalgo::law::{LawConstant, LawFunction, LawInterpol, LawLinear, LawS};
use rustycad_tkmath::gp::{Circle3, Point2};

/// `fl_blend_members`: the members a two-surface blend function shares, at `x`;
/// `section` prints the section of the class.
pub(super) fn blend_members<F: BlendFunction>(
    f: &mut F,
    x: [f64; 4],
    t: f64,
    tol: f64,
    section: impl FnOnce(&mut F) -> Result<(), Failure>,
) {
    let xv = to_vector(x);
    equations(f, &xv);
    attempt(|| f.is_solution(&xv, tol).map(flag));
    tangents(f);
    attempt(|| {
        let frames = f.tangent(x[0], x[1], x[2], x[3])?;
        [
            frames.tg_first,
            frames.tg_last,
            frames.norm_first,
            frames.norm_last,
        ]
        .into_iter()
        .for_each(vec3);
        Ok(())
    });
    attempt(|| section(f));
    sections(f, x, t);
    app_queries(f, tol);
}

/// `fl_const_rad S1 S2 C radius choix shape t trim first last X1..X4 tol`.
pub fn const_rad_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (s1, s2, c) = (surface(&mut r), surface(&mut r), guide(&mut r));
    let (radius, choix, shape, t) = (r.real(), r.integer(), r.integer(), r.real());
    let (trim, first, last) = (r.flag(), r.real(), r.real());
    let (x, tol) = (read_x(&mut r), r.real());
    let mut f = ConstRad::new(&s1, &s2, &c).expect("function");
    f.set_radius(radius, choix);
    f.set_shape(shape_of(shape));
    if trim {
        attempt(|| f.set_interval(first, last));
    }
    let _ = f.set_param(t);
    blend_members(&mut f, x, t, tol, |f| {
        print_circle_section(f.section_circle(t, x)?)
    });
    attempt(|| {
        let axis = f.axe_rot(t)?;
        pnt(axis.location());
        dir(axis.direction());
        Ok(())
    });
    end();
}

/// `fl_law`: 7 numbers.
pub(super) fn law(r: &mut Reader<'_>) -> Handle<dyn LawFunction> {
    let kind = r.integer();
    let (p1, r1, p2, r2, r3) = (r.real(), r.real(), r.real(), r.real(), r.real());
    r.real();
    match kind {
        0 => {
            let l = LawConstant::new();
            l.set(r1, p1, p2);
            Handle::new(l).cast()
        }
        1 => {
            let l = LawLinear::new();
            l.set(p1, r1, p2, r2);
            Handle::new(l).cast()
        }
        2 => {
            let l = LawS::new();
            l.set(p1, r1, p2, r2).expect("law S");
            Handle::new(l).cast()
        }
        _ => {
            let mut buffer = [
                Point2::new(p1, r1),
                Point2::new((p1 + p2) / 2., r3),
                Point2::new(p2, r2),
            ];
            let l = LawInterpol::new();
            l.set(&Array1::from_slice(&mut buffer), false)
                .expect("interpolation");
            Handle::new(l).cast()
        }
    }
}

/// `fl_evol_rad S1 S2 C Law choix shape t trim first last X1..X4 tol`.
pub fn evol_rad_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (s1, s2, c) = (surface(&mut r), surface(&mut r), guide(&mut r));
    let radius_law = law(&mut r);
    let (choix, shape, t) = (r.integer(), r.integer(), r.real());
    let (trim, first, last) = (r.flag(), r.real(), r.real());
    let (x, tol) = (read_x(&mut r), r.real());
    let mut f = EvolRad::new(&s1, &s2, &c, &radius_law).expect("function");
    f.set_choix(choix);
    f.set_shape(shape_of(shape));
    if trim {
        attempt(|| f.set_interval(first, last));
    }
    let _ = f.set_param(t);
    blend_members(&mut f, x, t, tol, |f| {
        print_circle_section(f.section_circle(t, x)?)
    });
    end();
}

fn print_circle_section((pdeb, pfin, circle): (f64, f64, Circle3)) -> Result<(), Failure> {
    real(pdeb);
    real(pfin);
    let frame = circle.position();
    pnt(frame.location());
    dir(frame.direction());
    dir(frame.x_direction());
    real(circle.radius());
    Ok(())
}
