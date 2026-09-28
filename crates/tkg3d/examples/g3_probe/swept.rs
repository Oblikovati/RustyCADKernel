//! The swept surfaces and their evaluation representations; the counterpart of the
//! second half of tools/oracle/tkg3d_probe_surfaces.cxx.
use super::common::{
    Reader, ax1, ax2, built_surface, curve_summary, dir, dump, end, flag, int, name, pnt, show,
    surface_summary, trsf,
};
use super::shapes;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomSurface, GeomSurfaceOfLinearExtrusion, GeomSurfaceOfRevolution,
    GeomSweptSurface,
};
use rustycad_tkg3d::geom_eval::{
    Domain2d, Map2d, RepSurfaceDesc, RepSurfaceDescDerivBounded, RepSurfaceDescFull,
    RepSurfaceDescMapped,
};

fn swept_state(s: &Handle<dyn GeomSweptSurface>, uv: &[f64], params: &[f64]) {
    dir(s.direction());
    let basis = s.basis_curve();
    name(&basis);
    curve_summary(&*basis, params);
    int(s.continuity() as i32);
    if let Some(v) = s.cast::<GeomSurfaceOfRevolution>().get() {
        ax1(v.axis());
        pnt(v.location());
        show(v.reference_plane(), ax2);
    }
    surface_summary(&**s, uv);
    dump(&**s, -1);
}

pub fn swept_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(surface) = built_surface(&mut r) else {
        return;
    };
    let (uv, params) = (r.reals(), r.reals());
    let (d, a, p) = (r.dir(), r.ax1(), r.pnt());
    let c = shapes::curve(&mut r).expect("probe basis curve");
    let s: Handle<dyn GeomSweptSurface> = surface.cast();
    swept_state(&s, &uv, &params);
    if let Some(e) = s.cast::<GeomSurfaceOfLinearExtrusion>().get() {
        e.set_direction(d);
        swept_state(&s, &uv, &params);
        e.set_basis_curve(&c).unwrap();
        swept_state(&s, &uv, &params);
    }
    if let Some(v) = s.cast::<GeomSurfaceOfRevolution>().get() {
        v.set_direction(d);
        swept_state(&s, &uv, &params);
        v.set_location(p);
        swept_state(&s, &uv, &params);
        v.set_axis(a);
        swept_state(&s, &uv, &params);
        v.set_basis_curve(&c).unwrap();
        swept_state(&s, &uv, &params);
    }
    end();
}

// One descriptor spec, as g3_surface_desc reads it.
fn surface_desc(r: &mut Reader, rep: Handle<dyn GeomSurface>) -> Handle<dyn RepSurfaceDesc> {
    let kind = r.integer();
    if kind == 0 {
        return Handle::new(RepSurfaceDescFull::new(rep)).cast();
    }
    let order = r.integer() as usize;
    if kind == 1 {
        return Handle::new(RepSurfaceDescDerivBounded::new(rep, order)).cast();
    }
    let has_domain = r.flag();
    let domain = Domain2d {
        u_first: r.real(),
        u_last: r.real(),
        v_first: r.real(),
        v_last: r.real(),
    };
    let map = Map2d {
        scale_u: r.real(),
        offset_u: r.real(),
        scale_v: r.real(),
        offset_v: r.real(),
        swap_uv: r.flag(),
    };
    let domain = has_domain.then_some(domain);
    Handle::new(RepSurfaceDescMapped::new(rep, order, domain, map)).cast()
}

fn has_rep(s: &Handle<dyn GeomSurface>) -> bool {
    if let Some(b) = s.cast::<GeomBSplineSurface>().get() {
        return b.has_eval_representation();
    }
    if let Some(e) = s.cast::<GeomSurfaceOfLinearExtrusion>().get() {
        return e.has_eval_representation();
    }
    s.cast::<GeomSurfaceOfRevolution>()
        .get()
        .is_some_and(|v| v.has_eval_representation())
}

fn set_rep(s: &Handle<dyn GeomSurface>, d: Handle<dyn RepSurfaceDesc>) -> Result<(), Failure> {
    if let Some(b) = s.cast::<GeomBSplineSurface>().get() {
        return b.set_eval_representation(d);
    }
    if let Some(e) = s.cast::<GeomSurfaceOfLinearExtrusion>().get() {
        return e.set_eval_representation(d);
    }
    let revolution = s.cast::<GeomSurfaceOfRevolution>();
    revolution
        .get()
        .map_or(Ok(()), |v| v.set_eval_representation(d))
}

fn clear_rep(s: &Handle<dyn GeomSurface>) {
    if let Some(b) = s.cast::<GeomBSplineSurface>().get() {
        b.clear_eval_representation();
    }
    if let Some(e) = s.cast::<GeomSurfaceOfLinearExtrusion>().get() {
        e.clear_eval_representation();
    }
    if let Some(v) = s.cast::<GeomSurfaceOfRevolution>().get() {
        v.clear_eval_representation();
    }
}

pub fn surface_evalrep_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(s) = built_surface(&mut r) else {
        return;
    };
    let uv = r.reals();
    let rep = shapes::surface(&mut r).expect("probe representation surface");
    let d = surface_desc(&mut r, rep);
    let t = trsf(&mut r);
    flag(has_rep(&s));
    show(set_rep(&s, d.clone()), |()| ());
    flag(has_rep(&s));
    surface_summary(&*s, &uv);
    let c: Handle<dyn GeomSurface> = s.copy().unwrap().cast();
    flag(has_rep(&c));
    surface_summary(&*c, &uv);
    let own: Handle<dyn RepSurfaceDesc> = Handle::new(RepSurfaceDescFull::new(s.clone())).cast();
    show(set_rep(&s, own), |()| ());
    flag(has_rep(&s));
    let none: Handle<dyn RepSurfaceDesc> =
        Handle::new(RepSurfaceDescFull::new(Handle::default())).cast();
    show(set_rep(&s, none), |()| ());
    clear_rep(&s);
    flag(has_rep(&s));
    show(set_rep(&s, d), |()| ());
    s.transform(t).unwrap();
    flag(has_rep(&s));
    surface_summary(&*s, &uv);
    end();
}
