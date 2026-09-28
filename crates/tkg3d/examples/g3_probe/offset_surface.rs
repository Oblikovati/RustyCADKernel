//! Rectangular trimmed surfaces and offset surfaces; the counterpart of
//! tools/oracle/tkg3d_probe_offset_surface.cxx.
use super::common::{
    Reader, attempt, built_surface, dump, end, flag, gtrsf2d, int, isos, name, real, show,
    surface_summary, trsf,
};
use super::shapes;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomGeometry, GeomOffsetSurface, GeomRectangularTrimmedSurface, GeomSurface,
};

/// Surface kinds 9 (two-direction trim), 10 (one-direction trim) and 11 (offset).
pub fn spec(kind: i32, r: &mut Reader) -> Result<Handle<dyn GeomSurface>, Failure> {
    match kind {
        9 => {
            let (u1, u2, v1, v2) = (r.real(), r.real(), r.real(), r.real());
            let (us, vs) = (r.flag(), r.flag());
            let basis = shapes::surface(r)?;
            let trimmed = GeomRectangularTrimmedSurface::new(&basis, (u1, u2), (v1, v2), (us, vs))?;
            Ok(Handle::new(trimmed).cast())
        }
        10 => {
            let (p1, p2) = (r.real(), r.real());
            let (u_trim, sense) = (r.flag(), r.flag());
            let basis = shapes::surface(r)?;
            let trimmed =
                GeomRectangularTrimmedSurface::with_one_direction(&basis, p1, p2, u_trim, sense)?;
            Ok(Handle::new(trimmed).cast())
        }
        _ => {
            let (d, no_check) = (r.real(), r.flag());
            let basis = shapes::surface(r)?;
            Ok(Handle::new(GeomOffsetSurface::new(&basis, d, no_check)?).cast())
        }
    }
}

fn trim_state(t: &GeomRectangularTrimmedSurface, uv: &[f64]) {
    name(&t.basis_surface());
    surface_summary(t, uv);
    dump(t, -1);
}

pub fn rect_trim_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(s) = built_surface(&mut r) else {
        return;
    };
    let (uv, iso, params) = (r.reals(), r.reals(), r.reals());
    let (u1, u2, v1, v2) = (r.real(), r.real(), r.real(), r.real());
    let (us, vs) = (r.flag(), r.flag());
    let (p1, p2) = (r.real(), r.real());
    let (u_trim, sense) = (r.flag(), r.flag());
    let m = trsf(&mut r);
    let t: Handle<GeomRectangularTrimmedSurface> = s.cast();
    trim_state(&t, &uv);
    show(t.copy(), |c| {
        trim_state(&c.cast::<GeomRectangularTrimmedSurface>(), &uv)
    });
    isos(&*t, &iso, &params);
    attempt(|| t.set_trim(u1, u2, v1, v2, us, vs));
    trim_state(&t, &uv);
    attempt(|| t.set_trim_direction(p1, p2, u_trim, sense));
    trim_state(&t, &uv);
    attempt(|| t.u_reverse());
    trim_state(&t, &uv);
    attempt(|| t.v_reverse());
    trim_state(&t, &uv);
    attempt(|| t.transform(m));
    trim_state(&t, &uv);
    isos(&*t, &iso, &params);
    end();
}

fn offset_state(o: &GeomOffsetSurface, uv: &[f64]) {
    real(o.offset());
    int(o.basis_surf_continuity() as i32);
    name(&o.basis_surface());
    flag(o.has_eval_representation());
    show(o.surface(), |e| {
        name(&e);
        if let Some(e) = e.get() {
            surface_summary(e, uv);
        }
    });
    surface_summary(o, uv);
    dump(o, -1);
}

fn osculating(o: &GeomOffsetSurface, uv: &[f64]) {
    for pair in uv.chunks(2).filter(|pair| pair.len() == 2) {
        let (u, v) = (pair[0], pair[1]);
        for along_u in [true, false] {
            let mut opposite = false;
            let mut surface: Handle<GeomBSplineSurface> = Handle::default();
            let found = if along_u {
                o.u_osculating_surface(u, v, &mut opposite, &mut surface)
            } else {
                o.v_osculating_surface(u, v, &mut opposite, &mut surface)
            };
            show(found, |found| {
                flag(found);
                flag(opposite);
                name(&surface);
                if let Some(s) = surface.get() {
                    surface_summary(s, uv);
                }
            });
        }
    }
}

pub fn offset_surface_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(s) = built_surface(&mut r) else {
        return;
    };
    let (uv, iso, params) = (r.reals(), r.reals(), r.reals());
    let (d, no_check) = (r.real(), r.flag());
    let m = trsf(&mut r);
    let basis: Option<Handle<dyn GeomSurface>> = match shapes::surface(&mut r) {
        Ok(basis) => Some(basis),
        Err(error) => {
            print!("{} ", error.exception_type());
            None
        }
    };
    let o: Handle<GeomOffsetSurface> = s.cast();
    offset_state(&o, &uv);
    osculating(&o, &uv);
    isos(&*o, &iso, &params);
    show(o.copy(), |c| {
        offset_state(&c.cast::<GeomOffsetSurface>(), &uv)
    });
    attempt(|| o.set_offset_value(d));
    offset_state(&o, &uv);
    attempt(|| o.u_reverse());
    offset_state(&o, &uv);
    attempt(|| o.v_reverse());
    offset_state(&o, &uv);
    attempt(|| o.transform(m));
    offset_state(&o, &uv);
    isos(&*o, &iso, &params);
    for pair in uv.chunks(2).filter(|pair| pair.len() == 2) {
        let (u, v) = o.transform_parameters(pair[0], pair[1], m);
        real(u);
        real(v);
    }
    show(o.parametric_transformation(m), gtrsf2d);
    if let Some(basis) = basis {
        attempt(|| o.set_basis_surface(&basis, no_check));
        offset_state(&o, &uv);
        osculating(&o, &uv);
    }
    attempt(|| o.set_basis_surface(&o.cast(), no_check));
    offset_state(&o, &uv);
    end();
}
